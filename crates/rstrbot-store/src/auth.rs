use crate::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

// Argon2 runs outside database transactions. This proof records exactly which
// stored credential was verified; it is rechecked under the user row's write
// lock before a session or replacement password can be committed.
struct PasswordProof {
    user_id: String,
    hash: String,
}

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
        let proof = self.verify_login_password(username, password).await?;
        self.create_verified_session(proof).await
    }

    async fn verify_login_password(&self, username: &str, password: &str) -> Result<PasswordProof> {
        if password.len() > 1024 {
            return Err(Error::Unauthorized);
        }
        let username = normalize_username(username).map_err(|_| Error::Unauthorized)?;
        let row =
            sqlx::query("SELECT id, password_hash FROM users WHERE username = $1 AND disabled = 0")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?
                .ok_or(Error::Unauthorized)?;
        verify_password(&row, password).await
    }

    async fn create_verified_session(&self, proof: PasswordProof) -> Result<TokenSession> {
        let mut tx = self.pool.begin().await?;
        // The conditional write also serializes with password change/recovery
        // on PostgreSQL. A check followed by an unlocked INSERT would let an
        // old login create a valid session after all sessions were revoked.
        let changed = sqlx::query("UPDATE users SET password_hash = password_hash WHERE id = $1 AND password_hash = $2 AND disabled = 0")
            .bind(&proof.user_id).bind(&proof.hash).execute(&mut *tx).await?.rows_affected();
        if changed != 1 {
            return Err(Error::Unauthorized);
        }
        let row = sqlx::query("SELECT id, username, instance_admin FROM users WHERE id = $1")
            .bind(&proof.user_id)
            .fetch_one(&mut *tx)
            .await?;
        let user = user_from_row(&row)?;
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
        if current.len() > 1024 {
            return Err(Error::Unauthorized);
        }
        let row = sqlx::query("SELECT id, password_hash FROM users WHERE id = $1 AND disabled = 0")
            .bind(&user.id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::Unauthorized)?;
        let proof = verify_password(&row, current).await?;
        let hash = hash_password(replacement).await?;
        self.replace_verified_password(proof, hash).await
    }

    async fn replace_verified_password(&self, proof: PasswordProof, hash: String) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let changed = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2 AND password_hash = $3 AND disabled = 0")
            .bind(hash)
            .bind(&proof.user_id)
            .bind(&proof.hash)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if changed != 1 {
            return Err(Error::Unauthorized);
        }
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(&proof.user_id)
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

