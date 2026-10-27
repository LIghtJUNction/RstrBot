//! Minimal public Rust worker SDK. It never imports core database objects.
use crate::{InstanceContext, MAX_FRAME_BYTES, PROTOCOL_VERSION, PluginError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::future::Future;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Bounded, credential-free error returned to the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcFailure {
    pub code: i64,
    pub message: String,
}
impl RpcFailure {
    pub fn invalid(message: &str) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }
    pub fn failed(message: &str) -> Self {
        Self {
            code: -32000,
            message: message.into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".into(),
        }
    }
    pub fn cancelled() -> Self {
        Self {
            code: -32800,
            message: "Invocation cancelled".into(),
        }
    }
}

/// Current call scope. A plugin cannot enlarge the core's authorization context.
#[derive(Clone)]
pub struct PluginCall {
    pub request_id: String,
    pub context: InstanceContext,
    pub cancellation: CancellationToken,
    outgoing: mpsc::Sender<Value>,
}
impl PluginCall {
    /// Emit a bounded progress event, with cancellation-aware backpressure.
    pub async fn emit(&self, method: &str, mut params: Value) -> Result<(), RpcFailure> {
        if !matches!(
            method,
            "model.delta" | "tool.progress" | "platform.progress" | "task.progress"
        ) {
            return Err(RpcFailure::invalid("Unsupported event method"));
        }
        let object = params
            .as_object_mut()
            .ok_or_else(|| RpcFailure::invalid("Event parameters must be an object"))?;
        object.insert("request_id".into(), self.request_id.clone().into());
        let frame = json!({"jsonrpc":"2.0","method":method,"params":params});
        if serde_json::to_vec(&frame)
            .map_err(|_| RpcFailure::invalid("Invalid event"))?
            .len()
            > MAX_FRAME_BYTES
        {
            return Err(RpcFailure::invalid("Event exceeds byte limit"));
        }
        tokio::select! {
            _ = self.cancellation.cancelled() => Err(RpcFailure::cancelled()),
            sent = self.outgoing.send(frame) => sent.map_err(|_| RpcFailure::failed("Host disconnected")),
        }
    }
}

/// Serve one host connection and one invocation, then exit.
///
/// Workers obtain the socket and bootstrap nonce from the launcher. Arbitrary
/// public TCP endpoints are deliberately not introduced by this SDK.
pub async fn serve<H, F>(capabilities: &[&str], handler: H) -> Result<(), PluginError>
where
    H: Fn(String, Value, PluginCall) -> F + Send + Sync + 'static,
    F: Future<Output = Result<Value, RpcFailure>> + Send + 'static,
{
    #[cfg(unix)]
    {
        serve_unix(capabilities, handler).await
    }
    #[cfg(not(unix))]
    {
        let _ = (capabilities, handler);
        Err(PluginError::UnsupportedPlatform)
    }
}

