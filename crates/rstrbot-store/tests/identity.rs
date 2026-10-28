//! Durable identity invariants, run unchanged on SQLite and PostgreSQL.
//! These tests exercise the trusted storage boundary with synthetic verifier
//! records; the HTTP adapter separately tests Google/WebAuthn cryptography.
use rstrbot_store::{Backup, Error, Store, TokenSession};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{AnyPool, Row, any::AnyPoolOptions};

const PASSWORD: &str = "correct horse battery staple for identity tests";
const REPLACEMENT: &str = "a new correct password that is not the old one";

struct Fixture {
    store: Store,
    session: TokenSession,
    pool: AnyPool,
    url: String,
    _directory: Option<tempfile::TempDir>,
}

async fn database() -> (Store, Option<tempfile::TempDir>, String) {
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
        (Store::connect(&url).await.unwrap(), None, url)
    } else {
        let directory = tempfile::tempdir().unwrap();
        let url = Store::sqlite_file_url(&directory.path().join("identity.db")).unwrap();
        (Store::connect(&url).await.unwrap(), Some(directory), url)
    }
}

async fn fixture() -> Fixture {
    let (store, directory, url) = database().await;
    let session = store
        .bootstrap("alice", PASSWORD, "Identity tests")
        .await
        .unwrap();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    Fixture {
        store,
        session,
        pool,
        url,
        _directory: directory,
    }
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn credential(label: &str) -> Value {
    json!({"synthetic_store_fixture":label,"counter":0})
}

async fn verify_email(f: &Fixture, email: &str) {
    let challenge = f
        .store
        .begin_email_change(&f.session.token, email)
        .await
        .unwrap();
    let bound = f
        .store
        .consume_email_change(&f.session.token, &challenge.token)
        .await
        .unwrap();
    assert_eq!(bound.email, email);
}

async fn clear_delivery_limits(f: &Fixture) {
    // Move only the deterministic test clock state, never sleep through a window.
    sqlx::query("DELETE FROM auth_delivery_limits")
        .execute(&f.pool)
        .await
        .unwrap();
}

fn rechecksum(backup: &mut Backup) {
    backup.sha256 = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&backup.tables).unwrap())
    );
}

#[tokio::test]
async fn password_reset_capacity_is_generic_and_storage_errors_still_propagate() {
    let f = fixture().await;
    verify_email(&f, "bounded@example.test").await;
    for _ in 0..20 {
        f.store
            .create_auth_challenge("login", Some(&f.session.user.id), None, json!({}), 300)
            .await
            .unwrap();
    }
    assert!(
        f.store
            .begin_password_reset("bounded@example.test")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        f.store
            .begin_password_reset("unknown@example.test")
            .await
            .unwrap()
            .is_none()
    );
    let reset_limits: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_delivery_limits WHERE scope = $1")
            .bind(format!("password_reset:{}", f.session.user.id))
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(
        reset_limits, 0,
        "a capacity rejection rolls back the undelivered request"
    );
    sqlx::query("DROP TABLE auth_delivery_limits")
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.begin_password_reset("bounded@example.test").await,
        Err(Error::Database(_))
    ));
}

#[tokio::test]
async fn recent_auth_is_session_bound_and_does_not_extend_absolute_expiry() {
    let f = fixture().await;
    let second = f.store.login("alice", PASSWORD).await.unwrap();
    sqlx::query("UPDATE auth_session_security SET reauthenticated_at = 1")
        .execute(&f.pool)
        .await
        .unwrap();
    let expires: i64 = sqlx::query_scalar("SELECT expires_at FROM sessions WHERE token_hash = $1")
        .bind(digest(&f.session.token))
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.require_recent_session(&f.session.token).await,
        Err(Error::RecentAuthRequired)
    ));
    assert!(matches!(
        f.store
            .reauthenticate_password(&f.session.token, "incorrect")
            .await,
        Err(Error::Unauthorized)
    ));
    let until = f
        .store
        .reauthenticate_password(&f.session.token, PASSWORD)
        .await
        .unwrap();
    assert!(until <= expires);
    f.store
        .require_recent_session(&f.session.token)
        .await
        .unwrap();
    assert!(matches!(
        f.store.require_recent_session(&second.token).await,
        Err(Error::RecentAuthRequired)
    ));
    let unchanged: i64 =
        sqlx::query_scalar("SELECT expires_at FROM sessions WHERE token_hash = $1")
            .bind(digest(&f.session.token))
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(unchanged, expires);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(
        count, 2,
        "reauthentication must not create an extra session"
    );
    f.store.revoke(&f.session.token).await.unwrap();
    assert!(matches!(
        f.store
            .reauthenticate_password(&f.session.token, PASSWORD)
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store
            .link_google_verified(&f.session.token, "late-link", None, 0)
            .await,
        Err(Error::Unauthorized)
    ));
    sqlx::query("UPDATE users SET disabled = 1 WHERE id = $1")
        .bind(&second.user.id)
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.require_recent_session(&second.token).await,
        Err(Error::Unauthorized)
    ));
}

