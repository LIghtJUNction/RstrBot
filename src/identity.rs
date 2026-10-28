//! Account-bound WebAuthn ceremonies and recent authentication.
//!
//! Only the public browser challenge is returned. The paired verifier state is
//! stored in a single-use, expiring database record and bound to its purpose,
//! configuration revision and (for account changes) the original session.
mod google;

use crate::{
    api::{ApiResult, LoginGate, Payload, actor, session_response, token},
    identity_config::{self, IdentityConfig},
    state::{AppError, AppState},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::{get, patch, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rstrbot_store::{PasskeyCredential, Store};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use webauthn_rs::prelude::*;

const CEREMONY_SECONDS: i64 = 300;

/// These routes inherit the application's JSON limit, CSRF boundary and
/// authentication limiter. They do not expose a second unauthenticated API.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/security", get(security))
        .route("/auth/reauth", post(reauthenticate))
        .route("/auth/passkeys/register/options", post(register_options))
        .route("/auth/passkeys/register/finish", post(register_finish))
        .route("/auth/passkeys/login/options", post(login_options))
        .route("/auth/passkeys/login/finish", post(login_finish))
        .route("/auth/passkeys/reauth/options", post(reauth_options))
        .route("/auth/passkeys/reauth/finish", post(reauth_finish))
        .route("/auth/passkeys/{id}", patch(rename).delete(remove))
        .route("/auth/google/start", post(google::start))
        .route("/auth/google/callback", get(google::callback))
        .route("/auth/google/unlink", post(google::unlink))
}

fn ceremony_failed() -> AppError {
    AppError::new(
        StatusCode::UNAUTHORIZED,
        "passkey_verification_failed",
        "Passkey verification failed or expired. Start a new sign-in attempt",
    )
}

fn proof_error(mut error: AppError) -> AppError {
    // A failed second proof does not mean that the already authenticated
    // browser session expired. Keep credential errors separate from logout.
    if error.status == StatusCode::UNAUTHORIZED {
        error.status = StatusCode::FORBIDDEN;
        if error.code == "unauthorized" {
            error.code = "reauth_failed".into();
            error.message = "Authentication could not be confirmed. Try again".into();
        }
    }
    error
}

fn configured(config: &IdentityConfig) -> Result<Webauthn, AppError> {
    let origin = config.public_origin.as_deref().ok_or_else(|| {
        AppError::bad_request("Set the public HTTPS origin before enabling passkeys")
    })?;
    let origin = Url::parse(origin)
        .map_err(|_| AppError::bad_request("The configured public origin is invalid"))?;
    let host = origin
        .domain()
        .ok_or_else(|| AppError::bad_request("Passkeys require a domain name or localhost"))?;
    let rp_id = config.rp_id.as_deref().unwrap_or(host);
    // The library also permits an RP that is a parent domain. This product
    // deliberately binds one exact host, including after a restored setting.
    if rp_id != host
        || !(origin.scheme() == "https" || (origin.scheme() == "http" && host == "localhost"))
        || !origin.username().is_empty()
        || origin.password().is_some()
        || origin.path() != "/"
        || origin.query().is_some()
        || origin.fragment().is_some()
    {
        return Err(AppError::bad_request(
            "Passkeys require a secure origin and the exact relying-party hostname",
        ));
    }
    WebauthnBuilder::new(rp_id, &origin)
        .and_then(|builder| {
            builder
                .rp_name("RstrBot")
                .timeout(Duration::from_secs(CEREMONY_SECONDS as u64))
                .build()
        })
        .map_err(|_| AppError::bad_request("Passkey origin and relying-party ID do not match"))
}

pub(crate) fn passkey_available(config: &IdentityConfig) -> bool {
    configured(config).is_ok()
}

fn key_summary(key: &PasskeyCredential) -> Value {
    // These flags come from the verified authenticator data saved by the
    // WebAuthn library. They identify backup capability/state, not a vendor.
    json!({
        "id":key.id,"name":key.name,"created_at":key.created_at,
        "last_used_at":key.last_used_at,
        "backup_eligible":key.credential.pointer("/cred/backup_eligible"),
        "backed_up":key.credential.pointer("/cred/backup_state"),
    })
}

async fn security(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    let account = state.store().await.auth_account(&token(&headers)?).await?;
    Ok(Json(json!({
        "email":account.email,
        "google":account.google.iter().map(|identity| json!({
            "id":identity.id,"email":identity.email,"created_at":identity.created_at,
            "last_used_at":identity.last_used_at,
        })).collect::<Vec<_>>(),
        "passkeys":account.passkeys.iter().map(key_summary).collect::<Vec<_>>(),
        "recent_auth_until":account.recent_auth_until,"has_password":account.has_password,
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordReauth {
    password: String,
}

async fn reauthenticate(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<PasswordReauth>,
) -> ApiResult<Json<Value>> {
    let user = actor(&state, &headers).await?;
    let _permit = gate.enter(&user.username)?;
    let until = state
        .store()
        .await
        .reauthenticate_password(&token(&headers)?, &body.password)
        .await
        .map_err(|error| proof_error(error.into()))?;
    Ok(Json(json!({"recent_auth_until":until})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistrationInput {
    name: String,
    #[serde(default)]
    replace_id: Option<String>,
    #[serde(default)]
    authenticator: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RegistrationIntent {
    revision: i64,
    user_id: String,
    name: String,
    replace_id: Option<String>,
    state: PasskeyRegistration,
}

async fn register_options(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<RegistrationInput>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let db = state.store().await;
    let user = db.require_recent_session(&credential).await?;
    let _permit = gate.enter(&user.username)?;
    let name = body.name.trim();
    if name.is_empty() || name.len() > 100 || name.chars().any(char::is_control) {
        return Err(AppError::bad_request(
            "Passkey name must contain 1–100 bytes without control characters",
        ));
    }
    let account = db.auth_account(&credential).await?;
    if body
        .replace_id
        .as_ref()
        .is_some_and(|id| !account.passkeys.iter().any(|key| &key.id == id))
    {
        return Err(AppError::bad_request(
            "The passkey to replace is no longer available",
        ));
    }
    let keys = decode_keys(&account.passkeys)?;
    let exclude = Some(keys.iter().map(|key| key.cred_id().clone()).collect());
    let config = identity_config::load(&state).await?;
    let webauthn = configured(&config)?;
    let user_id =
        Uuid::parse_str(&user.id).map_err(|_| AppError::internal("The account ID is invalid"))?;
    let hint = body.authenticator.as_deref().unwrap_or("any");
    let (challenge, registration) = match hint {
        "any" | "security-key" | "platform" => {
            webauthn.start_passkey_registration(user_id, &user.username, &user.username, exclude)
        }
        "google-password-manager" => {
            // This is the library's documented Android/GMS compatibility path,
            // chosen explicitly by the user. It is not the default for keys.
            if !headers
                .get(header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("Android"))
            {
                return Err(AppError::bad_request(
                    "The Google Password Manager compatibility option is for Android browsers",
                ));
            }
            webauthn.start_google_passkey_in_google_password_manager_only_registration(
                user_id,
                &user.username,
                &user.username,
                exclude,
            )
        }
        _ => return Err(AppError::bad_request("Unknown authenticator preference")),
    }
    .map_err(|_| proof_error(ceremony_failed()))?;
    let mut public = serde_json::to_value(challenge)?;
    if hint != "google-password-manager" {
        // A browser preference may encourage discoverable/cloud credentials;
        // the verifier continues to require UV and accepts physical keys too.
        // It does not claim that a resident key was actually created.
        public["publicKey"]["authenticatorSelection"]["residentKey"] = json!("preferred");
        public["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
        if hint == "security-key" {
            public["publicKey"]["hints"] = json!(["security-key"]);
        } else if hint == "platform" {
            public["publicKey"]["hints"] = json!(["client-device"]);
        }
    }
    let intent = RegistrationIntent {
        revision: config.revision,
        user_id: user.id.clone(),
        name: name.into(),
        replace_id: body.replace_id,
        state: registration,
    };
    let issued = db
        .create_auth_challenge(
            "passkey.register",
            Some(&user.id),
            Some(&credential),
            serde_json::to_value(intent)?,
            CEREMONY_SECONDS,
        )
        .await?;
    public["challenge_id"] = json!(issued.token);
    public["expires_at"] = json!(issued.expires_at);
    Ok(Json(public))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistrationFinish {
    challenge_id: String,
    credential: RegisterPublicKeyCredential,
}

async fn register_finish(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<RegistrationFinish>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let db = state.store().await;
    let user = db.require_recent_session(&credential).await?;
    let _permit = gate.enter(&user.username)?;
    let intent: RegistrationIntent = serde_json::from_value(
        db.take_auth_challenge(&body.challenge_id, "passkey.register", Some(&credential))
            .await
            .map_err(|error| proof_error(error.into()))?,
    )?;
    let config = identity_config::load(&state).await?;
    if intent.revision != config.revision || intent.user_id != user.id {
        return Err(proof_error(ceremony_failed()));
    }
    let key = configured(&config)?
        .finish_passkey_registration(&body.credential, &intent.state)
        .map_err(|_| proof_error(ceremony_failed()))?;
    let id = URL_SAFE_NO_PAD.encode(key.cred_id());
    let key = db
        .add_passkey_verified(
            &credential,
            &id,
            &intent.name,
            serde_json::to_value(key)?,
            intent.replace_id.as_deref(),
            config.revision,
        )
        .await?;
    Ok(Json(json!({"passkey":key_summary(&key)})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginInput {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct AuthenticationIntent {
    revision: i64,
    user_id: String,
    versions: Vec<(String, i64)>,
    state: PasskeyAuthentication,
}

fn decode_keys(records: &[PasskeyCredential]) -> Result<Vec<Passkey>, AppError> {
    records
        .iter()
        .map(|record| {
            serde_json::from_value(record.credential.clone())
                .map_err(|_| AppError::internal("Stored passkey data is invalid"))
        })
        .collect()
}

async fn authentication_options(
    state: &AppState,
    db: &Store,
    username: &str,
    purpose: &str,
    session: Option<&str>,
) -> ApiResult<Json<Value>> {
    let records = db.passkeys_for_username(username).await?;
    let first = records.first().ok_or_else(ceremony_failed)?;
    let config = identity_config::load(state).await?;
    let (challenge, authentication) = configured(&config)?
        .start_passkey_authentication(&decode_keys(&records)?)
        .map_err(|_| ceremony_failed())?;
    let intent = AuthenticationIntent {
        revision: config.revision,
        user_id: first.user_id.clone(),
        versions: records
            .iter()
            .map(|record| (record.credential_id.clone(), record.version))
            .collect(),
        state: authentication,
    };
    let issued = db
        .create_auth_challenge(
            purpose,
            Some(&first.user_id),
            session,
            serde_json::to_value(intent)?,
            CEREMONY_SECONDS,
        )
        .await?;
    let mut public = serde_json::to_value(challenge)?;
    public["challenge_id"] = json!(issued.token);
    public["expires_at"] = json!(issued.expires_at);
    Ok(Json(public))
}

async fn login_options(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    Payload(body): Payload<LoginInput>,
) -> ApiResult<Json<Value>> {
    let _permit = gate.enter(&body.username)?;
    authentication_options(
        &state,
        &state.store().await,
        &body.username,
        "passkey.login",
        None,
    )
    .await
}

async fn reauth_options(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let user = actor(&state, &headers).await?;
    let _permit = gate.enter(&user.username)?;
    authentication_options(
        &state,
        &state.store().await,
        &user.username,
        "passkey.reauth",
        Some(&token(&headers)?),
    )
    .await
    .map_err(proof_error)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthenticationFinish {
    challenge_id: String,
    credential: PublicKeyCredential,
}

async fn verify_authentication(
    state: &AppState,
    body: AuthenticationFinish,
    purpose: &str,
    session: Option<&str>,
) -> Result<(PasskeyCredential, i64), AppError> {
    let db = state.store().await;
    let intent: AuthenticationIntent = serde_json::from_value(
        db.take_auth_challenge(&body.challenge_id, purpose, session)
            .await?,
    )?;
    let config = identity_config::load(state).await?;
    if intent.revision != config.revision {
        return Err(ceremony_failed());
    }
    let result = configured(&config)?
        .finish_passkey_authentication(&body.credential, &intent.state)
        .map_err(|_| ceremony_failed())?;
    if !result.user_verified() {
        return Err(ceremony_failed());
    }
    let id = URL_SAFE_NO_PAD.encode(result.cred_id());
    let mut record = db.passkey_by_credential_id(&id).await?;
    if record.user_id != intent.user_id
        || !intent
            .versions
            .iter()
            .any(|(allowed, version)| allowed == &id && *version == record.version)
    {
        return Err(ceremony_failed());
    }
    let mut key: Passkey = serde_json::from_value(record.credential)?;
    if key.update_credential(&result).is_none() {
        return Err(ceremony_failed());
    }
    record.credential = serde_json::to_value(key)?;
    Ok((record, config.revision))
}

async fn login_finish(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<AuthenticationFinish>,
) -> ApiResult<Response> {
    let _permit = gate.enter("passkey-finish")?;
    let (key, revision) = verify_authentication(&state, body, "passkey.login", None).await?;
    let session = state
        .store()
        .await
        .login_passkey_verified(&key.credential_id, key.version, key.credential, revision)
        .await?;
    session_response(&state, &headers, session).await
}

async fn reauth_finish(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<AuthenticationFinish>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let user = actor(&state, &headers).await?;
    let _permit = gate.enter(&user.username)?;
    let (key, revision) = verify_authentication(&state, body, "passkey.reauth", Some(&credential))
        .await
        .map_err(proof_error)?;
    let until = state
        .store()
        .await
        .reauthenticate_passkey_verified(
            &credential,
            &key.credential_id,
            key.version,
            key.credential,
            revision,
        )
        .await
        .map_err(|error| proof_error(error.into()))?;
    Ok(Json(json!({"recent_auth_until":until})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenameInput {
    name: String,
}

async fn rename(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Payload(body): Payload<RenameInput>,
) -> ApiResult<Json<Value>> {
    let key = state
        .store()
        .await
        .rename_passkey(&token(&headers)?, &id, &body.name)
        .await?;
    Ok(Json(json!({"passkey":key_summary(&key)})))
}

async fn remove(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .store()
        .await
        .remove_passkey(&token(&headers)?, &id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passkeys_require_an_exact_dns_relying_party_and_secure_origin() {
        for origin in ["https://bot.example.com", "http://localhost:3000"] {
            let config = IdentityConfig {
                public_origin: Some(origin.into()),
                ..Default::default()
            };
            assert!(passkey_available(&config));
        }
        for (origin, rp_id) in [
            ("https://bot.example.com", Some("example.com")),
            ("https://bot.example.com", Some("other.example.com")),
            ("http://bot.example.com", None),
            ("https://user:password@bot.example.com", None),
            ("https://bot.example.com/not-an-origin", None),
            ("https://bot.example.com/?query=invalid", None),
            ("https://bot.example.com/#fragment", None),
            ("http://127.0.0.1:3000", None),
        ] {
            let config = IdentityConfig {
                public_origin: Some(origin.into()),
                rp_id: rp_id.map(str::to_owned),
                ..Default::default()
            };
            assert!(!passkey_available(&config), "{origin} / {rp_id:?}");
        }
    }
}
