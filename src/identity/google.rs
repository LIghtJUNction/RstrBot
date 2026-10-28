//! Google's authorization-code OIDC flow. Endpoints are fixed, identity is the
//! verified issuer/subject pair, and an email address never links accounts.
use super::*;
use axum::{
    extract::Query,
    http::HeaderValue,
    response::{IntoResponse, Redirect},
};
use futures_util::StreamExt;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const FLOW_COOKIE: &str = "rstrbot_google_flow";
const FLOW_PATH: &str = "/api/v1/auth/google";
const MAX_OIDC_RESPONSE: usize = 131_072;

fn denied() -> AppError {
    AppError::new(
        StatusCode::UNAUTHORIZED,
        "google_verification_failed",
        "Google sign-in failed or expired. Start a new sign-in attempt",
    )
}

fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn secret_digest(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn same_secret(left: &str, right: &str) -> bool {
    let left = Sha256::digest(left.as_bytes());
    let right = Sha256::digest(right.as_bytes());
    left.iter()
        .zip(right.iter())
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

fn flow_cookie(headers: &HeaderMap) -> Option<&str> {
    let mut found = None;
    for header in headers.get_all(header::COOKIE) {
        for field in header.to_str().ok()?.split(';') {
            let (name, value) = field.trim().split_once('=')?;
            if name == FLOW_COOKIE {
                if found.is_some() || value.len() > 128 {
                    return None;
                }
                found = Some(value);
            }
        }
    }
    found
}

fn cookie(value: &str, secure: bool, max_age: i64) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(&format!(
        "{FLOW_COOKIE}={value}; Path={FLOW_PATH}; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        if secure { "; Secure" } else { "" },
    ))
    .map_err(|_| AppError::internal("OAuth cookie could not be encoded"))
}

fn configuration(config: &IdentityConfig) -> Result<(&str, String), AppError> {
    if !config.google.enabled || config.google.client_id.is_empty() {
        return Err(AppError::bad_request("Google sign-in is not configured"));
    }
    let origin = config.public_origin.as_deref().ok_or_else(|| {
        AppError::bad_request("Google sign-in requires a configured public origin")
    })?;
    Ok((origin, format!("{origin}/api/v1/auth/google/callback")))
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Mode {
    Login,
    Link,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StartInput {
    mode: Mode,
}

#[derive(Deserialize, Serialize)]
struct Intent {
    mode: Mode,
    revision: i64,
    nonce: String,
    binding_hash: String,
    verifier_cipher: String,
    session_cipher: Option<String>,
    user_id: Option<String>,
}

fn authorization_url(
    client_id: &str,
    redirect: &str,
    state: &str,
    intent: &Intent,
    verifier: &str,
) -> ApiResult<Url> {
    let mut url = Url::parse(AUTHORIZE_URL)
        .map_err(|_| AppError::internal("Google authorization URL is invalid"))?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email")
        .append_pair("state", state)
        .append_pair("nonce", &intent.nonce)
        .append_pair(
            "code_challenge",
            &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
        )
        .append_pair("code_challenge_method", "S256")
        .append_pair("prompt", "select_account");
    Ok(url)
}

pub(super) async fn start(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(body): Payload<StartInput>,
) -> ApiResult<Response> {
    let _permit = gate.enter("google-start")?;
    let config = identity_config::load(&state).await?;
    let (origin, redirect) = configuration(&config)?;
    let db = state.store().await;
    let (session_cipher, user_id) = if body.mode == Mode::Login {
        (None, None)
    } else {
        let session = token(&headers)?;
        let user = db.require_recent_session(&session).await?;
        (Some(state.vault.seal(&session)?), Some(user.id))
    };
    let verifier = random_secret();
    let binding = random_secret();
    let intent = Intent {
        mode: body.mode,
        revision: config.revision,
        nonce: random_secret(),
        binding_hash: secret_digest(&binding),
        verifier_cipher: state.vault.seal(&verifier)?,
        session_cipher,
        user_id,
    };
    let issued = db
        .create_auth_challenge(
            "google.oidc",
            None,
            None,
            serde_json::to_value(&intent)?,
            CEREMONY_SECONDS,
        )
        .await?;
    let url = authorization_url(
        &config.google.client_id,
        &redirect,
        &issued.token,
        &intent,
        &verifier,
    )?;
    let mut response =
        Json(json!({"url":url.as_str(),"expires_at":issued.expires_at})).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie(&binding, origin.starts_with("https://"), CEREMONY_SECONDS)?,
    );
    Ok(response)
}

#[derive(Deserialize)]
pub(super) struct CallbackInput {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

pub(super) async fn callback(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Query(body): Query<CallbackInput>,
) -> Response {
    let result = async {
        let _permit = gate.enter("google-callback")?;
        complete(&state, &headers, body).await
    }
    .await;
    let mut response = match result {
        Ok(response) => response,
        Err(error) => {
            // No provider token, email, authorization code or state is logged.
            tracing::warn!(code=%error.code, "Google sign-in did not complete");
            Redirect::to("/?auth=failed").into_response()
        }
    };
    let secure = identity_config::load(&state)
        .await
        .ok()
        .and_then(|config| config.public_origin)
        .is_some_and(|origin| origin.starts_with("https://"));
    if let Ok(clear) = cookie("", secure, 0) {
        response.headers_mut().append(header::SET_COOKIE, clear);
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

async fn complete(
    state: &AppState,
    headers: &HeaderMap,
    body: CallbackInput,
) -> ApiResult<Response> {
    let challenge = body
        .state
        .as_deref()
        .filter(|state| state.len() <= 128)
        .ok_or_else(denied)?;
    let binding = flow_cookie(headers).ok_or_else(denied)?;
    let db = state.store().await;
    // The original strict login cookie is deliberately not needed here: a
    // cross-site navigation does not carry it. Instead this one-use intent has
    // a separate browser binding and an encrypted original session lease.
    let intent: Intent = serde_json::from_value(
        db.take_auth_challenge(challenge, "google.oidc", None)
            .await?,
    )?;
    if !same_secret(&intent.binding_hash, &secret_digest(binding)) || body.error.is_some() {
        return Err(denied());
    }
    let code = body
        .code
        .as_deref()
        .filter(|code| !code.is_empty() && code.len() <= 8192)
        .ok_or_else(denied)?;
    let config = identity_config::load(state).await?;
    let (_, redirect) = configuration(&config)?;
    if config.revision != intent.revision {
        return Err(denied());
    }
    let original_session = intent
        .session_cipher
        .as_deref()
        .map(|cipher| state.vault.open(cipher))
        .transpose()?;
    if let Some(session) = original_session.as_deref() {
        let user = db.authenticate(session).await?;
        if intent.user_id.as_deref() != Some(user.id.as_str()) {
            return Err(denied());
        }
    }
    let client_secret = state.vault.open(
        config
            .google
            .client_secret_cipher
            .as_deref()
            .ok_or_else(denied)?,
    )?;
    let verifier = state.vault.open(&intent.verifier_cipher)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| AppError::internal("Google client could not start"))?;
    let exchanged = bounded_json(
        client
            .post(TOKEN_URL)
            .form(&[
                ("code", code),
                ("client_id", config.google.client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("redirect_uri", redirect.as_str()),
                ("grant_type", "authorization_code"),
                ("code_verifier", verifier.as_str()),
            ])
            .send()
            .await
            .map_err(|_| denied())?,
    )
    .await?;
    let id_token = exchanged
        .get("id_token")
        .and_then(Value::as_str)
        .ok_or_else(denied)?;
    let keys: JwkSet = serde_json::from_value(
        bounded_json(client.get(JWKS_URL).send().await.map_err(|_| denied())?).await?,
    )
    .map_err(|_| denied())?;
    let claims = verify_id_token(id_token, &keys, &config.google.client_id, &intent)?;
    if identity_config::load(state).await?.revision != intent.revision {
        return Err(denied());
    }
    let session = match intent.mode {
        Mode::Login => Some(
            db.login_google_verified(&claims.sub, intent.revision)
                .await?,
        ),
        Mode::Link => {
            let session = original_session.as_deref().ok_or_else(denied)?;
            db.link_google_verified(
                session,
                &claims.sub,
                claims
                    .email
                    .as_deref()
                    .filter(|_| claims.email_verified == Some(true)),
                intent.revision,
            )
            .await?;
            None
        }
    };
    let mut response = Redirect::to("/?auth=success").into_response();
    if let Some(session) = session {
        // Reuse the app's cookie attributes; never put bearer credentials into
        // a redirect URL or the callback response body.
        let authenticated = session_response(state, headers, session).await?;
        if let Some(cookie) = authenticated.headers().get(header::SET_COOKIE) {
            response
                .headers_mut()
                .insert(header::SET_COOKIE, cookie.clone());
        }
    }
    Ok(response)
}

async fn bounded_json(response: reqwest::Response) -> Result<Value, AppError> {
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > MAX_OIDC_RESPONSE as u64)
    {
        return Err(denied());
    }
    let mut chunks = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| denied())?;
        if body.len() + chunk.len() > MAX_OIDC_RESPONSE {
            return Err(denied());
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| denied())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: Audience,
    exp: u64,
    iat: u64,
    nonce: String,
    azp: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
}

fn verify_id_token(
    token: &str,
    keys: &JwkSet,
    client_id: &str,
    intent: &Intent,
) -> Result<Claims, AppError> {
    if token.len() > 32_768 || keys.keys.len() > 16 {
        return Err(denied());
    }
    let header = decode_header(token).map_err(|_| denied())?;
    if header.alg != Algorithm::RS256 {
        return Err(denied());
    }
    let kid = header
        .kid
        .as_deref()
        .filter(|kid| !kid.is_empty() && kid.len() <= 256)
        .ok_or_else(denied)?;
    let jwk = keys.find(kid).ok_or_else(denied)?;
    if jwk
        .common
        .public_key_use
        .as_ref()
        .is_some_and(|usage| !matches!(usage, jsonwebtoken::jwk::PublicKeyUse::Signature))
        || jwk
            .common
            .key_algorithm
            .is_some_and(|algorithm| algorithm != jsonwebtoken::jwk::KeyAlgorithm::RS256)
    {
        return Err(denied());
    }
    let key = DecodingKey::from_jwk(jwk).map_err(|_| denied())?;
    let mut policy = Validation::new(Algorithm::RS256);
    policy.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
    policy.set_audience(&[client_id]);
    policy.set_required_spec_claims(&["iss", "aud", "exp", "sub"]);
    policy.validate_nbf = true;
    policy.leeway = 30;
    let claims = decode::<Claims>(token, &key, &policy)
        .map_err(|_| denied())?
        .claims;
    let now = rstrbot_store::now().max(0) as u64;
    let audience_count = match &claims.aud {
        Audience::Single(aud) if aud == client_id => 1,
        Audience::Multiple(aud) if aud.iter().any(|value| value == client_id) => aud.len(),
        _ => return Err(denied()),
    };
    if !matches!(
        claims.iss.as_str(),
        "https://accounts.google.com" | "accounts.google.com"
    ) || claims.sub.is_empty()
        || claims.sub.len() > 255
        || !claims.sub.is_ascii()
        || claims
            .sub
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        || claims.iat > now.saturating_add(30)
        || claims.exp < claims.iat
        || !same_secret(&claims.nonce, &intent.nonce)
        || claims.azp.as_deref().is_some_and(|azp| azp != client_id)
        || (audience_count > 1 && claims.azp.as_deref() != Some(client_id))
    {
        return Err(denied());
    }
    Ok(claims)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UnlinkInput {
    id: String,
}

pub(super) async fn unlink(
    State(state): State<AppState>,
    headers: HeaderMap,
    Payload(body): Payload<UnlinkInput>,
) -> ApiResult<StatusCode> {
    state
        .store()
        .await
        .remove_google(&token(&headers)?, &body.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use openssl::{pkey::Private, rsa::Rsa};
    use std::sync::OnceLock;

    fn signing_key() -> &'static Rsa<Private> {
        static KEY: OnceLock<Rsa<Private>> = OnceLock::new();
        KEY.get_or_init(|| Rsa::generate(2048).unwrap())
    }

    fn keys() -> JwkSet {
        let key = signing_key();
        serde_json::from_value(json!({"keys":[{
            "kty":"RSA","use":"sig","alg":"RS256","kid":"fixture-key",
            "n":URL_SAFE_NO_PAD.encode(key.n().to_vec()),
            "e":URL_SAFE_NO_PAD.encode(key.e().to_vec()),
        }]}))
        .unwrap()
    }

    fn intent() -> Intent {
        Intent {
            mode: Mode::Login,
            revision: 1,
            nonce: "one-use-server-nonce".into(),
            binding_hash: secret_digest("browser-only-cookie"),
            verifier_cipher: "not-used-in-verifier-unit-test".into(),
            session_cipher: None,
            user_id: None,
        }
    }

    fn claims() -> Value {
        json!({
            "iss":"https://accounts.google.com","aud":"fixture-client",
            "sub":"google-stable-subject","nonce":"one-use-server-nonce",
            "iat":rstrbot_store::now(),"exp":rstrbot_store::now()+600,
            "email":"display-only@example.test","email_verified":true,
        })
    }

    fn signed(claims: &Value) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("fixture-key".into());
        encode(
            &header,
            claims,
            &EncodingKey::from_rsa_der(&signing_key().private_key_to_der().unwrap()),
        )
        .unwrap()
    }

    #[test]
    fn google_accepts_a_real_signature_and_uses_subject_not_email() {
        let verified =
            verify_id_token(&signed(&claims()), &keys(), "fixture-client", &intent()).unwrap();
        assert_eq!(verified.sub, "google-stable-subject");
        assert_eq!(verified.email.as_deref(), Some("display-only@example.test"));
        let mut alias_issuer = claims();
        alias_issuer["iss"] = json!("accounts.google.com");
        assert!(
            verify_id_token(&signed(&alias_issuer), &keys(), "fixture-client", &intent()).is_ok()
        );
    }

    #[test]
    fn google_rejects_signed_tokens_with_wrong_claims_or_missing_required_fields() {
        let now = rstrbot_store::now();
        for (field, value) in [
            ("iss", json!("https://attacker.invalid")),
            ("aud", json!("different-client")),
            ("nonce", json!("different-flow")),
            ("exp", json!(now - 120)),
            ("iat", json!(now + 120)),
            ("azp", json!("another-client")),
            ("sub", json!("")),
            ("nbf", json!(now + 120)),
        ] {
            let mut changed = claims();
            changed[field] = value;
            assert!(
                verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_err(),
                "{field}"
            );
        }
        for field in ["iss", "aud", "sub", "nonce", "exp", "iat"] {
            let mut changed = claims();
            changed.as_object_mut().unwrap().remove(field);
            assert!(
                verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_err(),
                "missing {field}"
            );
        }
    }

    #[test]
    fn google_rejects_invalid_signature_unknown_key_and_algorithm_substitution() {
        let token = signed(&claims());
        let (payload, signature) = token.rsplit_once('.').unwrap();
        let changed = format!(
            "{payload}.{}{}",
            if signature.starts_with('A') { "B" } else { "A" },
            &signature[1..]
        );
        assert!(verify_id_token(&changed, &keys(), "fixture-client", &intent()).is_err());
        let mut wrong_key = keys();
        wrong_key.keys[0].common.key_id = Some("rotated-key".into());
        assert!(verify_id_token(&token, &wrong_key, "fixture-client", &intent()).is_err());
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("fixture-key".into());
        let substituted = encode(
            &header,
            &claims(),
            &EncodingKey::from_secret(b"not-a-Google-signing-key"),
        )
        .unwrap();
        assert!(verify_id_token(&substituted, &keys(), "fixture-client", &intent()).is_err());
        let mut encryption_key = keys();
        encryption_key.keys[0].common.public_key_use =
            Some(jsonwebtoken::jwk::PublicKeyUse::Encryption);
        assert!(verify_id_token(&token, &encryption_key, "fixture-client", &intent()).is_err());
    }

    #[test]
    fn google_multiple_audiences_require_the_right_authorized_party() {
        let mut changed = claims();
        changed["aud"] = json!(["fixture-client", "another-service"]);
        assert!(verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_err());
        changed["azp"] = json!("another-service");
        assert!(verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_err());
        changed["azp"] = json!("fixture-client");
        assert!(verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_ok());
    }

    #[test]
    fn google_subject_is_bounded_ascii_and_keeps_its_exact_identity() {
        for bad in [
            "   ".into(),
            "different\nsubject".into(),
            "账户".into(),
            "x".repeat(256),
        ] {
            let mut changed = claims();
            changed["sub"] = json!(bad);
            assert!(
                verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).is_err()
            );
        }
        let mut changed = claims();
        changed["sub"] = json!("Case-Sensitive-Subject");
        let verified =
            verify_id_token(&signed(&changed), &keys(), "fixture-client", &intent()).unwrap();
        assert_eq!(verified.sub, "Case-Sensitive-Subject");
        assert!(serde_json::from_value::<StartInput>(json!({"mode":"reauth"})).is_err());
    }

    #[test]
    fn google_authorization_binds_code_pkce_nonce_without_claiming_provider_reauthentication() {
        let mut current = intent();
        let verifier = random_secret();
        assert_eq!(verifier.len(), 43);
        for mode in [Mode::Login, Mode::Link] {
            current.mode = mode;
            let url = authorization_url(
                "fixture-client",
                "https://rstrbot.example/api/v1/auth/google/callback",
                "server-stored-one-use-state",
                &current,
                &verifier,
            )
            .unwrap();
            assert_eq!(url.scheme(), "https");
            assert_eq!(url.host_str(), Some("accounts.google.com"));
            assert_eq!(url.path(), "/o/oauth2/v2/auth");
            let query = url
                .query_pairs()
                .collect::<std::collections::HashMap<_, _>>();
            assert_eq!(query["response_type"], "code");
            assert_eq!(query["client_id"], "fixture-client");
            assert_eq!(query["state"], "server-stored-one-use-state");
            assert_eq!(query["nonce"], current.nonce);
            assert_eq!(query["scope"], "openid email");
            assert_eq!(
                query["redirect_uri"],
                "https://rstrbot.example/api/v1/auth/google/callback"
            );
            assert_eq!(query["code_challenge_method"], "S256");
            assert_eq!(
                query["code_challenge"],
                URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
            );
            assert!(!url.as_str().contains(&verifier));
            assert!(!query.contains_key("client_secret"));
            assert!(!query.contains_key("max_age"));
            assert!(!query.contains_key("claims"));
        }
    }

    #[test]
    fn google_browser_binding_rejects_duplicate_cookie_values() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("rstrbot_google_flow=one"),
        );
        assert_eq!(flow_cookie(&headers), Some("one"));
        headers.append(
            header::COOKIE,
            HeaderValue::from_static("rstrbot_google_flow=two"),
        );
        assert!(flow_cookie(&headers).is_none());
        assert!(same_secret("same", "same"));
        assert!(!same_secret("first", "other"));
    }
}
