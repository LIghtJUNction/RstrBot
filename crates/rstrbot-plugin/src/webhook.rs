//! A small real platform adapter for administrator-approved HTTP webhooks.
//! It sends once and reports unknown delivery after ambiguous network failures.
use crate::{provider::approved_client, sdk::RpcFailure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Journal a standalone service intent before touching the network.
/// Existing pending or unknown entries are returned for reconciliation, never replayed.
pub async fn send_once(directory: &Path, params: &Value) -> Result<Value, RpcFailure> {
    use tokio::io::AsyncWriteExt;
    let operation = params
        .get("operation_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 100)
        .ok_or_else(|| RpcFailure::invalid("A bounded operation_id is required"))?;
    let fingerprint = hex::encode(Sha256::digest(
        serde_json::to_vec(&json!({
            "url":params.get("url"),"text":params.get("text"),"operation_id":operation
        }))
        .map_err(|_| RpcFailure::invalid("Invalid operation"))?,
    ));
    let name = hex::encode(Sha256::digest(operation.as_bytes()));
    tokio::fs::create_dir_all(directory)
        .await
        .map_err(|_| RpcFailure::failed("Cannot open delivery journal"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
            .await
            .map_err(|_| RpcFailure::failed("Cannot protect delivery journal"))?;
    }
    let path = directory.join(format!("{name}.json"));
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(&path).await {
        Ok(mut file) => {
            let pending = json!({"fingerprint":fingerprint,"receipt":{"delivery":"unknown","operation_id":operation,
                "reason":"Intent exists; send may be in flight or its receipt may not have persisted. Reconcile before retrying."}});
            file.write_all(pending.to_string().as_bytes())
                .await
                .map_err(|_| RpcFailure::failed("Cannot persist intent; nothing was sent"))?;
            file.sync_all()
                .await
                .map_err(|_| RpcFailure::failed("Cannot sync intent; nothing was sent"))?;
            sync_directory(directory).await?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = tokio::fs::symlink_metadata(&path).await.map_err(|_| {
                RpcFailure::failed("Cannot read existing intent; reconcile manually")
            })?;
            if !metadata.is_file() || metadata.len() > 16_384 {
                return Err(RpcFailure::failed(
                    "Invalid intent record; reconcile manually",
                ));
            }
            let bytes = tokio::fs::read(&path).await.map_err(|_| {
                RpcFailure::failed("Cannot read existing intent; reconcile manually")
            })?;
            let existing: Value = serde_json::from_slice(&bytes)
                .map_err(|_| RpcFailure::failed("Incomplete intent record; reconcile manually"))?;
            if existing["fingerprint"] != fingerprint {
                return Err(RpcFailure::invalid(
                    "operation_id belongs to different parameters",
                ));
            }
            return Ok(existing["receipt"].clone());
        }
        Err(_) => {
            return Err(RpcFailure::failed(
                "Cannot create delivery intent; nothing was sent",
            ));
        }
    }
    let receipt = match send(params).await {
        Ok(receipt) => receipt,
        Err(error) => {
            json!({"delivery":"not_sent","operation_id":operation,"reason":error.message})
        }
    };
    let record = json!({"fingerprint":fingerprint,"receipt":receipt});
    let temporary = directory.join(format!("{name}.{}.tmp", uuid::Uuid::new_v4().simple()));
    let mut temporary_options = tokio::fs::OpenOptions::new();
    temporary_options.write(true).create_new(true);
    #[cfg(unix)]
    temporary_options.mode(0o600);
    let persist = async {
        let mut file = temporary_options.open(&temporary).await?;
        file.write_all(record.to_string().as_bytes()).await?;
        file.sync_all().await?;
        tokio::fs::rename(&temporary, &path).await?;
        Ok::<_, std::io::Error>(())
    }
    .await;
    if persist.is_err() || sync_directory(directory).await.is_err() {
        let mut honest = receipt;
        honest["persistence"] = "failed".into();
        honest["reconciliation_required"] = true.into();
        return Ok(honest);
    }
    Ok(receipt)
}

async fn sync_directory(path: &Path) -> Result<(), RpcFailure> {
    #[cfg(unix)]
    {
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || std::fs::File::open(path)?.sync_all())
            .await
            .map_err(|_| RpcFailure::failed("Journal sync task failed"))?
            .map_err(|_| RpcFailure::failed("Journal directory sync failed"))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Send once to the configured route. The core must authorize the stored route;
/// no caller-controlled destination should reach this function without approval.
pub async fn send(params: &Value) -> Result<Value, RpcFailure> {
    let url = params
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcFailure::invalid("A configured webhook URL is required"))?;
    let operation = params
        .get("operation_id")
        .and_then(Value::as_str)
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 100
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
        })
        .ok_or_else(|| RpcFailure::invalid("A bounded operation_id is required"))?;
    let text = params
        .get("text")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 65_536)
        .ok_or_else(|| RpcFailure::invalid("text must contain 1 to 65536 bytes"))?;
    let (client, _) = approved_client(
        url,
        params
            .get("allow_private_network")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    )
    .await?;
    let mut request = client
        .post(url)
        .header("Idempotency-Key", operation)
        .json(&json!({"text":text,"operation_id":operation}));
    if let Some(token) = params.get("token").and_then(Value::as_str) {
        if token.len() > 8192 || token.contains(['\r', '\n']) {
            return Err(RpcFailure::invalid("Invalid webhook credential"));
        }
        request = request.bearer_auth(token);
    }
    // Sending the idempotency key does not imply that a remote webhook honors it.
    // The core journals the intent and receipt; this adapter never retries.
    match request.send().await {
        Ok(response) if response.status().is_success() => Ok(json!({
            "delivery":"accepted","http_status":response.status().as_u16(),
            "operation_id":operation,"deduplication_guaranteed":false,
            "receipt_id":response.headers().get("x-request-id").and_then(|v|v.to_str().ok()).map(|s|s.chars().take(200).collect::<String>())
        })),
        Ok(response) => Ok(
            json!({"delivery":"rejected","http_status":response.status().as_u16(),"operation_id":operation,"deduplication_guaranteed":false}),
        ),
        Err(_) => Ok(
            json!({"delivery":"unknown","operation_id":operation,"deduplication_guaranteed":false,
            "reason":"Connection ended without a reliable receipt. Reconcile before retrying."}),
        ),
    }
}

