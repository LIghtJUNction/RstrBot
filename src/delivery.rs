//! Explicit outbound sends. Persist intent first; never replay an unknown send.
use crate::state::{AppError, AppState, sanitize_resource};
use axum::http::StatusCode;
use rstrbot_plugin::{InstanceContext, PluginError};
use rstrbot_store::{Resource, Role, Store, User, Visibility};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendRequest {
    pub text: String,
    pub idempotency_key: String,
}

/// The original login credential remains an in-memory lease. A valid member
/// identity alone cannot renew a logged-out, expired or password-revoked session.
async fn authorize(
    state: &AppState,
    db: &Store,
    user: &User,
    route: &Resource,
    credential: &str,
) -> Result<(), AppError> {
    if state.shutdown.is_cancelled() {
        return Err(AppError::conflict("The service is stopping"));
    }
    let current_user = db.authenticate(credential).await?;
    if current_user.id != user.id {
        return Err(rstrbot_store::Error::Unauthorized.into());
    }
    db.require_role(&route.workspace_id, &user.id, Role::Member)
        .await?;
    let current = db
        .get_resource(&route.workspace_id, &user.id, &route.id)
        .await?;
    if current.data["enabled"] != true || current.version != route.version {
        return Err(AppError::conflict("Delivery route authority changed"));
    }
    if route.data["allow_private_network"] == true
        && !state.config.read().await.allow_private_network
    {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "private_network_disabled",
            "Instance policy currently disables private-network access",
        ));
    }
    Ok(())
}

