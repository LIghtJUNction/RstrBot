use crate::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

impl Store {
    /// The HTTP/CLI boundary verifies the short-lived initialization proof.
    /// The unique marker makes account creation atomic even across racing clients.
    pub async fn bootstrap(
        &self,
        username: &str,
        password: &str,
        workspace: &str,
    ) -> Result<TokenSession> {
        let username = normalize_username(username)?;
        let name = checked_label(workspace, "workspace name", 200)?;
        let hash = hash_password(password).await?;
        let user = User {
            id: new_id(),
            username,
            instance_admin: true,
        };
        let workspace_id = new_id();
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO instance_state (key, value) VALUES ('initialized', 'true')")
            .execute(&mut *tx)
            .await
            .map_err(unique_error)?;
        insert_user(&mut tx, &user, &hash).await?;
        create_workspace_in(&mut tx, &workspace_id, &name, &user.id).await?;
        let session = create_session_in(&mut tx, user).await?;
        tx.commit().await?;
        Ok(session)
    }

    pub async fn create_user(&self, actor: &User, username: &str, password: &str) -> Result<User> {
        self.require_instance_admin(&actor.id).await?;
        let username = normalize_username(username)?;
        let hash = hash_password(password).await?;
        let user = User {
            id: new_id(),
            username,
            instance_admin: false,
        };
        let mut tx = self.pool.begin().await?;
        insert_user(&mut tx, &user, &hash).await?;
        tx.commit().await?;
        Ok(user)
    }

    pub async fn users(&self, actor: &User) -> Result<Vec<User>> {
        self.require_instance_admin(&actor.id).await?;
        let rows = sqlx::query("SELECT id, username, instance_admin FROM users WHERE disabled = 0 ORDER BY username LIMIT 1000")
            .fetch_all(&self.pool).await?;
        rows.iter().map(user_from_row).collect()
    }

    pub async fn user_by_username(&self, username: &str) -> Result<User> {
        let username = normalize_username(username)?;
        let row = sqlx::query(
            "SELECT id, username, instance_admin FROM users WHERE username = $1 AND disabled = 0",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::NotFound)?;
        user_from_row(&row)
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<TokenSession> {
        if password.len() > 1024 {
            return Err(Error::Unauthorized);
        }
        let username = normalize_username(username).map_err(|_| Error::Unauthorized)?;
        let row = sqlx::query("SELECT id, username, instance_admin, password_hash FROM users WHERE username = $1 AND disabled = 0")
            .bind(username).fetch_optional(&self.pool).await?.ok_or(Error::Unauthorized)?;
        let password = password.to_owned();
        let hash: String = row.try_get("password_hash")?;
        let valid = tokio::task::spawn_blocking(move || {
            PasswordHash::new(&hash).is_ok_and(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
        })
        .await
        .map_err(|_| Error::Invalid("password verification worker stopped".into()))?;
        if !valid {
            return Err(Error::Unauthorized);
        }
        let user = user_from_row(&row)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND expires_at <= $2")
            .bind(&user.id)
            .bind(now())
            .execute(&mut *tx)
            .await?;
        let session = create_session_in(&mut tx, user).await?;
        tx.commit().await?;
        Ok(session)
    }

    pub async fn authenticate(&self, token: &str) -> Result<User> {
        if token.len() != 64 {
            return Err(Error::Unauthorized);
        }
        let row = sqlx::query("SELECT u.id, u.username, u.instance_admin FROM users u JOIN sessions s ON s.user_id = u.id WHERE s.token_hash = $1 AND s.expires_at > $2 AND u.disabled = 0")
            .bind(token_hash(token)).bind(now()).fetch_optional(&self.pool).await?.ok_or(Error::Unauthorized)?;
        user_from_row(&row)
    }

    pub async fn revoke(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(token_hash(token))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Password changes require the current password and revoke every active session.
    pub async fn change_password(
        &self,
        user: &User,
        current: &str,
        replacement: &str,
    ) -> Result<()> {
        let verification = self.login(&user.username, current).await?;
        self.revoke(&verification.token).await?;
        let hash = hash_password(replacement).await?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(hash)
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Local operator recovery. Expose only to a separate trusted CLI proof, never unauthenticated HTTP.
    pub async fn recover_password(&self, username: &str, replacement: &str) -> Result<()> {
        let user = self.user_by_username(username).await?;
        let hash = hash_password(replacement).await?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(hash)
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO instance_state (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = excluded.value")
            .bind(format!("password_recovery:{}", user.id)).bind(now().to_string()).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}

fn normalize_username(value: &str) -> Result<String> {
    let value = value.trim().to_lowercase();
    if !(3..=64).contains(&value.len())
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-@.".contains(c))
    {
        return Err(Error::Invalid(
            "username must contain 3–64 ASCII letters, digits, _, -, @ or .".into(),
        ));
    }
    Ok(value)
}

async fn hash_password(password: &str) -> Result<String> {
    if !(12..=1024).contains(&password.len()) {
        return Err(Error::Invalid("password must contain 12–1024 bytes".into()));
    }
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| Error::Invalid("password hashing failed".into()))
    })
    .await
    .map_err(|_| Error::Invalid("password hashing worker stopped".into()))?
}

fn user_from_row(row: &sqlx::any::AnyRow) -> Result<User> {
    Ok(User {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        instance_admin: row.try_get::<i64, _>("instance_admin")? == 1,
    })
}

async fn insert_user(tx: &mut Transaction<'_, Any>, user: &User, hash: &str) -> Result<()> {
    sqlx::query("INSERT INTO users (id, username, password_hash, instance_admin, disabled, created_at) VALUES ($1, $2, $3, $4, 0, $5)")
        .bind(&user.id).bind(&user.username).bind(hash).bind(i64::from(user.instance_admin)).bind(now())
        .execute(&mut **tx).await.map_err(unique_error)?;
    Ok(())
}

pub(crate) async fn create_workspace_in(
    tx: &mut Transaction<'_, Any>,
    id: &str,
    name: &str,
    owner: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO workspaces (id, name, revision, created_at) VALUES ($1, $2, 0, $3)")
        .bind(id)
        .bind(name)
        .bind(now())
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO members (workspace_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(id)
        .bind(owner)
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO budgets (workspace_id, limit_microusd, spent_microusd, reserved_microusd) VALUES ($1, -1, 0, 0)")
        .bind(id).execute(&mut **tx).await?;
    Ok(())
}

async fn create_session_in(tx: &mut Transaction<'_, Any>, user: User) -> Result<TokenSession> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let expires_at = now() + 7 * 24 * 60 * 60;
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at, created_at) VALUES ($1, $2, $3, $4)")
        .bind(token_hash(&token)).bind(&user.id).bind(expires_at).bind(now()).execute(&mut **tx).await?;
    Ok(TokenSession {
        token,
        user,
        expires_at,
    })
}

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