#[tokio::test]
async fn email_requires_same_live_session_single_use_and_collision_free_alias() {
    let f = fixture().await;
    let challenge = f
        .store
        .begin_email_change(&f.session.token, " Alice@Example.test ")
        .await
        .unwrap();
    assert!(matches!(
        f.store.login("alice@example.test", PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    assert!(
        f.store
            .begin_password_reset("alice@example.test")
            .await
            .unwrap()
            .is_none()
    );
    assert!(!format!("{challenge:?}").contains(&challenge.token));
    let stored: String = sqlx::query_scalar("SELECT token_hash FROM auth_challenges")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(stored, digest(&challenge.token));
    assert_ne!(stored, challenge.token);
    let second = f.store.login("alice", PASSWORD).await.unwrap();
    assert!(matches!(
        f.store
            .consume_email_change(&second.token, &challenge.token)
            .await,
        Err(Error::Unauthorized)
    ));
    let email = f
        .store
        .consume_email_change(&f.session.token, &challenge.token)
        .await
        .unwrap();
    assert_eq!(email.email, "alice@example.test");
    assert_eq!(email.version, 1);
    assert!(matches!(
        f.store
            .consume_email_change(&f.session.token, &challenge.token)
            .await,
        Err(Error::Unauthorized)
    ));
    assert_eq!(
        f.store
            .login("ALICE@EXAMPLE.TEST", PASSWORD)
            .await
            .unwrap()
            .user
            .id,
        f.session.user.id
    );
    assert!(matches!(
        f.store
            .create_user(&f.session.user, "alice@example.test", PASSWORD)
            .await,
        Err(Error::Conflict)
    ));
    let other = f
        .store
        .create_user(&f.session.user, "legacy@example.test", PASSWORD)
        .await
        .unwrap();
    let other_session = f.store.login(&other.username, PASSWORD).await.unwrap();
    assert!(matches!(
        f.store
            .begin_email_change(&other_session.token, "alice@example.test")
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store
            .begin_email_change(&f.session.token, "legacy@example.test")
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store
            .begin_email_change(&f.session.token, "new@example.test")
            .await,
        Err(Error::RateLimited)
    ));
    assert_eq!(
        f.store
            .auth_account(&f.session.token)
            .await
            .unwrap()
            .email
            .unwrap()
            .email,
        "alice@example.test"
    );
}

#[tokio::test]
async fn email_replacement_keeps_old_until_verified_and_invalidates_old_reset() {
    let f = fixture().await;
    verify_email(&f, "old@example.test").await;
    let old_reset = f
        .store
        .begin_password_reset("old@example.test")
        .await
        .unwrap()
        .unwrap();
    assert!(
        f.store
            .begin_password_reset("old@example.test")
            .await
            .unwrap()
            .is_none(),
        "throttles use the same response as unknown addresses"
    );
    assert!(
        f.store
            .begin_password_reset("missing@example.test")
            .await
            .unwrap()
            .is_none()
    );
    clear_delivery_limits(&f).await;
    let change = f
        .store
        .begin_email_change(&f.session.token, "new@example.test")
        .await
        .unwrap();
    f.store.login("old@example.test", PASSWORD).await.unwrap();
    assert!(matches!(
        f.store.login("new@example.test", PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    let bound = f
        .store
        .consume_email_change(&f.session.token, &change.token)
        .await
        .unwrap();
    assert_eq!(bound.version, 2);
    assert!(matches!(
        f.store
            .finish_password_reset(&old_reset.token, REPLACEMENT)
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store.login("old@example.test", PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    f.store.login("new@example.test", PASSWORD).await.unwrap();
    assert!(
        f.store
            .begin_password_reset("old@example.test")
            .await
            .unwrap()
            .is_none()
    );
    let reset = f
        .store
        .begin_password_reset("new@example.test")
        .await
        .unwrap()
        .unwrap();
    f.store
        .create_auth_challenge(
            "passkey_login",
            Some(&f.session.user.id),
            None,
            json!({"state":"pending"}),
            300,
        )
        .await
        .unwrap();
    f.store
        .finish_password_reset(&reset.token, REPLACEMENT)
        .await
        .unwrap();
    assert!(matches!(
        f.store.authenticate(&f.session.token).await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store.login("alice", PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    f.store
        .login("new@example.test", REPLACEMENT)
        .await
        .unwrap();
    assert!(matches!(
        f.store.finish_password_reset(&reset.token, PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges WHERE user_id = $1")
            .bind(&f.session.user.id)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn password_change_logout_and_expiry_cannot_be_undone_by_pending_email_tokens() {
    let f = fixture().await;
    verify_email(&f, "verified@example.test").await;
    let reset = f
        .store
        .begin_password_reset("verified@example.test")
        .await
        .unwrap()
        .unwrap();
    f.store
        .change_password(&f.session.user, PASSWORD, REPLACEMENT)
        .await
        .unwrap();
    assert!(matches!(
        f.store.finish_password_reset(&reset.token, PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    let fresh = f.store.login("alice", REPLACEMENT).await.unwrap();
    clear_delivery_limits(&f).await;
    let change = f
        .store
        .begin_email_change(&fresh.token, "changed@example.test")
        .await
        .unwrap();
    f.store.revoke(&fresh.token).await.unwrap();
    assert!(matches!(
        f.store
            .consume_email_change(&fresh.token, &change.token)
            .await,
        Err(Error::Unauthorized)
    ));
    let fresh = f.store.login("alice", REPLACEMENT).await.unwrap();
    assert!(
        matches!(
            f.store
                .consume_email_change(&fresh.token, &change.token)
                .await,
            Err(Error::Unauthorized)
        ),
        "a different session cannot inherit an old mailbox proof"
    );
    clear_delivery_limits(&f).await;
    let reset = f
        .store
        .begin_password_reset("verified@example.test")
        .await
        .unwrap()
        .unwrap();
    sqlx::query("UPDATE auth_challenges SET expires_at = 1 WHERE token_hash = $1")
        .bind(digest(&reset.token))
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.finish_password_reset(&reset.token, PASSWORD).await,
        Err(Error::Unauthorized)
    ));
}

#[tokio::test]
async fn durable_challenges_are_purpose_session_bound_expiring_and_once_only() {
    let f = fixture().await;
    let challenge = f
        .store
        .create_auth_challenge(
            "register",
            Some(&f.session.user.id),
            Some(&f.session.token),
            json!({"verifier":"server state"}),
            300,
        )
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .take_auth_challenge(&challenge.token, "login", Some(&f.session.token))
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store
            .take_auth_challenge(&challenge.token, "register", None)
            .await,
        Err(Error::Unauthorized)
    ));
    let second = f.store.login("alice", PASSWORD).await.unwrap();
    assert!(matches!(
        f.store
            .take_auth_challenge(&challenge.token, "register", Some(&second.token))
            .await,
        Err(Error::Unauthorized)
    ));
    let (first, second) = tokio::join!(
        f.store
            .take_auth_challenge(&challenge.token, "register", Some(&f.session.token)),
        f.store
            .take_auth_challenge(&challenge.token, "register", Some(&f.session.token))
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(matches!(first.or(second), Ok(value) if value["verifier"] == "server state"));
    let expires = f
        .store
        .create_auth_challenge("login", None, None, json!({}), 1)
        .await
        .unwrap();
    sqlx::query("UPDATE auth_challenges SET expires_at = 1 WHERE token_hash = $1")
        .bind(digest(&expires.token))
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .take_auth_challenge(&expires.token, "login", None)
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store
            .create_auth_challenge("password_reset", None, None, json!({}), 300)
            .await,
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        f.store
            .create_auth_challenge("login", None, None, json!({}), 601)
            .await,
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        f.store
            .create_auth_challenge("login", None, None, json!({"huge":"x".repeat(65_536)}), 300)
            .await,
        Err(Error::Invalid(_))
    ));
    for _ in 0..20 {
        f.store
            .create_auth_challenge("bounded", Some(&f.session.user.id), None, json!({}), 300)
            .await
            .unwrap();
    }
    assert!(matches!(
        f.store
            .create_auth_challenge("bounded", Some(&f.session.user.id), None, json!({}), 300)
            .await,
        Err(Error::RateLimited)
    ));
    f.store.close().await;
    let reopened = Store::connect(&f.url).await.unwrap();
    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges WHERE purpose = 'bounded'")
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(
        pending, 20,
        "restart retains pending server-side state without making it client-owned"
    );
    reopened.close().await;
}

#[tokio::test]
async fn google_subject_links_are_unique_and_profile_email_never_auto_links() {
    let f = fixture().await;
    let other = f
        .store
        .create_user(&f.session.user, "bob", PASSWORD)
        .await
        .unwrap();
    let bob = f.store.login(&other.username, PASSWORD).await.unwrap();
    let linked = f
        .store
        .link_google_verified(
            &f.session.token,
            "stable-sub",
            Some("profile@example.test"),
            0,
        )
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .link_google_verified(&bob.token, "stable-sub", None, 0)
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store.login_google_verified("unknown-sub", 0).await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store.login("profile@example.test", PASSWORD).await,
        Err(Error::Unauthorized)
    ));
    assert!(
        f.store
            .begin_password_reset("profile@example.test")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        f.store
            .auth_account(&f.session.token)
            .await
            .unwrap()
            .email
            .is_none()
    );
    let google_session = f
        .store
        .login_google_verified("stable-sub", 0)
        .await
        .unwrap();
    assert_eq!(google_session.user.id, f.session.user.id);
    f.store.authenticate(&google_session.token).await.unwrap();
    assert!(matches!(
        f.store.require_recent_session(&google_session.token).await,
        Err(Error::RecentAuthRequired)
    ));
    assert!(matches!(
        f.store
            .remove_google(&google_session.token, &linked.id)
            .await,
        Err(Error::RecentAuthRequired)
    ));
    f.store
        .reauthenticate_password(&google_session.token, PASSWORD)
        .await
        .unwrap();
    f.store
        .require_recent_session(&google_session.token)
        .await
        .unwrap();
    assert!(matches!(
        f.store.remove_google(&bob.token, &linked.id).await,
        Err(Error::NotFound)
    ));
    f.store
        .remove_google(&f.session.token, &linked.id)
        .await
        .unwrap();
    assert!(matches!(
        f.store.authenticate(&google_session.token).await,
        Err(Error::Unauthorized)
    ));
    f.store.authenticate(&f.session.token).await.unwrap();
    assert!(matches!(
        f.store.login_google_verified("stable-sub", 0).await,
        Err(Error::Unauthorized)
    ));
}

#[tokio::test]
async fn multiple_passkeys_keep_independent_state_and_revoked_versions_never_recur() {
    let f = fixture().await;
    let hardware = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "hardware_credential",
            "USB security key",
            credential("hardware"),
            None,
            0,
        )
        .await
        .unwrap();
    let cloud = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "synced_credential",
            "Cloud key",
            credential("cloud"),
            None,
            0,
        )
        .await
        .unwrap();
    assert_ne!(hardware.version, cloud.version);
    let by_name = f.store.passkeys_for_username("alice").await.unwrap();
    assert_eq!(by_name.len(), 2);
    assert!(
        f.store
            .passkeys_for_username("unknown")
            .await
            .unwrap()
            .is_empty()
    );
    let hardware_session = f
        .store
        .login_passkey_verified(
            &hardware.credential_id,
            hardware.version,
            json!({"counter":1}),
            0,
        )
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .login_passkey_verified(
                &hardware.credential_id,
                hardware.version,
                json!({"counter":1}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    let advanced = f
        .store
        .passkey_by_credential_id(&hardware.credential_id)
        .await
        .unwrap();
    assert!(advanced.version > cloud.version);
    let renamed = f
        .store
        .rename_passkey(&f.session.token, &hardware.id, "Primary USB key")
        .await
        .unwrap();
    assert_eq!(
        renamed.version, advanced.version,
        "renaming must preserve the verified counter state"
    );
    assert_eq!(renamed.credential, json!({"counter":1}));
    let cloud_session = f
        .store
        .login_passkey_verified(&cloud.credential_id, cloud.version, json!({"counter":1}), 0)
        .await
        .unwrap();
    f.store
        .remove_passkey(&f.session.token, &hardware.id)
        .await
        .unwrap();
    assert!(matches!(
        f.store.authenticate(&hardware_session.token).await,
        Err(Error::Unauthorized)
    ));
    f.store.authenticate(&cloud_session.token).await.unwrap();
    assert!(matches!(
        f.store
            .login_passkey_verified(
                &hardware.credential_id,
                advanced.version,
                json!({"counter":2}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    let reenrolled = f
        .store
        .add_passkey_verified(
            &f.session.token,
            &hardware.credential_id,
            "Re-enrolled key",
            credential("new state"),
            None,
            0,
        )
        .await
        .unwrap();
    assert!(reenrolled.version > advanced.version);
    assert_ne!(reenrolled.id, hardware.id);
    assert!(matches!(
        f.store
            .login_passkey_verified(
                &hardware.credential_id,
                hardware.version,
                json!({"counter":9}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store
            .login_passkey_verified(
                &hardware.credential_id,
                advanced.version,
                json!({"counter":9}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    f.store
        .login_passkey_verified(
            &reenrolled.credential_id,
            reenrolled.version,
            json!({"counter":1}),
            0,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn passkey_replacement_is_atomic_owned_and_revokes_old_method_sessions() {
    let f = fixture().await;
    let old = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "old_credential",
            "Old key",
            credential("old"),
            None,
            0,
        )
        .await
        .unwrap();
    let spare = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "spare_credential",
            "Spare key",
            credential("spare"),
            None,
            0,
        )
        .await
        .unwrap();
    let old_session = f
        .store
        .login_passkey_verified(&old.credential_id, old.version, json!({"counter":1}), 0)
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .add_passkey_verified(
                &f.session.token,
                &spare.credential_id,
                "Duplicate replacement",
                credential("new"),
                Some(&old.id),
                0
            )
            .await,
        Err(Error::Conflict)
    ));
    f.store
        .passkey_by_credential_id(&old.credential_id)
        .await
        .unwrap();
    f.store.authenticate(&old_session.token).await.unwrap();
    let bob = f
        .store
        .create_user(&f.session.user, "bob", PASSWORD)
        .await
        .unwrap();
    let bob = f.store.login(&bob.username, PASSWORD).await.unwrap();
    assert!(matches!(
        f.store
            .add_passkey_verified(
                &bob.token,
                "foreign_replacement",
                "Wrong account",
                credential("new"),
                Some(&old.id),
                0
            )
            .await,
        Err(Error::NotFound)
    ));
    let new = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "new_credential",
            "Replacement key",
            credential("new"),
            Some(&old.id),
            0,
        )
        .await
        .unwrap();
    assert_eq!(f.store.count_passkeys().await.unwrap(), 2);
    assert!(matches!(
        f.store.passkey_by_credential_id(&old.credential_id).await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        f.store.authenticate(&old_session.token).await,
        Err(Error::Unauthorized)
    ));
    f.store.authenticate(&f.session.token).await.unwrap();
    f.store
        .login_passkey_verified(&new.credential_id, new.version, json!({"counter":1}), 0)
        .await
        .unwrap();
}

#[tokio::test]
async fn passkey_reauth_checks_owner_version_enabled_account_and_original_expiry() {
    let f = fixture().await;
    let key = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "reauth_credential",
            "Reauth key",
            credential("reauth"),
            None,
            0,
        )
        .await
        .unwrap();
    let bob = f
        .store
        .create_user(&f.session.user, "bob", PASSWORD)
        .await
        .unwrap();
    let bob = f.store.login(&bob.username, PASSWORD).await.unwrap();
    sqlx::query("UPDATE auth_session_security SET reauthenticated_at = 1 WHERE token_hash = $1")
        .bind(digest(&f.session.token))
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.remove_passkey(&f.session.token, &key.id).await,
        Err(Error::RecentAuthRequired)
    ));
    assert!(matches!(
        f.store
            .reauthenticate_passkey_verified(
                &bob.token,
                &key.credential_id,
                key.version,
                json!({"counter":1}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    let expires: i64 = sqlx::query_scalar("SELECT expires_at FROM sessions WHERE token_hash = $1")
        .bind(digest(&f.session.token))
        .fetch_one(&f.pool)
        .await
        .unwrap();
    let until = f
        .store
        .reauthenticate_passkey_verified(
            &f.session.token,
            &key.credential_id,
            key.version,
            json!({"counter":1}),
            0,
        )
        .await
        .unwrap();
    assert!(until <= expires);
    f.store
        .require_recent_session(&f.session.token)
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .reauthenticate_passkey_verified(
                &f.session.token,
                &key.credential_id,
                key.version,
                json!({"counter":2}),
                0
            )
            .await,
        Err(Error::Unauthorized)
    ));
    let current = f
        .store
        .passkey_by_credential_id(&key.credential_id)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET disabled = 1 WHERE id = $1")
        .bind(&f.session.user.id)
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .login_passkey_verified(&key.credential_id, current.version, json!({"counter":2}), 0)
            .await,
        Err(Error::Unauthorized)
    ));
}

#[tokio::test]
async fn removing_the_last_signin_method_rolls_back_but_atomic_replacement_is_allowed() {
    let f = fixture().await;
    let key = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "last_credential",
            "Only key",
            credential("only"),
            None,
            0,
        )
        .await
        .unwrap();
    // Future password-less accounts must obey the invariant too. There is no
    // public API to disable passwords in this release, so seed that state here.
    sqlx::query("UPDATE users SET password_hash = '' WHERE id = $1")
        .bind(&f.session.user.id)
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.store.remove_passkey(&f.session.token, &key.id).await,
        Err(Error::Invalid(_))
    ));
    f.store
        .passkey_by_credential_id(&key.credential_id)
        .await
        .unwrap();
    let replacement = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "replacement_only",
            "New only key",
            credential("replacement"),
            Some(&key.id),
            0,
        )
        .await
        .unwrap();
    assert_eq!(f.store.count_passkeys().await.unwrap(), 1);
    let google = f
        .store
        .link_google_verified(&f.session.token, "last-google", None, 0)
        .await
        .unwrap();
    f.store
        .remove_passkey(&f.session.token, &replacement.id)
        .await
        .unwrap();
    assert!(matches!(
        f.store.remove_google(&f.session.token, &google.id).await,
        Err(Error::Invalid(_))
    ));
    f.store
        .login_google_verified("last-google", 0)
        .await
        .unwrap();
}

#[tokio::test]
async fn identity_configuration_cas_serializes_with_external_credential_commits() {
    let f = fixture().await;
    let one = json!({"public_origin":"https://example.test","rp_id":"example.test","google":{"enabled":true}});
    let setting = f
        .store
        .set_identity_setting(&f.session.token, "authentication", 0, one.clone())
        .await
        .unwrap();
    assert_eq!(setting.revision, 1);
    assert!(matches!(
        f.store
            .set_identity_setting(&f.session.token, "authentication", 0, one.clone())
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store
            .link_google_verified(&f.session.token, "stale-google", None, 0)
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store
            .add_passkey_verified(
                &f.session.token,
                "stale_credential",
                "Stale",
                credential("stale"),
                None,
                0
            )
            .await,
        Err(Error::Conflict)
    ));
    let key = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "configured_credential",
            "Configured",
            credential("config"),
            None,
            1,
        )
        .await
        .unwrap();
    f.store
        .link_google_verified(&f.session.token, "configured-google", None, 1)
        .await
        .unwrap();
    let changed_origin = json!({"public_origin":"https://other.test","rp_id":"other.test"});
    assert!(matches!(
        f.store
            .set_identity_setting(
                &f.session.token,
                "authentication",
                1,
                changed_origin.clone()
            )
            .await,
        Err(Error::Invalid(_))
    ));
    let mut two = one.clone();
    two["google"]["enabled"] = json!(false);
    f.store
        .set_identity_setting(&f.session.token, "authentication", 1, two)
        .await
        .unwrap();
    assert!(matches!(
        f.store
            .login_passkey_verified(&key.credential_id, key.version, json!({"counter":1}), 1)
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store
            .reauthenticate_passkey_verified(
                &f.session.token,
                &key.credential_id,
                key.version,
                json!({"counter":1}),
                1
            )
            .await,
        Err(Error::Conflict)
    ));
    assert!(matches!(
        f.store.login_google_verified("configured-google", 1).await,
        Err(Error::Conflict)
    ));
    f.store
        .remove_passkey(&f.session.token, &key.id)
        .await
        .unwrap();
    f.store
        .set_identity_setting(&f.session.token, "authentication", 2, changed_origin)
        .await
        .unwrap();
    let bob = f
        .store
        .create_user(&f.session.user, "bob", PASSWORD)
        .await
        .unwrap();
    let bob = f.store.login(&bob.username, PASSWORD).await.unwrap();
    assert!(matches!(
        f.store
            .set_identity_setting(&bob.token, "authentication", 3, one)
            .await,
        Err(Error::Forbidden)
    ));
    assert_eq!(
        f.store
            .identity_setting("authentication")
            .await
            .unwrap()
            .unwrap()
            .revision,
        3
    );
}

#[tokio::test]
async fn schema_one_upgrades_without_losing_password_sessions_or_users() {
    let f = fixture().await;
    for table in [
        "auth_session_security",
        "auth_challenges",
        "auth_delivery_limits",
        "auth_emails",
        "auth_google",
        "auth_passkeys",
    ] {
        sqlx::query(&format!("DROP TABLE {table}"))
            .execute(&f.pool)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM counters WHERE name = 'identity'")
        .execute(&f.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE instance_state SET value = '1' WHERE key = 'schema_version'")
        .execute(&f.pool)
        .await
        .unwrap();
    f.store.close().await;
    assert!(Store::probe(&f.url).await.unwrap().rstrbot_schema);
    let upgraded = Store::connect(&f.url).await.unwrap();
    assert_eq!(
        upgraded.authenticate(&f.session.token).await.unwrap().id,
        f.session.user.id
    );
    let summary = upgraded.auth_account(&f.session.token).await.unwrap();
    assert!(
        summary.has_password
            && summary.email.is_none()
            && summary.google.is_empty()
            && summary.passkeys.is_empty()
    );
    upgraded
        .reauthenticate_password(&f.session.token, PASSWORD)
        .await
        .unwrap();
    let row = sqlx::query("SELECT value FROM instance_state WHERE key = 'schema_version'")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(row.try_get::<String, _>("value").unwrap(), "2");
    upgraded.login("alice", PASSWORD).await.unwrap();
    assert!(Store::probe(&f.url).await.unwrap().initialized);
}

#[tokio::test]
async fn backup_two_preserves_bindings_and_config_but_discards_all_authentication_sessions() {
    let f = fixture().await;
    verify_email(&f, "backup@example.test").await;
    let setting = json!({"public_origin":"https://example.test","rp_id":"example.test","smtp":{"password_enc":"encrypted-secret"}});
    f.store
        .set_identity_setting(&f.session.token, "authentication", 0, setting.clone())
        .await
        .unwrap();
    let key = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "backup_credential",
            "Saved key",
            credential("backup"),
            None,
            1,
        )
        .await
        .unwrap();
    f.store
        .link_google_verified(&f.session.token, "backup-google", None, 1)
        .await
        .unwrap();
    let challenge = f
        .store
        .create_auth_challenge(
            "login",
            None,
            None,
            json!({"state":"secret verifier state"}),
            300,
        )
        .await
        .unwrap();
    let reset = f
        .store
        .begin_password_reset("backup@example.test")
        .await
        .unwrap()
        .unwrap();
    let backup = f.store.export_backup(&f.session.user).await.unwrap();
    assert_eq!(backup.format, "rstrbot-db-v2");
    assert_eq!(backup.schema_version, 2);
    for excluded in [
        "sessions",
        "auth_session_security",
        "auth_challenges",
        "auth_delivery_limits",
    ] {
        assert!(!backup.tables.contains_key(excluded));
    }
    let serialized = serde_json::to_string(&backup).unwrap();
    assert!(
        !serialized.contains(&f.session.token)
            && !serialized.contains(&challenge.token)
            && !serialized.contains(&reset.token)
    );
    assert!(!serialized.contains("secret verifier state"));
    let (restored, _directory, url) = database().await;
    let prior = restored
        .create_auth_challenge("login", None, None, json!({}), 300)
        .await
        .unwrap();
    restored.import_backup(&backup).await.unwrap();
    assert!(restored.recovery_mode().await.unwrap());
    assert!(matches!(
        restored.authenticate(&f.session.token).await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        restored
            .take_auth_challenge(&prior.token, "login", None)
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        restored
            .take_auth_challenge(&challenge.token, "login", None)
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        restored
            .finish_password_reset(&reset.token, REPLACEMENT)
            .await,
        Err(Error::Unauthorized)
    ));
    let login = restored
        .login("backup@example.test", PASSWORD)
        .await
        .unwrap();
    let summary = restored.auth_account(&login.token).await.unwrap();
    assert_eq!(summary.email.unwrap().email, "backup@example.test");
    assert_eq!(summary.google.len(), 1);
    assert_eq!(summary.passkeys.len(), 1);
    assert_eq!(summary.passkeys[0].credential, key.credential);
    assert_eq!(
        restored
            .identity_setting("authentication")
            .await
            .unwrap()
            .unwrap()
            .value,
        setting
    );
    restored
        .login_google_verified("backup-google", 1)
        .await
        .unwrap();
    restored
        .login_passkey_verified(&key.credential_id, key.version, json!({"counter":1}), 1)
        .await
        .unwrap();
    restored.close().await;
    Store::connect(&url).await.unwrap().close().await;
}

#[tokio::test]
async fn legacy_backups_import_without_inventing_identity_bindings() {
    let f = fixture().await;
    let mut backup = f.store.export_backup(&f.session.user).await.unwrap();
    backup.format = "rstrbot-db-v1".into();
    backup.schema_version = 1;
    for table in ["auth_emails", "auth_google", "auth_passkeys"] {
        backup.tables.remove(table);
    }
    backup
        .tables
        .get_mut("counters")
        .unwrap()
        .retain(|row| row["name"] != "identity");
    for row in backup.tables.get_mut("instance_state").unwrap() {
        if row["key"] == "schema_version" {
            row["value"] = json!("1");
        }
    }
    rechecksum(&mut backup);
    let (restored, _directory, url) = database().await;
    restored.import_backup(&backup).await.unwrap();
    let session = restored.login("alice", PASSWORD).await.unwrap();
    let summary = restored.auth_account(&session.token).await.unwrap();
    assert!(summary.email.is_none() && summary.google.is_empty() && summary.passkeys.is_empty());
    restored
        .add_passkey_verified(
            &session.token,
            "first_new_key",
            "New after migration",
            credential("first"),
            None,
            0,
        )
        .await
        .unwrap();
    restored.close().await;
    Store::connect(&url).await.unwrap().close().await;
}

#[tokio::test]
async fn backup_identity_invariant_failures_leave_empty_target_unchanged() {
    let f = fixture().await;
    let key = f
        .store
        .add_passkey_verified(
            &f.session.token,
            "checked_credential",
            "Checked",
            credential("checked"),
            None,
            0,
        )
        .await
        .unwrap();
    let backup = f.store.export_backup(&f.session.user).await.unwrap();
    let mut rewound = backup.clone();
    for row in rewound.tables.get_mut("counters").unwrap() {
        if row["name"] == "identity" {
            row["value"] = json!(key.version - 1);
        }
    }
    rechecksum(&mut rewound);
    let (empty, _directory, _) = database().await;
    assert!(matches!(
        empty.import_backup(&rewound).await,
        Err(Error::Invalid(_))
    ));
    assert!(!empty.initialized().await.unwrap());
    let mut malformed = backup.clone();
    malformed.tables.get_mut("auth_passkeys").unwrap()[0]["credential"] = json!("[]");
    rechecksum(&mut malformed);
    assert!(matches!(
        empty.import_backup(&malformed).await,
        Err(Error::Invalid(_))
    ));
    assert!(!empty.initialized().await.unwrap());
    let mut collision = backup.clone();
    let bob = f
        .store
        .create_user(&f.session.user, "occupied@example.test", PASSWORD)
        .await
        .unwrap();
    collision.tables.get_mut("users").unwrap().push(json!({"id":bob.id,"username":bob.username,"password_hash":"fixture-only","instance_admin":0,"disabled":0,"created_at":1}));
    collision.tables.get_mut("auth_emails").unwrap().push(json!({"user_id":f.session.user.id,"email":"occupied@example.test","verified_at":1,"version":1}));
    rechecksum(&mut collision);
    assert!(matches!(
        empty.import_backup(&collision).await,
        Err(Error::Invalid(_))
    ));
    assert!(!empty.initialized().await.unwrap());
    empty.import_backup(&backup).await.unwrap();
}