/// MCP stdio service, scoped to exactly one operator-configured destination.
/// A caller may select text and an intent ID, never the URL or credential.
pub async fn serve_mcp_stdio() -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
    let url = std::env::var("RSTRBOT_WEBHOOK_URL")?;
    let token = std::env::var("RSTRBOT_WEBHOOK_TOKEN").ok();
    let private = std::env::var("RSTRBOT_ALLOW_PRIVATE_NETWORK").is_ok_and(|value| value == "1");
    let journal = std::path::PathBuf::from(std::env::var("RSTRBOT_WEBHOOK_STATE")?);
    if !journal.is_absolute() {
        return Err("RSTRBOT_WEBHOOK_STATE must be an absolute private directory".into());
    }
    let mut reader = FramedRead::new(
        tokio::io::stdin(),
        LinesCodec::new_with_max_length(crate::MAX_FRAME_BYTES),
    );
    let mut writer = FramedWrite::new(
        tokio::io::stdout(),
        LinesCodec::new_with_max_length(crate::MAX_FRAME_BYTES),
    );
    let mut initialized = false;
    while let Some(line) = reader.next().await {
        let frame: Value = serde_json::from_str(&line?)?;
        if frame.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Err("JSON-RPC 2.0 required".into());
        }
        let Some(id) = frame.get("id").cloned() else {
            continue;
        };
        let method = frame
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let result = match method {
            "initialize" => {
                initialized = true;
                Ok(
                    json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"rstrbot-webhook","version":env!("CARGO_PKG_VERSION")},"instructions":"This service can send to one operator-configured webhook. It cannot change destination. Unknown deliveries require reconciliation, not automatic replay."}),
                )
            }
            "ping" => Ok(json!({})),
            _ if !initialized => Err(RpcFailure::failed("Initialize before calling tools")),
            "tools/list" => Ok(
                json!({"tools":[{"name":"send_message","description":"Send one message to the configured webhook. Never automatically repeat an uncertain delivery.","inputSchema":{"type":"object","properties":{"text":{"type":"string","minLength":1,"maxLength":65536},"operation_id":{"type":"string","minLength":1,"maxLength":100}},"required":["text","operation_id"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":false,"openWorldHint":true}}]}),
            ),
            "tools/call" => {
                let name = frame.pointer("/params/name").and_then(Value::as_str);
                let arguments = frame
                    .pointer("/params/arguments")
                    .and_then(Value::as_object);
                if name != Some("send_message")
                    || arguments.is_none_or(|args| {
                        args.keys()
                            .any(|key| !matches!(key.as_str(), "text" | "operation_id"))
                    })
                {
                    Err(RpcFailure::invalid(
                        "Only the configured send_message capability is permitted",
                    ))
                } else {
                    let mut parameters = frame
                        .pointer("/params/arguments")
                        .cloned()
                        .unwrap_or(json!({}));
                    parameters["url"] = url.clone().into();
                    parameters["token"] = json!(token);
                    parameters["allow_private_network"] = private.into();
                    match send_once(&journal, &parameters).await {
                        Ok(receipt) => Ok(
                            json!({"content":[{"type":"text","text":receipt.to_string()}],"isError":receipt["delivery"]!="accepted"}),
                        ),
                        Err(error) => Ok(
                            json!({"content":[{"type":"text","text":error.message}],"isError":true}),
                        ),
                    }
                }
            }
            _ => Err(RpcFailure::not_found()),
        };
        let response = match result {
            Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
            Err(error) => json!({"jsonrpc":"2.0","id":id,"error":error}),
        };
        writer.send(response.to_string()).await?;
    }
    Ok(())
}