pub async fn send(
    state: AppState,
    user: &User,
    workspace: &str,
    route_id: &str,
    request: SendRequest,
    credential: &str,
) -> Result<Value, AppError> {
    if request.text.trim().is_empty() || request.text.len() > 32768 {
        return Err(AppError::bad_request("Message must contain 1–32768 bytes"));
    }
    let db = state.store().await;
    db.require_role(workspace, &user.id, Role::Member).await?;
    if db.recovery_mode().await? {
        return Err(AppError::conflict("Recovery mode prevents external sends"));
    }
    let route = db.get_resource(workspace, &user.id, route_id).await?;
    if route.kind != "routes" || route.data["enabled"] != true {
        return Err(AppError::bad_request("This delivery route is not enabled"));
    }
    // Do not issue a new durable intention using a revoked HTTP credential.
    if db.authenticate(credential).await?.id != user.id {
        return Err(rstrbot_store::Error::Unauthorized.into());
    }
    let manifest = state
        .packages
        .read()
        .await
        .get("webhook")
        .cloned()
        .ok_or_else(|| AppError::bad_request("Webhook adapter is not installed"))?;
    let operation_key = format!("delivery:{}", request.idempotency_key);
    let fingerprint = hex::encode(Sha256::digest(serde_json::to_vec(
        &json!({"route_id":route_id,"route_version":route.version,"text":request.text}),
    )?));
    if let Some(previous) = db
        .begin_operation(workspace, &user.id, &operation_key, &fingerprint)
        .await?
    {
        let id = previous["delivery_id"]
            .as_str()
            .ok_or_else(|| AppError::conflict("Original delivery requires reconciliation"))?;
        return Ok(serde_json::to_value(sanitize_resource(
            db.get_resource(workspace, &user.id, id).await?,
        ))?);
    }
    let receipt=db.create_resource(workspace,&user.id,"deliveries",Visibility::Private,json!({"route_id":route_id,"route_version":route.version,"text":request.text,"status":"prepared","delivery":"not_attempted","persistence":"saved","deduplication_guaranteed":false})).await?;
    db.finish_operation(
        workspace,
        &user.id,
        &operation_key,
        json!({"delivery_id":receipt.id}),
    )
    .await?;
    // Repeat authorization immediately before crossing the side-effect boundary.
    let private_requested = route.data["allow_private_network"]
        .as_bool()
        .unwrap_or(false);
    let private_allowed = private_requested && state.config.read().await.allow_private_network;
    if let Err(error) = authorize(&state, &db, user, &route, credential).await {
        // Existing receipts were returned above, even when networking is now
        // disabled. This newly claimed intention has not crossed the network
        // boundary; persist that fact instead of leaving an ambiguous claim.
        let mut data = receipt.data;
        data["status"] = json!("not_sent");
        data["delivery"] = json!("not_sent");
        data["finished_at"] = json!(chrono::Utc::now().timestamp());
        data["error"] = json!({"code":error.code,"message":error.message});
        let blocked = db
            .internal_update_resource(&receipt.id, receipt.version, data)
            .await?;
        db.internal_append_event(
            &blocked.id,
            "delivery.finished",
            json!({"delivery_id":blocked.id,"status":"not_sent"}),
        )
        .await?;
        return Err(error);
    }
    let mut data = receipt.data.clone();
    data["status"] = json!("sending");
    data["delivery"] = json!("in_flight");
    let pending = db
        .internal_update_resource(&receipt.id, receipt.version, data)
        .await?;
    let context = InstanceContext {
        workspace_id: workspace.into(),
        instance_id: route_id.into(),
        generation: route.version as u64,
    };
    let mut params = json!({"url":route.data["url"],"text":request.text,"operation_id":receipt.id,"allow_private_network":private_allowed});
    if let Some(cipher) = route.data["token_cipher"].as_str() {
        params["token"] = json!(state.vault.open(cipher)?);
    }
    let cancel = state.shutdown.child_token();
    let result = {
        let call = state.host.call_authorized(
            &manifest,
            &context,
            ("platform.send", params),
            cancel.clone(),
            None,
            || async {
                authorize(&state, &db, user, &route, credential)
                    .await
                    .map_err(|_| PluginError::Cancelled)
            },
        );
        let monitor = async {
            let mut tick = tokio::time::interval(Duration::from_millis(350));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                if let Err(error) = authorize(&state, &db, user, &route, credential).await {
                    return error;
                }
            }
        };
        tokio::pin!(call, monitor);
        tokio::select! {
            biased;
            result = &mut call => result.map_err(AppError::from),
            error = &mut monitor => {
                cancel.cancel();
                // The host may already know the final receipt while it reaps
                // the worker. Preserve that result. Otherwise keep uncertainty
                // and finish process cleanup before writing the terminal state.
                match tokio::time::timeout(Duration::from_secs(2), &mut call).await {
                    Ok(Ok(receipt)) => Ok(receipt),
                    _ => Err(error),
                }
            },
        }
    };
    let mut data = pending.data;
    match result {
        Ok(result) => {
            data["status"] = result["delivery"].clone();
            data["delivery"] = result["delivery"].clone();
            data["receipt"] = result;
        }
        Err(error) => {
            data["status"] = json!("unknown");
            data["delivery"] = json!("unknown");
            data["error"] = json!({"code":error.code,"message":"Delivery may have reached the destination; reconcile before creating another send"});
        }
    }
    data["finished_at"] = json!(chrono::Utc::now().timestamp());
    let receipt = db
        .internal_update_resource(&receipt.id, pending.version, data)
        .await?;
    db.internal_append_event(
        &receipt.id,
        "delivery.finished",
        json!({"delivery_id":receipt.id,"status":receipt.data["status"]}),
    )
    .await?;
    Ok(serde_json::to_value(sanitize_resource(receipt))?)
}

pub async fn recover(state: &AppState) -> Result<(), AppError> {
    let db = state.store().await;
    let mut cursor = None;
    loop {
        let page = db
            .internal_list_page("deliveries", cursor.as_ref(), 128)
            .await?;
        for mut receipt in page.items {
            let status = receipt.data["status"].as_str().unwrap_or_default();
            if status == "sending" || status == "prepared" {
                let outcome = if status == "sending" {
                    "unknown"
                } else {
                    "not_sent"
                };
                receipt.data["status"] = json!(outcome);
                receipt.data["delivery"] = json!(outcome);
                receipt.data["recovery"] =
                    json!("No automatic replay; review the original operation id");
                db.internal_update_resource(&receipt.id, receipt.version, receipt.data)
                    .await?;
            }
        }
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    Ok(())
}
