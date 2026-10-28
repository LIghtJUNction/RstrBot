//! Durable account bindings and one-time authentication state.
//!
//! Methods ending in `_verified` are trusted core operations: the caller must
//! first validate the Google ID token or WebAuthn ceremony with a maintained
//! verifier. Email strings, credential IDs and JSON are never themselves proof
//! of identity. No method in this module auto-links accounts by email.

use crate::auth::{create_session_in, hash_password, token_hash, user_from_row, verify_password};
use crate::*;
use rand::{RngCore, rngs::OsRng};

const RECENT_AUTH_SECONDS: i64 = 300;
const EMAIL_TOKEN_SECONDS: i64 = 900;
const MAX_AUTH_METHODS: i64 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedEmail {
    pub email: String,
    pub verified_at: i64,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleIdentity {
    pub id: String,
    pub user_id: String,
    pub subject: String,
    /// Informational claim only. This does not establish a verified-email login.
    pub email: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: String,
    pub user_id: String,
    pub credential_id: String,
    pub name: String,
    /// Server-owned WebAuthn verifier state. HTTP summaries should omit this.
    pub credential: Value,
    pub version: i64,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthAccount {
    pub email: Option<VerifiedEmail>,
    pub google: Vec<GoogleIdentity>,
    pub passkeys: Vec<PasskeyCredential>,
    pub has_password: bool,
    pub recent_auth_until: i64,
}

#[derive(Clone, Serialize)]
pub struct AuthChallenge {
    pub token: String,
    pub expires_at: i64,
}

impl std::fmt::Debug for AuthChallenge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthChallenge")
            .field("token", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySetting {
    pub revision: i64,
    pub value: Value,
}

impl Store {
    /// Current authenticated account summary. This deliberately excludes tokens
    /// and password hashes. The HTTP layer should also omit verifier JSON.
    pub async fn auth_account(&self, session: &str) -> Result<AuthAccount> {
        let mut tx = self.pool.begin().await?;
        let (user, recent_auth_until) = session_in(&mut tx, session, false).await?;
        let email = email_in(&mut tx, &user.id).await?;
        let google =
            sqlx::query("SELECT * FROM auth_google WHERE user_id = $1 ORDER BY created_at, id")
                .bind(&user.id)
                .fetch_all(&mut *tx)
                .await?
                .iter()
                .map(google_from_row)
                .collect::<Result<_>>()?;
        let passkeys =
            sqlx::query("SELECT * FROM auth_passkeys WHERE user_id = $1 ORDER BY created_at, id")
                .bind(&user.id)
                .fetch_all(&mut *tx)
                .await?
                .iter()
                .map(passkey_from_row)
                .collect::<Result<_>>()?;
        let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(&user.id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(AuthAccount {
            email,
            google,
            passkeys,
            has_password: !hash.is_empty(),
            recent_auth_until,
        })
    }

    pub async fn require_recent_session(&self, session: &str) -> Result<User> {
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        tx.commit().await?;
        Ok(user)
    }

    /// Verify the current password without creating another session or extending
    /// the session's absolute expiration. Recheck the exact hash after Argon2.
    pub async fn reauthenticate_password(&self, session: &str, password: &str) -> Result<i64> {
        if password.len() > 1024 {
            return Err(Error::Unauthorized);
        }
        let user = self.authenticate(session).await?;
        let row = sqlx::query("SELECT id, password_hash FROM users WHERE id = $1 AND disabled = 0")
            .bind(&user.id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::Unauthorized)?;
        let proof = verify_password(&row, password).await?;
        let mut tx = self.pool.begin().await?;
        let (current, _) = session_in(&mut tx, session, false).await?;
        let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(&current.id)
            .fetch_one(&mut *tx)
            .await?;
        if current.id != proof.user_id || hash != proof.hash {
            return Err(Error::Unauthorized);
        }
        let until = mark_recent_in(&mut tx, session).await?;
        tx.commit().await?;
        Ok(until)
    }

    /// Begin a same-session email binding/replacement. Sending the email belongs
    /// to the core; only a SHA-256 token digest and bounded intent are stored.
    pub async fn begin_email_change(&self, session: &str, email: &str) -> Result<AuthChallenge> {
        let email = normalize_email(email)?;
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        lock_identity(&mut tx).await?;
        ensure_email_available(&mut tx, &user.id, &email).await?;
        delivery_limit(&mut tx, &format!("email_change:{}", user.id)).await?;
        let previous = email_in(&mut tx, &user.id).await?;
        // Resending supersedes the previous pending intent, never the currently
        // verified address. A failed delivery cannot remove the existing email.
        delete_challenges_in(&mut tx, &user.id, "email_change").await?;
        let result = insert_challenge(&mut tx, "email_change", &user.id, &token_hash(session),
            serde_json::json!({"email": email, "email_version": previous.map(|e| e.version).unwrap_or(0)}), EMAIL_TOKEN_SECONDS).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Consume and bind atomically. The session that requested the mail must
    /// still be active; a later change of address invalidates older intents.
    pub async fn consume_email_change(
        &self,
        session: &str,
        challenge: &str,
    ) -> Result<VerifiedEmail> {
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, false).await?;
        lock_identity(&mut tx).await?;
        let row = challenge_in(&mut tx, challenge, "email_change").await?;
        if row.try_get::<String, _>("user_id")? != user.id
            || row.try_get::<String, _>("session_hash")? != token_hash(session)
        {
            return Err(Error::Unauthorized);
        }
        let data: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        let email = normalize_email(data["email"].as_str().ok_or(Error::Unauthorized)?)?;
        let old_version = email_in(&mut tx, &user.id)
            .await?
            .map(|e| e.version)
            .unwrap_or(0);
        if data["email_version"].as_i64() != Some(old_version) {
            return Err(Error::Conflict);
        }
        ensure_email_available(&mut tx, &user.id, &email).await?;
        let bound = VerifiedEmail {
            email,
            verified_at: now(),
            version: old_version.checked_add(1).ok_or(Error::Conflict)?,
        };
        sqlx::query("INSERT INTO auth_emails (user_id, email, verified_at, version) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET email = excluded.email, verified_at = excluded.verified_at, version = excluded.version")
            .bind(&user.id).bind(&bound.email).bind(bound.verified_at).bind(bound.version).execute(&mut *tx).await.map_err(unique_error)?;
        delete_challenges_in(&mut tx, &user.id, "email_change").await?;
        delete_challenges_in(&mut tx, &user.id, "password_reset").await?;
        tx.commit().await?;
        Ok(bound)
    }

    /// The HTTP caller always returns a generic response, including None and
    /// throttled requests. Only explicitly verified addresses can reset a local
    /// password. Google profile email claims never enter this lookup.
    pub async fn begin_password_reset(&self, email: &str) -> Result<Option<AuthChallenge>> {
        let email = match normalize_email(email) {
            Ok(email) => email,
            Err(_) => return Ok(None),
        };
        let user: Option<String> = sqlx::query_scalar("SELECT u.id FROM users u JOIN auth_emails e ON e.user_id = u.id WHERE e.email = $1 AND u.disabled = 0")
            .bind(&email).fetch_optional(&self.pool).await?;
        let Some(user) = user else {
            return Ok(None);
        };
        let mut tx = self.pool.begin().await?;
        match lock_user(&mut tx, &user).await {
            Ok(_) => {}
            Err(Error::Unauthorized) => return Ok(None),
            Err(error) => return Err(error),
        }
        lock_identity(&mut tx).await?;
        let Some(current) = email_in(&mut tx, &user).await? else {
            return Ok(None);
        };
        if current.email != email {
            return Ok(None);
        }
        match delivery_limit(&mut tx, &format!("password_reset:{user}")).await {
            Ok(()) => {}
            Err(Error::RateLimited) => return Ok(None),
            Err(error) => return Err(error),
        }
        let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(&user)
            .fetch_one(&mut *tx)
            .await?;
        delete_challenges_in(&mut tx, &user, "password_reset").await?;
        let challenge = match insert_challenge(&mut tx, "password_reset", &user, "",
            serde_json::json!({"email_version":current.version,"password_fingerprint":token_hash(&hash)}), EMAIL_TOKEN_SECONDS).await {
                Ok(challenge) => challenge,
                Err(Error::RateLimited) => return Ok(None),
                Err(error) => return Err(error),
            };
        tx.commit().await?;
        Ok(Some(challenge))
    }

    pub async fn finish_password_reset(&self, challenge: &str, replacement: &str) -> Result<()> {
        validate_token(challenge)?;
        // Reject unknown/expired tokens before doing expensive password work.
        let user: String = sqlx::query_scalar("SELECT user_id FROM auth_challenges WHERE token_hash = $1 AND purpose = 'password_reset' AND expires_at > $2")
            .bind(token_hash(challenge)).bind(now()).fetch_optional(&self.pool).await?.ok_or(Error::Unauthorized)?;
        let new_hash = hash_password(replacement).await?;
        let mut tx = self.pool.begin().await?;
        lock_user(&mut tx, &user).await?;
        lock_identity(&mut tx).await?;
        let row = challenge_in(&mut tx, challenge, "password_reset").await?;
        if row.try_get::<String, _>("user_id")? != user {
            return Err(Error::Unauthorized);
        }
        let data: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        let email = email_in(&mut tx, &user).await?.ok_or(Error::Unauthorized)?;
        let old_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(&user)
            .fetch_one(&mut *tx)
            .await?;
        if data["email_version"].as_i64() != Some(email.version)
            || data["password_fingerprint"].as_str() != Some(token_hash(&old_hash).as_str())
        {
            return Err(Error::Unauthorized);
        }
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(new_hash)
            .bind(&user)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(&user)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM auth_challenges WHERE user_id = $1")
            .bind(&user)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Store a bounded, expiring server-side ceremony. A bound session must be
    /// current, and the payload is never supplied to an authenticator as trusted
    /// state. The random returned token is stored only as a digest.
    pub async fn create_auth_challenge(
        &self,
        purpose: &str,
        user_id: Option<&str>,
        session: Option<&str>,
        payload: Value,
        ttl_seconds: i64,
    ) -> Result<AuthChallenge> {
        validate_purpose(purpose)?;
        if !(1..=600).contains(&ttl_seconds) {
            return Err(Error::Invalid(
                "authentication challenge lifetime must be 1–600 seconds".into(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let bound_user = if let Some(session) = session {
            let (user, _) = session_in(&mut tx, session, false).await?;
            if user_id.is_some_and(|id| id != user.id) {
                return Err(Error::Unauthorized);
            }
            user.id
        } else {
            user_id.unwrap_or_default().to_owned()
        };
        lock_identity(&mut tx).await?;
        if !bound_user.is_empty() {
            let active: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND disabled = 0")
                    .bind(&bound_user)
                    .fetch_one(&mut *tx)
                    .await?;
            if active != 1 {
                return Err(Error::Unauthorized);
            }
        }
        let challenge = insert_challenge(
            &mut tx,
            purpose,
            &bound_user,
            &session.map(token_hash).unwrap_or_default(),
            payload,
            ttl_seconds,
        )
        .await?;
        tx.commit().await?;
        Ok(challenge)
    }

    /// Consume before external proof verification: a failed proof cannot retry
    /// the same challenge. Concurrent finish requests can consume it only once.
    pub async fn take_auth_challenge(
        &self,
        token: &str,
        purpose: &str,
        session: Option<&str>,
    ) -> Result<Value> {
        validate_purpose(purpose)?;
        let mut tx = self.pool.begin().await?;
        let user = if let Some(session) = session {
            Some(session_in(&mut tx, session, false).await?.0)
        } else {
            None
        };
        lock_identity(&mut tx).await?;
        let row = challenge_in(&mut tx, token, purpose).await?;
        let bound = row.try_get::<String, _>("session_hash")?;
        if bound != session.map(token_hash).unwrap_or_default() {
            return Err(Error::Unauthorized);
        }
        let owner = row.try_get::<String, _>("user_id")?;
        if user.as_ref().is_some_and(|user| user.id != owner) {
            return Err(Error::Unauthorized);
        }
        if !owner.is_empty() {
            let active: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND disabled = 0")
                    .bind(&owner)
                    .fetch_one(&mut *tx)
                    .await?;
            if active != 1 {
                return Err(Error::Unauthorized);
            }
        }
        let result = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        sqlx::query("DELETE FROM auth_challenges WHERE token_hash = $1")
            .bind(token_hash(token))
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn link_google_verified(
        &self,
        session: &str,
        subject: &str,
        email: Option<&str>,
        expected_config_revision: i64,
    ) -> Result<GoogleIdentity> {
        let subject = checked_label(subject, "Google subject", 255)?;
        let email = email.map(normalize_email).transpose()?;
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        lock_identity(&mut tx).await?;
        check_config_revision(&mut tx, expected_config_revision).await?;
        if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM auth_google WHERE user_id = $1")
            .bind(&user.id)
            .fetch_one(&mut *tx)
            .await?
            >= MAX_AUTH_METHODS
        {
            return Err(Error::Invalid(
                "at most 20 Google accounts may be linked".into(),
            ));
        }
        let identity = GoogleIdentity {
            id: new_id(),
            user_id: user.id,
            subject,
            email,
            created_at: now(),
            last_used_at: None,
        };
        sqlx::query("INSERT INTO auth_google (id, user_id, subject, email, created_at, last_used_at) VALUES ($1, $2, $3, $4, $5, NULL)")
            .bind(&identity.id).bind(&identity.user_id).bind(&identity.subject).bind(&identity.email).bind(identity.created_at).execute(&mut *tx).await.map_err(unique_error)?;
        tx.commit().await?;
        Ok(identity)
    }

    pub async fn login_google_verified(
        &self,
        subject: &str,
        expected_config_revision: i64,
    ) -> Result<TokenSession> {
        checked_label(subject, "Google subject", 255)?;
        let user_id: String =
            sqlx::query_scalar("SELECT user_id FROM auth_google WHERE subject = $1")
                .bind(subject)
                .fetch_optional(&self.pool)
                .await?
                .ok_or(Error::Unauthorized)?;
        let mut tx = self.pool.begin().await?;
        let user = lock_user(&mut tx, &user_id).await?;
        lock_identity(&mut tx).await?;
        check_config_revision(&mut tx, expected_config_revision).await?;
        let id: String =
            sqlx::query_scalar("SELECT id FROM auth_google WHERE subject = $1 AND user_id = $2")
                .bind(subject)
                .bind(&user_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(Error::Unauthorized)?;
        sqlx::query("UPDATE auth_google SET last_used_at = $1 WHERE id = $2")
            .bind(now())
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        let result = external_session(&mut tx, user, "google", &id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn remove_google(&self, session: &str, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        lock_identity(&mut tx).await?;
        let changed = sqlx::query("DELETE FROM auth_google WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(&user.id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if changed != 1 {
            return Err(Error::NotFound);
        }
        retain_signin_method(&mut tx, &user.id).await?;
        revoke_method_sessions(&mut tx, "google", id).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Only the core's WebAuthn verifier may consume these credential records.
    /// Unknown/ambiguous identifiers return no credentials, never email links.
    pub async fn passkeys_for_username(&self, username: &str) -> Result<Vec<PasskeyCredential>> {
        let login = normalize_login(username)?;
        let users: Vec<String> = sqlx::query_scalar("SELECT id FROM users WHERE disabled = 0 AND (username = $1 OR id IN (SELECT user_id FROM auth_emails WHERE email = $1)) LIMIT 2")
            .bind(login).fetch_all(&self.pool).await?;
        if users.len() != 1 {
            return Ok(Vec::new());
        }
        sqlx::query("SELECT * FROM auth_passkeys WHERE user_id = $1 ORDER BY created_at, id")
            .bind(&users[0])
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(passkey_from_row)
            .collect()
    }

    pub async fn passkey_by_credential_id(&self, credential_id: &str) -> Result<PasskeyCredential> {
        let row = sqlx::query("SELECT p.* FROM auth_passkeys p JOIN users u ON u.id = p.user_id WHERE p.credential_id = $1 AND u.disabled = 0")
            .bind(credential_id).fetch_optional(&self.pool).await?.ok_or(Error::Unauthorized)?;
        passkey_from_row(&row)
    }

    pub async fn add_passkey_verified(
        &self,
        session: &str,
        credential_id: &str,
        name: &str,
        credential: Value,
        replace_id: Option<&str>,
        expected_config_revision: i64,
    ) -> Result<PasskeyCredential> {
        validate_credential_id(credential_id)?;
        let name = checked_label(name, "passkey name", 100)?;
        let encoded = encode_payload(&credential)?;
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        lock_identity(&mut tx).await?;
        check_config_revision(&mut tx, expected_config_revision).await?;
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM auth_passkeys WHERE user_id = $1")
                .bind(&user.id)
                .fetch_one(&mut *tx)
                .await?;
        if count >= MAX_AUTH_METHODS && replace_id.is_none() {
            return Err(Error::Invalid("at most 20 passkeys may be enrolled".into()));
        }
        if let Some(old) = replace_id {
            let owned: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM auth_passkeys WHERE id = $1 AND user_id = $2",
            )
            .bind(old)
            .bind(&user.id)
            .fetch_one(&mut *tx)
            .await?;
            if owned != 1 {
                return Err(Error::NotFound);
            }
        }
        let version = next_credential_version(&mut tx).await?;
        let key = PasskeyCredential {
            id: new_id(),
            user_id: user.id,
            credential_id: credential_id.into(),
            name,
            credential,
            version,
            created_at: now(),
            last_used_at: None,
        };
        // First insert the verified replacement. Any failure rolls back before
        // the old credential or its sessions are touched.
        sqlx::query("INSERT INTO auth_passkeys (id, user_id, credential_id, name, credential, version, created_at, last_used_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL)")
            .bind(&key.id).bind(&key.user_id).bind(&key.credential_id).bind(&key.name).bind(encoded).bind(key.version).bind(key.created_at).execute(&mut *tx).await.map_err(unique_error)?;
        if let Some(old) = replace_id {
            sqlx::query("DELETE FROM auth_passkeys WHERE id = $1 AND user_id = $2")
                .bind(old)
                .bind(&key.user_id)
                .execute(&mut *tx)
                .await?;
            revoke_method_sessions(&mut tx, "passkey", old).await?;
        }
        tx.commit().await?;
        Ok(key)
    }

    pub async fn login_passkey_verified(
        &self,
        credential_id: &str,
        expected_version: i64,
        updated_credential: Value,
        expected_config_revision: i64,
    ) -> Result<TokenSession> {
        let key = self.passkey_by_credential_id(credential_id).await?;
        let mut tx = self.pool.begin().await?;
        let user = lock_user(&mut tx, &key.user_id).await?;
        lock_identity(&mut tx).await?;
        check_config_revision(&mut tx, expected_config_revision).await?;
        update_verified_passkey(
            &mut tx,
            &key.user_id,
            credential_id,
            expected_version,
            &updated_credential,
        )
        .await?;
        let session = external_session(&mut tx, user, "passkey", &key.id).await?;
        tx.commit().await?;
        Ok(session)
    }

    pub async fn reauthenticate_passkey_verified(
        &self,
        session: &str,
        credential_id: &str,
        expected_version: i64,
        updated_credential: Value,
        expected_config_revision: i64,
    ) -> Result<i64> {
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, false).await?;
        lock_identity(&mut tx).await?;
        check_config_revision(&mut tx, expected_config_revision).await?;
        update_verified_passkey(
            &mut tx,
            &user.id,
            credential_id,
            expected_version,
            &updated_credential,
        )
        .await?;
        let until = mark_recent_in(&mut tx, session).await?;
        tx.commit().await?;
        Ok(until)
    }

    pub async fn remove_passkey(&self, session: &str, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        lock_identity(&mut tx).await?;
        let changed = sqlx::query("DELETE FROM auth_passkeys WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(&user.id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if changed != 1 {
            return Err(Error::NotFound);
        }
        retain_signin_method(&mut tx, &user.id).await?;
        revoke_method_sessions(&mut tx, "passkey", id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn rename_passkey(
        &self,
        session: &str,
        id: &str,
        name: &str,
    ) -> Result<PasskeyCredential> {
        let name = checked_label(name, "passkey name", 100)?;
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        let changed =
            sqlx::query("UPDATE auth_passkeys SET name = $1 WHERE id = $2 AND user_id = $3")
                .bind(name)
                .bind(id)
                .bind(&user.id)
                .execute(&mut *tx)
                .await?
                .rows_affected();
        if changed != 1 {
            return Err(Error::NotFound);
        }
        let row = sqlx::query("SELECT * FROM auth_passkeys WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let result = passkey_from_row(&row)?;
        tx.commit().await?;
        Ok(result)
    }

    /// Trusted core configuration read. Decrypt only the individual field needed
    /// by an identity adapter; never expose this value in public capability APIs.
    pub async fn identity_setting(&self, key: &str) -> Result<Option<IdentitySetting>> {
        validate_setting_key(key)?;
        let value: Option<String> =
            sqlx::query_scalar("SELECT value FROM instance_state WHERE key = $1")
                .bind(format!("identity_config:{key}"))
                .fetch_optional(&self.pool)
                .await?;
        value
            .map(|v| serde_json::from_str(&v).map_err(Error::from))
            .transpose()
    }

    pub async fn set_identity_setting(
        &self,
        session: &str,
        key: &str,
        expected_revision: i64,
        value: Value,
    ) -> Result<IdentitySetting> {
        validate_setting_key(key)?;
        encode_payload(&value)?;
        let mut tx = self.pool.begin().await?;
        let (user, _) = session_in(&mut tx, session, true).await?;
        if !user.instance_admin {
            return Err(Error::Forbidden);
        }
        lock_identity(&mut tx).await?;
        let old = setting_in(&mut tx, key).await?;
        if old.as_ref().map(|s| s.revision).unwrap_or(0) != expected_revision {
            return Err(Error::Conflict);
        }
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_passkeys")
            .fetch_one(&mut *tx)
            .await?;
        if count != 0
            && ["public_origin", "rp_id"].iter().any(|field| {
                old.as_ref()
                    .map(|s| &s.value[*field])
                    .unwrap_or(&Value::Null)
                    != &value[*field]
            })
        {
            return Err(Error::Invalid(
                "public origin and RP ID cannot change while passkeys exist".into(),
            ));
        }
        let setting = IdentitySetting {
            revision: expected_revision
                .checked_add(1)
                .filter(|v| *v > 0)
                .ok_or(Error::Conflict)?,
            value,
        };
        sqlx::query("INSERT INTO instance_state (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = excluded.value")
            .bind(format!("identity_config:{key}")).bind(serde_json::to_string(&setting)?).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(setting)
    }

    pub async fn count_passkeys(&self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM auth_passkeys")
            .fetch_one(&self.pool)
            .await?)
    }
}

pub(crate) async fn lock_identity(tx: &mut Transaction<'_, Any>) -> Result<()> {
    let changed = sqlx::query("UPDATE counters SET value = value WHERE name = 'identity'")
        .execute(&mut **tx)
        .await?
        .rows_affected();
    if changed != 1 {
        return Err(Error::Invalid(
            "identity serialization marker is missing".into(),
        ));
    }
    Ok(())
}

async fn lock_user(tx: &mut Transaction<'_, Any>, id: &str) -> Result<User> {
    let changed = sqlx::query(
        "UPDATE users SET password_hash = password_hash WHERE id = $1 AND disabled = 0",
    )
    .bind(id)
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if changed != 1 {
        return Err(Error::Unauthorized);
    }
    let row = sqlx::query("SELECT id, username, instance_admin FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;
    user_from_row(&row)
}

async fn session_in(
    tx: &mut Transaction<'_, Any>,
    token: &str,
    recent: bool,
) -> Result<(User, i64)> {
    validate_token(token)?;
    let digest = token_hash(token);
    let user_id: String = sqlx::query_scalar(
        "SELECT user_id FROM sessions WHERE token_hash = $1 AND expires_at > $2",
    )
    .bind(&digest)
    .bind(now())
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(Error::Unauthorized)?;
    let user = lock_user(tx, &user_id).await?;
    let row = sqlx::query("SELECT s.expires_at, COALESCE(a.reauthenticated_at, s.created_at) AS reauthenticated_at FROM sessions s LEFT JOIN auth_session_security a ON a.token_hash = s.token_hash WHERE s.token_hash = $1 AND s.user_id = $2 AND s.expires_at > $3")
        .bind(digest).bind(&user_id).bind(now()).fetch_optional(&mut **tx).await?.ok_or(Error::Unauthorized)?;
    let until = row
        .try_get::<i64, _>("reauthenticated_at")?
        .saturating_add(RECENT_AUTH_SECONDS)
        .min(row.try_get("expires_at")?);
    if recent && until <= now() {
        return Err(Error::RecentAuthRequired);
    }
    Ok((user, until))
}

async fn mark_recent_in(tx: &mut Transaction<'_, Any>, session: &str) -> Result<i64> {
    let at = now();
    sqlx::query("INSERT INTO auth_session_security (token_hash, reauthenticated_at, auth_method, credential_id) VALUES ($1, $2, 'password', '') ON CONFLICT (token_hash) DO UPDATE SET reauthenticated_at = excluded.reauthenticated_at")
        .bind(token_hash(session)).bind(at).execute(&mut **tx).await?;
    let expires: i64 = sqlx::query_scalar("SELECT expires_at FROM sessions WHERE token_hash = $1")
        .bind(token_hash(session))
        .fetch_one(&mut **tx)
        .await?;
    Ok(at.saturating_add(RECENT_AUTH_SECONDS).min(expires))
}

async fn external_session(
    tx: &mut Transaction<'_, Any>,
    user: User,
    method: &str,
    credential: &str,
) -> Result<TokenSession> {
    let session = create_session_in(tx, user).await?;
    // Google SSO establishes identity but does not guarantee a freshly entered
    // account credential. Sensitive changes require local password/WebAuthn
    // verification even immediately after a successful Google login.
    sqlx::query("UPDATE auth_session_security SET auth_method = $1, credential_id = $2, reauthenticated_at = CASE WHEN $1 = 'google' THEN 0 ELSE reauthenticated_at END WHERE token_hash = $3")
        .bind(method).bind(credential).bind(token_hash(&session.token)).execute(&mut **tx).await?;
    Ok(session)
}

async fn revoke_method_sessions(
    tx: &mut Transaction<'_, Any>,
    method: &str,
    id: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM auth_challenges WHERE session_hash IN (SELECT token_hash FROM auth_session_security WHERE auth_method = $1 AND credential_id = $2)")
        .bind(method).bind(id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM sessions WHERE token_hash IN (SELECT token_hash FROM auth_session_security WHERE auth_method = $1 AND credential_id = $2)")
        .bind(method).bind(id).execute(&mut **tx).await?;
    Ok(())
}

async fn retain_signin_method(tx: &mut Transaction<'_, Any>, user: &str) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT (CASE WHEN password_hash <> '' THEN 1 ELSE 0 END) + (SELECT COUNT(*) FROM auth_google WHERE user_id = $1) + (SELECT COUNT(*) FROM auth_passkeys WHERE user_id = $1) AS methods FROM users WHERE id = $1")
        .bind(user).fetch_one(&mut **tx).await?;
    if count == 0 {
        return Err(Error::Invalid(
            "the account must retain a usable sign-in method".into(),
        ));
    }
    Ok(())
}

async fn update_verified_passkey(
    tx: &mut Transaction<'_, Any>,
    user: &str,
    credential_id: &str,
    expected_version: i64,
    credential: &Value,
) -> Result<()> {
    let version = next_credential_version(tx).await?;
    let changed = sqlx::query("UPDATE auth_passkeys SET credential = $1, version = $2, last_used_at = $3 WHERE user_id = $4 AND credential_id = $5 AND version = $6")
        .bind(encode_payload(credential)?).bind(version).bind(now()).bind(user).bind(credential_id).bind(expected_version).execute(&mut **tx).await?.rows_affected();
    if changed != 1 {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// A version is an opaque credential snapshot, not a per-row increment. Never
// reuse it after deletion/re-enrollment of the same authenticator credential ID:
// an assertion verified against the revoked record must not match its successor.
async fn next_credential_version(tx: &mut Transaction<'_, Any>) -> Result<i64> {
    sqlx::query_scalar("UPDATE counters SET value = value + 1 WHERE name = 'identity' AND value < 9223372036854775807 RETURNING value")
        .fetch_optional(&mut **tx).await?.ok_or(Error::Conflict)
}

async fn email_in(tx: &mut Transaction<'_, Any>, user: &str) -> Result<Option<VerifiedEmail>> {
    sqlx::query("SELECT email, verified_at, version FROM auth_emails WHERE user_id = $1")
        .bind(user)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| {
            Ok(VerifiedEmail {
                email: row.try_get("email")?,
                verified_at: row.try_get("verified_at")?,
                version: row.try_get("version")?,
            })
        })
        .transpose()
}

pub(crate) async fn ensure_username_available(
    tx: &mut Transaction<'_, Any>,
    username: &str,
) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_emails WHERE email = $1")
        .bind(username)
        .fetch_one(&mut **tx)
        .await?;
    if count != 0 {
        return Err(Error::Conflict);
    }
    Ok(())
}

async fn ensure_email_available(
    tx: &mut Transaction<'_, Any>,
    user: &str,
    email: &str,
) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT (SELECT COUNT(*) FROM auth_emails WHERE email = $1 AND user_id <> $2) + (SELECT COUNT(*) FROM users WHERE username = $1 AND id <> $2) AS occupied")
        .bind(email).bind(user).fetch_one(&mut **tx).await?;
    if count != 0 {
        return Err(Error::Conflict);
    }
    Ok(())
}

pub(crate) fn normalize_login(value: &str) -> Result<String> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty()
        || value.len() > 254
        || !value.is_ascii()
        || value
            .bytes()
            .any(|b| b.is_ascii_control() || b.is_ascii_whitespace())
    {
        return Err(Error::Unauthorized);
    }
    Ok(value)
}

fn normalize_email(value: &str) -> Result<String> {
    let email =
        normalize_login(value).map_err(|_| Error::Invalid("invalid email address".into()))?;
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| Error::Invalid("invalid email address".into()))?;
    if local.is_empty()
        || local.len() > 64
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
        || !local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b))
        || domain.is_empty()
        || !domain.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err(Error::Invalid("invalid email address".into()));
    }
    Ok(email)
}

fn encode_payload(value: &Value) -> Result<String> {
    let encoded = serde_json::to_string(value)?;
    if !value.is_object() || encoded.len() > 64 * 1024 {
        return Err(Error::Invalid(
            "identity state must be an object of at most 64 KiB".into(),
        ));
    }
    Ok(encoded)
}

fn validate_credential_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 4096
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(Error::Invalid(
            "credential ID must use bounded unpadded base64url".into(),
        ));
    }
    Ok(())
}

fn validate_token(token: &str) -> Result<()> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn validate_purpose(purpose: &str) -> Result<()> {
    checked_label(purpose, "challenge purpose", 64)?;
    if matches!(purpose, "email_change" | "password_reset") {
        return Err(Error::Invalid("use the dedicated email operation".into()));
    }
    Ok(())
}

async fn insert_challenge(
    tx: &mut Transaction<'_, Any>,
    purpose: &str,
    user: &str,
    session_hash: &str,
    payload: Value,
    ttl: i64,
) -> Result<AuthChallenge> {
    let encoded = encode_payload(&payload)?;
    sqlx::query("DELETE FROM auth_challenges WHERE expires_at <= $1")
        .bind(now())
        .execute(&mut **tx)
        .await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges")
        .fetch_one(&mut **tx)
        .await?;
    let owned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges WHERE user_id = $1")
        .bind(user)
        .fetch_one(&mut **tx)
        .await?;
    if total >= 4096 || (!user.is_empty() && owned >= 20) || (user.is_empty() && owned >= 1024) {
        return Err(Error::RateLimited);
    }
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let issued = now();
    let challenge = AuthChallenge {
        token,
        expires_at: issued + ttl,
    };
    sqlx::query("INSERT INTO auth_challenges (token_hash, purpose, user_id, session_hash, payload, expires_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(token_hash(&challenge.token)).bind(purpose).bind(user).bind(session_hash).bind(encoded).bind(challenge.expires_at).bind(issued).execute(&mut **tx).await?;
    Ok(challenge)
}

async fn challenge_in(
    tx: &mut Transaction<'_, Any>,
    token: &str,
    purpose: &str,
) -> Result<sqlx::any::AnyRow> {
    validate_token(token)?;
    sqlx::query(
        "SELECT * FROM auth_challenges WHERE token_hash = $1 AND purpose = $2 AND expires_at > $3",
    )
    .bind(token_hash(token))
    .bind(purpose)
    .bind(now())
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(Error::Unauthorized)
}

async fn delete_challenges_in(
    tx: &mut Transaction<'_, Any>,
    user: &str,
    purpose: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM auth_challenges WHERE user_id = $1 AND purpose = $2")
        .bind(user)
        .bind(purpose)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn delivery_limit(tx: &mut Transaction<'_, Any>, scope: &str) -> Result<()> {
    let at = now();
    let row = sqlx::query("SELECT * FROM auth_delivery_limits WHERE scope = $1")
        .bind(scope)
        .fetch_optional(&mut **tx)
        .await?;
    let (hour_start, hour_count, day_start, day_count) = if let Some(row) = row {
        let last: i64 = row.try_get("last_issued_at")?;
        let hour: i64 = row.try_get("hour_started_at")?;
        let day: i64 = row.try_get("day_started_at")?;
        let hc: i64 = if at - hour >= 3600 {
            0
        } else {
            row.try_get("hour_count")?
        };
        let dc: i64 = if at - day >= 86400 {
            0
        } else {
            row.try_get("day_count")?
        };
        if at.saturating_sub(last) < 60 || hc >= 3 || dc >= 5 {
            return Err(Error::RateLimited);
        }
        (
            if hc == 0 { at } else { hour },
            hc + 1,
            if dc == 0 { at } else { day },
            dc + 1,
        )
    } else {
        (at, 1, at, 1)
    };
    sqlx::query("INSERT INTO auth_delivery_limits (scope,last_issued_at,hour_started_at,hour_count,day_started_at,day_count) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (scope) DO UPDATE SET last_issued_at=excluded.last_issued_at,hour_started_at=excluded.hour_started_at,hour_count=excluded.hour_count,day_started_at=excluded.day_started_at,day_count=excluded.day_count")
        .bind(scope).bind(at).bind(hour_start).bind(hour_count).bind(day_start).bind(day_count).execute(&mut **tx).await?;
    Ok(())
}

fn google_from_row(row: &sqlx::any::AnyRow) -> Result<GoogleIdentity> {
    Ok(GoogleIdentity {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        subject: row.try_get("subject")?,
        email: row.try_get("email")?,
        created_at: row.try_get("created_at")?,
        last_used_at: row.try_get("last_used_at")?,
    })
}
fn passkey_from_row(row: &sqlx::any::AnyRow) -> Result<PasskeyCredential> {
    Ok(PasskeyCredential {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        credential_id: row.try_get("credential_id")?,
        name: row.try_get("name")?,
        credential: serde_json::from_str(&row.try_get::<String, _>("credential")?)?,
        version: row.try_get("version")?,
        created_at: row.try_get("created_at")?,
        last_used_at: row.try_get("last_used_at")?,
    })
}
fn validate_setting_key(key: &str) -> Result<()> {
    if key != "authentication" {
        return Err(Error::Invalid("unknown identity setting".into()));
    }
    Ok(())
}
async fn setting_in(tx: &mut Transaction<'_, Any>, key: &str) -> Result<Option<IdentitySetting>> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM instance_state WHERE key=$1")
        .bind(format!("identity_config:{key}"))
        .fetch_optional(&mut **tx)
        .await?;
    value
        .map(|v| serde_json::from_str(&v).map_err(Error::from))
        .transpose()
}
async fn check_config_revision(tx: &mut Transaction<'_, Any>, expected: i64) -> Result<()> {
    if setting_in(tx, "authentication")
        .await?
        .map(|s| s.revision)
        .unwrap_or(0)
        != expected
    {
        return Err(Error::Conflict);
    }
    Ok(())
}

/// Restore validates the same identity invariants as live writes. In particular,
/// malformed credentials cannot poison all future account reads, and a rewound
/// generation counter cannot make a revoked credential snapshot current again.
pub(crate) async fn validate_restored_identities(tx: &mut Transaction<'_, Any>) -> Result<()> {
    let collisions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_emails e JOIN users u ON u.username = e.email AND u.id <> e.user_id")
        .fetch_one(&mut **tx).await?;
    let counter: Option<i64> =
        sqlx::query_scalar("SELECT value FROM counters WHERE name = 'identity'")
            .fetch_optional(&mut **tx)
            .await?;
    let maximum: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM auth_passkeys")
        .fetch_one(&mut **tx)
        .await?;
    if collisions != 0 || counter.is_none_or(|value| value < maximum.max(0)) {
        return Err(Error::Invalid(
            "backup email aliases or identity generation are inconsistent".into(),
        ));
    }
    for table in ["auth_passkeys", "auth_google"] {
        let exceeded: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM (SELECT user_id FROM {table} GROUP BY user_id HAVING COUNT(*) > 20) excessive"))
            .fetch_one(&mut **tx).await?;
        if exceeded != 0 {
            return Err(Error::Invalid(
                "backup exceeds the per-user identity limit".into(),
            ));
        }
    }
    for row in sqlx::query("SELECT email, version FROM auth_emails")
        .fetch_all(&mut **tx)
        .await?
    {
        let email: String = row.try_get("email")?;
        if normalize_email(&email)? != email || row.try_get::<i64, _>("version")? < 1 {
            return Err(Error::Invalid(
                "invalid verified-email backup record".into(),
            ));
        }
    }
    for row in sqlx::query("SELECT * FROM auth_google")
        .fetch_all(&mut **tx)
        .await?
    {
        let record = google_from_row(&row)?;
        if checked_label(&record.subject, "Google subject", 255)? != record.subject {
            return Err(Error::Invalid(
                "invalid Google identity backup record".into(),
            ));
        }
        if let Some(email) = record.email
            && normalize_email(&email)? != email
        {
            return Err(Error::Invalid(
                "invalid Google identity backup email".into(),
            ));
        }
    }
    for row in sqlx::query("SELECT * FROM auth_passkeys")
        .fetch_all(&mut **tx)
        .await?
    {
        let record = passkey_from_row(&row)?;
        validate_credential_id(&record.credential_id)?;
        encode_payload(&record.credential)?;
        if record.version < 1 || checked_label(&record.name, "passkey name", 100)? != record.name {
            return Err(Error::Invalid("invalid passkey backup record".into()));
        }
    }
    if let Some(setting) = setting_in(tx, "authentication").await? {
        encode_payload(&setting.value)?;
        if setting.revision < 1 {
            return Err(Error::Invalid(
                "invalid identity configuration revision".into(),
            ));
        }
    }
    Ok(())
}
