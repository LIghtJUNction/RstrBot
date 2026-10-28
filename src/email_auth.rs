//! Verified email binding and recovery. Verification secrets are random,
//! single-use, hashed in storage, and only delivered through the configured MTA.

use crate::{
    api::{ApiResult, LoginGate, Payload, token},
    identity_config::{self, IdentityConfig},
    state::{AppError, AppState},
};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use lettre::{
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/email/start", post(start_email))
        .route("/auth/email/verify", post(verify_email))
        .route("/auth/password/reset/request", post(request_reset))
        .route("/auth/password/reset/finish", post(finish_reset))
        .route("/admin/auth-settings/test-email", post(test_email))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmailInput {
    email: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifyInput {
    token: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResetInput {
    token: String,
    new_password: String,
}

fn email_address(input: &str) -> ApiResult<String> {
    let input = input.trim().to_ascii_lowercase();
    if input.len() > 254 || input.parse::<Address>().is_err() {
        return Err(AppError::bad_request("Enter a valid email address"));
    }
    Ok(input)
}

fn rate_key(scope: &str, input: &str) -> String {
    format!("{scope}:{}", hex::encode(Sha256::digest(input.as_bytes())))
}

fn available(config: &IdentityConfig) -> ApiResult<&str> {
    if !config.smtp.enabled {
        return Err(AppError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "email_not_configured",
            "Email delivery has not been configured by the administrator",
        ));
    }
    config.public_origin.as_deref().ok_or_else(|| {
        AppError::bad_request("Configure the public origin before sending account email")
    })
}

async fn start_email(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(input): Payload<EmailInput>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let user = state
        .store()
        .await
        .require_recent_session(&credential)
        .await?;
    let _permit = gate.enter(&rate_key("email-bind", &user.id))?;
    let email = email_address(&input.email)?;
    let config = identity_config::load(&state).await?;
    let origin = available(&config)?;
    let challenge = state
        .store()
        .await
        .begin_email_change(&credential, &email)
        .await?;
    let link = format!("{origin}/#verify-email={}", challenge.token);
    let body = format!(
        "Confirm your email for RstrBot / 验证你的 RstrBot 邮箱\n\nOpen this link in the same browser where you requested the change, then select Confirm email.\n请在发起换绑的浏览器中打开链接，然后选择确认验证。\n\n{link}\n\nThis link expires in 15 minutes and can be used once. Your previous email remains active until verification succeeds.\n此链接 15 分钟内有效，仅能使用一次。验证成功前保留原邮箱。\n\nIf you did not request this, ignore this message.\n如果不是你发起的操作，请忽略此邮件。\n"
    );
    send(
        &state,
        &config,
        &email,
        "Verify your RstrBot email / 验证邮箱",
        body,
    )
    .await?;
    // An email being delivered does not extend or revive a revoked session.
    state.store().await.authenticate(&credential).await?;
    Ok(Json(
        json!({"pending":true,"expires_at":challenge.expires_at}),
    ))
}

async fn verify_email(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(input): Payload<VerifyInput>,
) -> ApiResult<Json<Value>> {
    let credential = token(&headers)?;
    let user = state.store().await.authenticate(&credential).await?;
    let _permit = gate.enter(&rate_key("email-verify", &user.id))?;
    if input.token.len() > 256 {
        return Err(AppError::bad_request("Invalid verification link"));
    }
    let verified = match state
        .store()
        .await
        .consume_email_change(&credential, &input.token)
        .await
    {
        Ok(verified) => verified,
        Err(rstrbot_store::Error::Unauthorized) => {
            state.store().await.authenticate(&credential).await?;
            return Err(AppError::new(
                StatusCode::FORBIDDEN,
                "email_verification_failed",
                "This email link is invalid, expired, or belongs to a different sign-in session",
            ));
        }
        Err(error) => return Err(error.into()),
    };
    Ok(Json(serde_json::to_value(verified)?))
}

async fn request_reset(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    Payload(input): Payload<EmailInput>,
) -> ApiResult<Json<Value>> {
    let email = email_address(&input.email)?;
    let permit = gate.enter(&rate_key("email-reset", &email))?;
    let config = identity_config::load(&state).await?;
    let origin = available(&config)?.to_string();
    let challenge = state.store().await.begin_password_reset(&email).await?;
    // Delivery runs out of the public response path. Registered/unregistered
    // addresses get the same response, including when the MTA rejects mail.
    if let Some(challenge) = challenge {
        tokio::spawn(async move {
            let _permit = permit;
            let link = format!("{origin}/#reset-password={}", challenge.token);
            let body = format!(
                "Reset your RstrBot password / 重设 RstrBot 密码\n\n{link}\n\nOpen the link and choose a new password. This link expires in 15 minutes and can be used once. Resetting your password signs out all existing sessions.\n打开链接并设置新密码。此链接 15 分钟内有效，仅能使用一次。重设密码会退出所有已登录会话。\n\nIf you did not request this, ignore this message; your password has not changed.\n如果不是你发起的操作，请忽略此邮件；你的密码尚未更改。\n"
            );
            if send(
                &state,
                &config,
                &email,
                "Reset your RstrBot password / 重设密码",
                body,
            )
            .await
            .is_err()
            {
                // Never log recipients, message content, provider credentials,
                // verification tokens, or the SMTP server's free-form error.
                tracing::warn!("Account recovery email could not be delivered");
            }
        });
    }
    Ok(Json(json!({"accepted":true})))
}

async fn finish_reset(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    Payload(input): Payload<ResetInput>,
) -> ApiResult<StatusCode> {
    let _permit = gate.enter("email-reset-finish")?;
    if input.token.len() > 256 || input.new_password.len() > 1024 {
        return Err(AppError::bad_request("Invalid password reset request"));
    }
    state
        .store()
        .await
        .finish_password_reset(&input.token, &input.new_password)
        .await
        .map_err(|error| {
            if matches!(error, rstrbot_store::Error::Unauthorized) {
                AppError::new(
                    StatusCode::BAD_REQUEST,
                    "password_reset_failed",
                    "This password reset link is invalid or expired; request a new link",
                )
            } else {
                error.into()
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn test_email(
    State(state): State<AppState>,
    Extension(gate): Extension<Arc<LoginGate>>,
    headers: HeaderMap,
    Payload(input): Payload<EmailInput>,
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
    let _permit = gate.enter(&rate_key("smtp-test", &user.id))?;
    let email = email_address(&input.email)?;
    let config = identity_config::load(&state).await?;
    available(&config)?;
    send(&state, &config, &email, "RstrBot email test / 邮件测试",
        "Your RstrBot outgoing email configuration is working.\nRstrBot 发信服务已成功递交这封测试邮件。\n".into()).await?;
    if !state
        .store()
        .await
        .require_recent_session(&credential)
        .await?
        .instance_admin
    {
        return Err(AppError::forbidden());
    }
    Ok(Json(json!({"accepted":true})))
}

async fn send(
    state: &AppState,
    config: &IdentityConfig,
    recipient: &str,
    subject: &str,
    body: String,
) -> ApiResult<()> {
    available(config)?;
    let smtp = &config.smtp;
    let sender: Address = smtp
        .from_email
        .parse()
        .map_err(|_| AppError::bad_request("Invalid configured sender address"))?;
    let recipient: Address = recipient
        .parse()
        .map_err(|_| AppError::bad_request("Invalid recipient address"))?;
    let message = Message::builder()
        .from(Mailbox::new(Some(smtp.from_name.clone()), sender))
        .to(Mailbox::new(None, recipient))
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|_| AppError::bad_request("Email could not be constructed"))?;
    let mut builder = match smtp.security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host),
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host),
        "local"
            if smtp
                .host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback()) =>
        {
            Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
                &smtp.host,
            ))
        }
        _ => return Err(AppError::bad_request("Invalid SMTP transport security")),
    }
    .map_err(|_| AppError::bad_request("SMTP transport could not be configured"))?
    .port(smtp.port)
    .timeout(Some(Duration::from_secs(8)));
    if let Some(cipher) = &smtp.password_cipher {
        builder = builder.credentials(Credentials::new(
            smtp.username.clone(),
            state.vault.open(cipher)?,
        ));
    }
    let mailer = builder.build();
    match tokio::time::timeout(Duration::from_secs(20), mailer.send(message)).await {
        Ok(Ok(_)) => Ok(()),
        _ => Err(AppError::new(
            StatusCode::BAD_GATEWAY,
            "email_delivery_failed",
            "The email could not be delivered; check the saved SMTP configuration",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_addresses_cannot_inject_headers_or_display_names() {
        assert_eq!(
            email_address(" Alice@Example.com ").unwrap(),
            "alice@example.com"
        );
        for bad in [
            "",
            "name",
            "Alice <alice@example.com>",
            "alice@example.com\r\nBcc:other@example.com",
        ] {
            assert!(email_address(bad).is_err());
        }
    }
}
