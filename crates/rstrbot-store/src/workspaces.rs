use crate::*;

impl Store {
    pub async fn workspaces(&self, user: &str) -> Result<Vec<Workspace>> {
        let rows = sqlx::query("SELECT w.id, w.name, w.created_at, m.role FROM workspaces w JOIN members m ON m.workspace_id = w.id JOIN users u ON u.id = m.user_id WHERE m.user_id = $1 AND u.disabled = 0 ORDER BY w.created_at, w.id")
            .bind(user).fetch_all(&self.pool).await?;
        rows.iter()
            .map(|r| {
                Ok(Workspace {
                    id: r.try_get("id")?,
                    name: r.try_get("name")?,
                    role: Role::parse(&r.try_get::<String, _>("role")?)?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn create_workspace(&self, user: &str, name: &str) -> Result<Workspace> {
        let name = checked_label(name, "workspace name", 200)?;
        let id = new_id();
        let mut tx = self.pool.begin().await?;
        let active: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND disabled = 0")
                .bind(user)
                .fetch_one(&mut *tx)
                .await?;
        if active == 0 {
            return Err(Error::Unauthorized);
        }
        crate::auth::create_workspace_in(&mut tx, &id, &name, user).await?;
        tx.commit().await?;
        Ok(Workspace {
            id,
            name,
            role: Role::Owner,
            created_at: now(),
        })
    }

    pub async fn membership(&self, workspace: &str, user: &str) -> Result<Role> {
        let role: Option<String> = sqlx::query_scalar("SELECT m.role FROM members m JOIN users u ON u.id = m.user_id WHERE m.workspace_id = $1 AND m.user_id = $2 AND u.disabled = 0")
            .bind(workspace).bind(user).fetch_optional(&self.pool).await?;
        role.map(|v| Role::parse(&v))
            .transpose()?
            .ok_or(Error::Forbidden)
    }

    pub async fn require_role(&self, workspace: &str, user: &str, minimum: Role) -> Result<Role> {
        let role = self.membership(workspace, user).await?;
        if role >= minimum {
            Ok(role)
        } else {
            Err(Error::Forbidden)
        }
    }

    pub async fn members(&self, workspace: &str, user: &str) -> Result<Vec<Member>> {
        self.require_role(workspace, user, Role::Admin).await?;
        let rows = sqlx::query("SELECT m.user_id, u.username, m.role FROM members m JOIN users u ON u.id = m.user_id WHERE m.workspace_id = $1 ORDER BY u.username")
            .bind(workspace).fetch_all(&self.pool).await?;
        rows.iter()
            .map(|r| {
                Ok(Member {
                    user_id: r.try_get("user_id")?,
                    username: r.try_get("username")?,
                    role: Role::parse(&r.try_get::<String, _>("role")?)?,
                })
            })
            .collect()
    }

    pub async fn set_member(
        &self,
        workspace: &str,
        actor: &str,
        target: &str,
        role: Role,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        lock_workspace(&mut tx, workspace).await?;
        let actor_role = role_in(&mut tx, workspace, actor).await?;
        let current = match role_in(&mut tx, workspace, target).await {
            Ok(role) => Some(role),
            Err(Error::Forbidden) => None,
            Err(error) => return Err(error),
        };
        if actor_role < Role::Admin
            || (actor_role != Role::Owner
                && (role >= Role::Admin || current.is_some_and(|v| v >= Role::Admin)))
        {
            return Err(Error::Forbidden);
        }
        if current == Some(Role::Owner) && role != Role::Owner {
            preserve_owner(&mut tx, workspace).await?;
        }
        let valid: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND disabled = 0")
                .bind(target)
                .fetch_one(&mut *tx)
                .await?;
        if valid == 0 {
            return Err(Error::NotFound);
        }
        sqlx::query("INSERT INTO members (workspace_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT (workspace_id, user_id) DO UPDATE SET role = excluded.role")
            .bind(workspace).bind(target).bind(role.as_str()).execute(&mut *tx).await?;
        crate::resources::record_event(
            &mut tx,
            workspace,
            actor,
            Visibility::Workspace,
            workspace,
            "membership.updated",
            &serde_json::json!({"user_id": target, "role":role}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn remove_member(&self, workspace: &str, actor: &str, target: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        lock_workspace(&mut tx, workspace).await?;
        let actor_role = role_in(&mut tx, workspace, actor).await?;
        let target_role = role_in(&mut tx, workspace, target).await?;
        if actor_role < Role::Admin || (actor_role != Role::Owner && target_role >= Role::Admin) {
            return Err(Error::Forbidden);
        }
        if target_role == Role::Owner {
            preserve_owner(&mut tx, workspace).await?;
        }
        sqlx::query("DELETE FROM members WHERE workspace_id = $1 AND user_id = $2")
            .bind(workspace)
            .bind(target)
            .execute(&mut *tx)
            .await?;
        crate::resources::record_event(
            &mut tx,
            workspace,
            actor,
            Visibility::Workspace,
            workspace,
            "membership.removed",
            &serde_json::json!({"user_id":target}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

async fn preserve_owner(tx: &mut Transaction<'_, Any>, workspace: &str) -> Result<()> {
    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM members WHERE workspace_id = $1 AND role = 'owner'",
    )
    .bind(workspace)
    .fetch_one(&mut **tx)
    .await?;
    if owners < 2 {
        return Err(Error::Invalid("a workspace must retain an owner".into()));
    }
    Ok(())
}