#[cfg(unix)]
async fn serve_unix<H, F>(capabilities: &[&str], handler: H) -> Result<(), PluginError>
where
    H: Fn(String, Value, PluginCall) -> F + Send + Sync + 'static,
    F: Future<Output = Result<Value, RpcFailure>> + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_util::codec::{Framed, LinesCodec};
    let socket = std::env::var_os("RSTRBOT_SOCKET")
        .ok_or_else(|| PluginError::Protocol("RSTRBOT_SOCKET is required".into()))?;
    let token = std::env::var("RSTRBOT_TOKEN")
        .map_err(|_| PluginError::Protocol("RSTRBOT_TOKEN is required".into()))?;
    let context: InstanceContext = serde_json::from_str(
        &std::env::var("RSTRBOT_CONTEXT")
            .map_err(|_| PluginError::Protocol("RSTRBOT_CONTEXT is required".into()))?,
    )
    .map_err(|_| PluginError::Protocol("invalid instance context".into()))?;
    let stream = tokio::net::UnixStream::connect(socket).await?;
    let mut wire = Framed::new(stream, LinesCodec::new_with_max_length(MAX_FRAME_BYTES));
    let hello: Value = serde_json::from_str(
        &wire
            .next()
            .await
            .ok_or_else(|| PluginError::Protocol("missing initialization".into()))?
            .map_err(|_| PluginError::Protocol("invalid initialization frame".into()))?,
    )
    .map_err(|_| PluginError::Protocol("invalid initialization JSON".into()))?;
    let hello_id = hello
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| PluginError::Protocol("initialize id required".into()))?;
    let expected_context = serde_json::to_value(&context)
        .map_err(|_| PluginError::Protocol("invalid context".into()))?;
    if hello.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || hello.get("method").and_then(Value::as_str) != Some("initialize")
        || hello
            .pointer("/params/protocol_version")
            .and_then(Value::as_str)
            != Some(PROTOCOL_VERSION)
        || hello.pointer("/params/token").and_then(Value::as_str) != Some(token.as_str())
        || hello.pointer("/params/context") != Some(&expected_context)
    {
        let error = json!({"jsonrpc":"2.0","id":hello_id,"error":{"code":-32001,"message":"Initialization rejected"}});
        wire.send(error.to_string())
            .await
            .map_err(|_| PluginError::Protocol("initialization write failed".into()))?;
        return Err(PluginError::Protocol("initialization rejected".into()));
    }
    let initialized = json!({"jsonrpc":"2.0","id":hello_id,"result":{
        "protocol_version":PROTOCOL_VERSION,"capabilities":capabilities,
        "workspace_id":context.workspace_id,"instance_id":context.instance_id,"generation":context.generation
    }});
    wire.send(initialized.to_string())
        .await
        .map_err(|_| PluginError::Protocol("handshake write failed".into()))?;
    let request: Value = serde_json::from_str(
        &wire
            .next()
            .await
            .ok_or_else(|| PluginError::Protocol("missing request".into()))?
            .map_err(|_| PluginError::Protocol("invalid request frame".into()))?,
    )
    .map_err(|_| PluginError::Protocol("invalid request JSON".into()))?;
    if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(PluginError::Protocol("JSON-RPC 2.0 required".into()));
    }
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| PluginError::Protocol("request id required".into()))?
        .to_owned();
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| PluginError::Protocol("method required".into()))?
        .to_owned();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    let (mut writer, mut reader) = wire.split();
    let (sender, mut receiver) = mpsc::channel::<Value>(32);
    let writer_task = tokio::spawn(async move {
        while let Some(value) = receiver.recv().await {
            let terminal = value.get("id").is_some()
                && (value.get("result").is_some() || value.get("error").is_some());
            let frame = value.to_string();
            if frame.len() > MAX_FRAME_BYTES {
                return Err(PluginError::Protocol(
                    "outgoing frame exceeds byte limit".into(),
                ));
            }
            writer
                .send(frame)
                .await
                .map_err(|_| PluginError::Protocol("host disconnected".into()))?;
            if terminal {
                break;
            }
        }
        Ok::<_, PluginError>(())
    });
    let cancellation = CancellationToken::new();
    let call = PluginCall {
        request_id: id.clone(),
        context,
        cancellation: cancellation.clone(),
        outgoing: sender.clone(),
    };
    let invocation = handler(method, params, call);
    tokio::pin!(invocation);
    let result = tokio::select! {
            result = &mut invocation => result,
            incoming = reader.next() => {
                match incoming {
                    Some(Ok(text)) => {
                        let frame: Value = serde_json::from_str(&text).map_err(|_| PluginError::Protocol("invalid cancellation JSON".into()))?;
                        if frame.get("method").and_then(Value::as_str) == Some("cancel")
                            && frame.pointer("/params/request_id").and_then(Value::as_str) == Some(id.as_str()) {
                            cancellation.cancel();
                            Err(RpcFailure::cancelled())
                        } else {
                            Err(RpcFailure::invalid("Only one invocation is allowed per worker"))
                        }
                    },
                    _ => { cancellation.cancel(); Err(RpcFailure::cancelled()) }
                }
            }
    };
    let response = match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(error) => json!({"jsonrpc":"2.0","id":id,"error":error}),
    };
    sender
        .send(response)
        .await
        .map_err(|_| PluginError::Protocol("host disconnected".into()))?;
    drop(sender);
    // Explicitly close rather than waiting for sender clones retained in the
    // completed handler future; the writer has already queued the final frame.
    writer_task
        .await
        .map_err(|_| PluginError::Protocol("writer stopped".into()))??;
    Ok(())
}
