//! Administrator-managed identity providers. Secrets are encrypted before the
//! configuration enters the database and are never returned to a browser.

use crate::{
    api::{ApiResult, LoginGate, Payload, actor, token},
    state::{AppError, AppState},
};
use axum::{Extension, Json, Router, extract::State, http::HeaderMap, routing::get};
use lettre::Address;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{net::IpAddr, sync::Arc};
use url::Url;

const SETTING: &str = "authentication";

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GoogleConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret_cipher: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SmtpConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub security: String,
    pub username: String,
    pub password_cipher: Option<String>,
    pub from_email: String,
    pub from_name: String,
}
impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: 587,
            security: "starttls".into(),
            username: String::new(),
            password_cipher: None,
            from_email: String::new(),
            from_name: "RstrBot".into(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IdentityConfig {
    #[serde(skip)]
    pub revision: i64,
    pub public_origin: Option<String>,
    pub rp_id: Option<String>,
    pub google: GoogleConfig,
    pub smtp: SmtpConfig,
}

pub async fn load(state: &AppState) -> ApiResult<IdentityConfig> {
    let Some(setting) = state.store().await.identity_setting(SETTING).await? else {
        return Ok(IdentityConfig::default());
    };
    let mut config: IdentityConfig = serde_json::from_value(setting.value)
        .map_err(|_| AppError::internal("Authentication configuration could not be read"))?;
    config.revision = setting.revision;
    Ok(config)
}

pub(crate) async fn cookie_is_secure(state: &AppState) -> ApiResult<bool> {
    Ok(state.config.read().await.secure_cookie
        || load(state)
            .await?
            .public_origin
            .is_some_and(|origin| origin.starts_with("https://")))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/options", get(options))
        .route("/admin/auth-settings", get(settings).put(save_settings))
}

async fn options(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    let config = load(&state).await?;
    Ok(Json(json!({
        "google_enabled": config.google.enabled,
        "passkey_enabled": crate::identity::passkey_available(&config),
        "email_enabled": config.smtp.enabled,
        "public_origin": config.public_origin,
        "rp_id": config.rp_id,
        "username_required": true
    })))
}

pub(crate) fn redacted(config: &IdentityConfig, passkey_count: i64) -> Value {
    json!({
        "revision": config.revision,
        "public_origin": config.public_origin,
        "rp_id": config.rp_id,
        "google": {
            "enabled": config.google.enabled,
            "client_id": config.google.client_id,
            "has_client_secret": config.google.client_secret_cipher.is_some(),
            "redirect_uri": config.public_origin.as_ref().map(|o| format!("{o}/api/v1/auth/google/callback"))
        },
        "smtp": {
            "enabled": config.smtp.enabled,
            "host": config.smtp.host,
            "port": config.smtp.port,
            "security": config.smtp.security,
            "username": config.smtp.username,
            "has_password": config.smtp.password_cipher.is_some(),
            "from_email": config.smtp.from_email,
            "from_name": config.smtp.from_name
        },
        "passkey_count": passkey_count
    })
}

async fn settings(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    if !actor(&state, &headers).await?.instance_admin {
        return Err(AppError::forbidden());
    }
    let config = load(&state).await?;
    Ok(Json(redacted(
        &config,
        state.store().await.count_passkeys().await?,
    )))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoogleInput {
    enabled: bool,
    client_id: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    clear_client_secret: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmtpInput {
    enabled: bool,
    host: String,
    port: u16,
    security: String,
    username: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    clear_password: bool,
    from_email: String,
    from_name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsInput {
    revision: i64,
    public_origin: Option<String>,
    rp_id: Option<String>,
    google: GoogleInput,
    smtp: SmtpInput,
}

async fn save_settings(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(input): Payload<SettingsInput>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let user = state
        .store()
        .await
        .require_recent_session(&credential)
        .await?;
    if !user.instance_admin {
        return Err(AppError::forbidden());
    }
    let _permit = gate.enter(&format!("identity-settings:{}", user.id))?;
    let previous = load(&state).await?;
    if input.revision != previous.revision {
        return Err(AppError::conflict(
            "Authentication settings changed; reload before saving",
        ));
    }
    let config = validated(&state, input, &previous)?;
    // CAS and the current administrator/session check are repeated by the
    // store under its identity mutation lock. No partial provider save.
    let setting = state
        .store()
        .await
        .set_identity_setting(
            &credential,
            SETTING,
            previous.revision,
            serde_json::to_value(&config)?,
        )
        .await?;
    let config = IdentityConfig {
        revision: setting.revision,
        ..config
    };
    Ok(Json(redacted(
        &config,
        state.store().await.count_passkeys().await?,
    )))
}

fn short(value: &str, maximum: usize, name: &str) -> ApiResult<String> {
    let value = value.trim();
    if value.len() > maximum || value.chars().any(char::is_control) {
        return Err(AppError::bad_request(format!("Invalid {name}")));
    }
    Ok(value.to_string())
}

fn secret(
    state: &AppState,
    input: Option<String>,
    clear: bool,
    prior: &Option<String>,
    target_changed: bool,
) -> ApiResult<Option<String>> {
    let input = input.filter(|value| !value.is_empty());
    if input
        .as_ref()
        .is_some_and(|value| value.len() > 8192 || value.contains(['\r', '\n']))
    {
        return Err(AppError::bad_request("Invalid provider credential"));
    }
    if clear && input.is_some() {
        return Err(AppError::bad_request(
            "Choose either a replacement secret or clear secret",
        ));
    }
    if clear {
        return Ok(None);
    }
    if let Some(input) = input {
        return Ok(Some(state.vault.seal(&input)?));
    }
    if target_changed && prior.is_some() {
        return Err(AppError::bad_request(
            "Changing the credential destination requires entering the secret again or clearing it",
        ));
    }
    Ok(prior.clone())
}

fn canonical_origin(
    value: Option<String>,
    rp_id: Option<String>,
) -> ApiResult<(Option<String>, Option<String>)> {
    let Some(value) = value.filter(|v| !v.trim().is_empty()) else {
        if rp_id.is_some_and(|id| !id.trim().is_empty()) {
            return Err(AppError::bad_request(
                "Set the public origin before the relying party ID",
            ));
        }
        return Ok((None, None));
    };
    let url =
        Url::parse(value.trim()).map_err(|_| AppError::bad_request("Invalid public origin"))?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::bad_request("Public origin needs a hostname"))?;
    let loopback = host == "localhost"
        || host
            .trim_matches(['[', ']'])
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
    if (!matches!(url.scheme(), "https") && !(url.scheme() == "http" && loopback))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::bad_request(
            "Use an HTTPS origin without path, query, or credentials; HTTP is only allowed on loopback",
        ));
    }
    let effective_rp = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if rp_id
        .filter(|id| !id.trim().is_empty())
        .is_some_and(|id| id.trim().to_ascii_lowercase() != effective_rp)
    {
        return Err(AppError::bad_request(
            "Relying party ID must equal the public origin hostname",
        ));
    }
    Ok((Some(url.origin().ascii_serialization()), Some(effective_rp)))
}

fn validated(
    state: &AppState,
    input: SettingsInput,
    prior: &IdentityConfig,
) -> ApiResult<IdentityConfig> {
    let (public_origin, rp_id) = canonical_origin(input.public_origin, input.rp_id)?;
    let client_id = short(&input.google.client_id, 254, "Google client ID")?;
    if input.google.enabled
        && (!client_id.ends_with(".apps.googleusercontent.com") || client_id.len() < 30)
    {
        return Err(AppError::bad_request(
            "Enter a Google OAuth web application client ID",
        ));
    }
    let google = GoogleConfig {
        enabled: input.google.enabled,
        client_secret_cipher: secret(
            state,
            input.google.client_secret,
            input.google.clear_client_secret,
            &prior.google.client_secret_cipher,
            prior.google.client_id != client_id,
        )?,
        client_id,
    };
    if google.enabled && (public_origin.is_none() || google.client_secret_cipher.is_none()) {
        return Err(AppError::bad_request(
            "Google sign-in requires a public origin and client secret",
        ));
    }
    let host = short(&input.smtp.host, 253, "SMTP hostname")?.to_ascii_lowercase();
    if host.contains(['/', '@', '?', '#', ' ', '\\']) {
        return Err(AppError::bad_request(
            "Enter an SMTP hostname without a URL scheme or path",
        ));
    }
    let security = input.smtp.security;
    if !matches!(security.as_str(), "tls" | "starttls" | "local") {
        return Err(AppError::bad_request(
            "Choose TLS, STARTTLS, or a loopback-only local relay",
        ));
    }
    if security == "local" && !host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback()) {
        return Err(AppError::bad_request(
            "Unencrypted SMTP is restricted to a literal loopback address",
        ));
    }
    let username = short(&input.smtp.username, 254, "SMTP username")?;
    let from_email = short(&input.smtp.from_email, 254, "sender email")?;
    let from_name = short(&input.smtp.from_name, 120, "sender name")?;
    let target_changed = prior.smtp.host != host
        || prior.smtp.port != input.smtp.port
        || prior.smtp.security != security
        || prior.smtp.username != username;
    let password_cipher = secret(
        state,
        input.smtp.password,
        input.smtp.clear_password,
        &prior.smtp.password_cipher,
        target_changed,
    )?;
    if input.smtp.enabled {
        if public_origin.is_none()
            || host.is_empty()
            || input.smtp.port == 0
            || from_email.parse::<Address>().is_err()
        {
            return Err(AppError::bad_request(
                "Email requires a public origin, SMTP host/port, and a valid sender address",
            ));
        }
        if username.is_empty() != password_cipher.is_none() {
            return Err(AppError::bad_request(
                "Provide both SMTP username and password, or leave both empty for a relay",
            ));
        }
    }
    Ok(IdentityConfig {
        revision: prior.revision,
        public_origin,
        rp_id,
        google,
        smtp: SmtpConfig {
            enabled: input.smtp.enabled,
            host,
            port: input.smtp.port,
            security,
            username,
            password_cipher,
            from_email,
            from_name,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_are_canonical_and_bound_to_one_rp() {
        let (origin, rp) = canonical_origin(Some("https://EXAMPLE.com:443/".into()), None).unwrap();
        assert_eq!(origin.as_deref(), Some("https://example.com"));
        assert_eq!(rp.as_deref(), Some("example.com"));
        assert!(canonical_origin(Some("http://localhost:3000".into()), None).is_ok());
        assert!(canonical_origin(Some("http://127.0.0.1:3000".into()), None).is_ok());
        for invalid in [
            "http://example.com",
            "https://user:pass@example.com",
            "https://example.com/path",
            "https://example.com/?next=bad",
            "https://example.com/#bad",
        ] {
            assert!(canonical_origin(Some(invalid.into()), None).is_err());
        }
        assert!(
            canonical_origin(
                Some("https://example.com".into()),
                Some("attacker.com".into())
            )
            .is_err()
        );
    }

    #[test]
    fn redacted_provider_configuration_never_exposes_ciphertext() {
        let config = IdentityConfig {
            google: GoogleConfig {
                client_secret_cipher: Some("GOOGLE_SECRET_CIPHERTEXT".into()),
                ..Default::default()
            },
            smtp: SmtpConfig {
                password_cipher: Some("SMTP_PASSWORD_CIPHERTEXT".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let encoded = redacted(&config, 0).to_string();
        assert!(!encoded.contains("CIPHERTEXT"));
        assert!(!encoded.contains("_cipher"));
        assert!(
            redacted(&config, 0)["smtp"]["has_password"]
                .as_bool()
                .unwrap()
        );
    }
}