async fn verify_password(row: &sqlx::any::AnyRow, password: &str) -> Result<PasswordProof> {
    let proof = PasswordProof {
        user_id: row.try_get("id")?,
        hash: row.try_get("password_hash")?,
    };
    let hash = proof.hash.clone();
    let password = password.to_owned();
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .map_err(|_| Error::Invalid("password verification worker stopped".into()))?;
    if valid {
        Ok(proof)
    } else {
        Err(Error::Unauthorized)
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

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "original correct horse battery staple";
    const REPLACEMENT: &str = "replacement correct horse battery staple";

    async fn database() -> (Store, Option<tempfile::TempDir>) {
        if let Ok(url) = std::env::var("RSTRBOT_TEST_DATABASE_URL") {
            sqlx::any::install_default_drivers();
            let pool = AnyPool::connect(&url).await.unwrap();
            let schema = format!("rstrbot_test_{}", uuid::Uuid::new_v4().simple());
            sqlx::query(&format!("CREATE SCHEMA {schema}"))
                .execute(&pool)
                .await
                .unwrap();
            pool.close().await;
            let separator = if url.contains('?') { '&' } else { '?' };
            let url = format!("{url}{separator}options=-csearch_path%3D{schema}");
            (Store::connect(&url).await.unwrap(), None)
        } else {
            let dir = tempfile::tempdir().unwrap();
            let url = Store::sqlite_file_url(&dir.path().join("auth.sqlite3")).unwrap();
            (Store::connect(&url).await.unwrap(), Some(dir))
        }
    }

    #[tokio::test]
    async fn password_change_rejects_an_already_verified_old_login() {
        let (store, _dir) = database().await;
        let original = store.bootstrap("alice", PASSWORD, "Auth").await.unwrap();
        // Pause the old login immediately after Argon2 succeeds. This forces
        // the dangerous ordering without sleeps or assumptions about CPU speed.
        let pending = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        store
            .change_password(&original.user, PASSWORD, REPLACEMENT)
            .await
            .unwrap();
        assert!(matches!(
            store.create_verified_session(pending).await,
            Err(Error::Unauthorized)
        ));
        assert!(matches!(
            store.authenticate(&original.token).await,
            Err(Error::Unauthorized)
        ));
        assert!(matches!(
            store.login("alice", PASSWORD).await,
            Err(Error::Unauthorized)
        ));
        let current = store.login("alice", REPLACEMENT).await.unwrap();
        assert_eq!(
            store.authenticate(&current.token).await.unwrap().id,
            original.user.id
        );
    }

    #[tokio::test]
    async fn recovery_rejects_an_already_verified_old_login() {
        let (store, _dir) = database().await;
        let original = store.bootstrap("alice", PASSWORD, "Auth").await.unwrap();
        let pending = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        store.recover_password("alice", REPLACEMENT).await.unwrap();
        assert!(matches!(
            store.create_verified_session(pending).await,
            Err(Error::Unauthorized)
        ));
        assert!(matches!(
            store.authenticate(&original.token).await,
            Err(Error::Unauthorized)
        ));
        assert!(store.login("alice", REPLACEMENT).await.is_ok());
    }

    #[tokio::test]
    async fn stale_password_change_cannot_overwrite_or_revoke_new_credentials() {
        let (store, _dir) = database().await;
        let original = store.bootstrap("alice", PASSWORD, "Auth").await.unwrap();
        let pending = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        let stale_replacement = "outdated replacement must never be committed";
        let pending_hash = hash_password(stale_replacement).await.unwrap();
        store
            .change_password(&original.user, PASSWORD, REPLACEMENT)
            .await
            .unwrap();
        let current = store.login("alice", REPLACEMENT).await.unwrap();
        assert!(matches!(
            store.replace_verified_password(pending, pending_hash).await,
            Err(Error::Unauthorized)
        ));
        assert!(store.authenticate(&current.token).await.is_ok());
        assert!(store.login("alice", REPLACEMENT).await.is_ok());
        assert!(matches!(
            store.login("alice", stale_replacement).await,
            Err(Error::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn verified_logins_recheck_disabled_state_and_current_admin_role() {
        let (store, _dir) = database().await;
        let original = store.bootstrap("alice", PASSWORD, "Auth").await.unwrap();
        let pending = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        sqlx::query("UPDATE users SET instance_admin = 0 WHERE id = $1")
            .bind(&original.user.id)
            .execute(&store.pool)
            .await
            .unwrap();
        let current = store.create_verified_session(pending).await.unwrap();
        assert!(!current.user.instance_admin);

        let pending_login = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        let pending_change = store
            .verify_login_password("alice", PASSWORD)
            .await
            .unwrap();
        let replacement_hash = hash_password(REPLACEMENT).await.unwrap();
        sqlx::query("UPDATE users SET disabled = 1 WHERE id = $1")
            .bind(&original.user.id)
            .execute(&store.pool)
            .await
            .unwrap();
        assert!(matches!(
            store.create_verified_session(pending_login).await,
            Err(Error::Unauthorized)
        ));
        assert!(matches!(
            store
                .replace_verified_password(pending_change, replacement_hash)
                .await,
            Err(Error::Unauthorized)
        ));
        assert!(matches!(
            store.authenticate(&current.token).await,
            Err(Error::Unauthorized)
        ));
    }
}
