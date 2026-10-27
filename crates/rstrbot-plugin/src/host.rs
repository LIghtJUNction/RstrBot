use crate::{MAX_FRAME_BYTES, Manifest, PROTOCOL_VERSION, PluginError, validate_manifest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{future::Future, path::PathBuf};
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;

/// Trusted context assigned by the core, never accepted from plugin output.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceContext {
    pub workspace_id: String,
    pub instance_id: String,
    pub generation: u64,
}

/// A notification attached to the current invocation by the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcEvent {
    pub method: String,
    pub params: Value,
}

/// Bounded launcher for trusted plugin executables.
///
/// Unix workers use a private Unix socket. Each call has a fresh credential,
/// process group and working directory. Disabling a plugin must cancel tokens
/// held by the core. No automatic retry is performed for uncertain operations.
#[derive(Clone)]
pub struct PluginHost {
    runtime_dir: PathBuf,
    slots: std::sync::Arc<Semaphore>,
}

impl PluginHost {
    pub fn new(runtime_dir: PathBuf) -> Self {
        Self {
            runtime_dir,
            slots: std::sync::Arc::new(Semaphore::new(8)),
        }
    }

    /// Execute a single RPC with finite queue, handshake and execution limits.
    /// The bounded event channel must be consumed concurrently with this future.
    pub async fn call(
        &self,
        manifest: &Manifest,
        context: &InstanceContext,
        method: &str,
        params: Value,
        cancellation: CancellationToken,
        events: Option<mpsc::Sender<RpcEvent>>,
    ) -> Result<Value, PluginError> {
        self.call_authorized(
            manifest,
            context,
            (method, params),
            cancellation,
            events,
            || std::future::ready(Ok(())),
        )
        .await
    }

    /// Revalidate core-owned authority after waiting for a process slot and
    /// immediately before issuing the business RPC. The callback is never
    /// supplied by the worker and runs without a shared mutex or DB transaction.
    /// An error prevents that invocation; after RPC dispatch, cancellation alone
    /// cannot prove that an external side effect was undone.
    pub async fn call_authorized<F, A>(
        &self,
        manifest: &Manifest,
        context: &InstanceContext,
        request: (&str, Value),
        cancellation: CancellationToken,
        events: Option<mpsc::Sender<RpcEvent>>,
        authorize: F,
    ) -> Result<Value, PluginError>
    where
        F: Fn() -> A + Send + Sync,
        A: Future<Output = Result<(), PluginError>> + Send,
    {
        let (method, params) = request;
        validate_manifest(manifest)?;
        if !manifest.permits(method) {
            return Err(PluginError::CapabilityDenied(method.into()));
        }
        if context.workspace_id.len() > 128
            || context.instance_id.len() > 128
            || context.workspace_id.is_empty()
            || context.instance_id.is_empty()
        {
            return Err(PluginError::Protocol(
                "invalid trusted instance context".into(),
            ));
        }
        if serde_json::to_vec(&params)
            .map_err(|_| PluginError::Protocol("invalid parameters".into()))?
            .len()
            > MAX_FRAME_BYTES / 2
        {
            return Err(PluginError::Protocol("request exceeds byte limit".into()));
        }
        let _slot = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(PluginError::Cancelled),
            permit = tokio::time::timeout(std::time::Duration::from_secs(10), self.slots.acquire()) =>
                permit.map_err(|_| PluginError::Timeout("queue"))?.map_err(|_| PluginError::Protocol("host closed".into()))?,
        };
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(PluginError::Cancelled),
            result = tokio::time::timeout(std::time::Duration::from_secs(10), authorize()) =>
                result.map_err(|_| PluginError::Timeout("authorization"))??,
        }
        #[cfg(unix)]
        {
            self.call_unix(
                manifest,
                context,
                (method, params),
                cancellation,
                events,
                &authorize,
            )
            .await
        }
        #[cfg(not(unix))]
        {
            let _ = (&self.runtime_dir, context, method, params, events);
            Err(PluginError::UnsupportedPlatform)
        }
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use nix::{
        sys::signal::{Signal, killpg},
        unistd::Pid,
    };
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::{os::unix::fs::PermissionsExt, process::Stdio, time::Duration};
    use tokio::{
        io::AsyncReadExt,
        net::UnixListener,
        process::{Child, Command},
    };
    use tokio_util::codec::{Framed, LinesCodec};

    type Wire = Framed<tokio::net::UnixStream, LinesCodec>;

    struct ChildGroup {
        child: Child,
        group: Pid,
        armed: bool,
    }
    impl ChildGroup {
        async fn cleanup(&mut self) {
            let _ = killpg(self.group, Signal::SIGTERM);
            let _ = tokio::time::timeout(Duration::from_millis(100), self.child.wait()).await;
            // The leader may already have exited while descendants still exist.
            let _ = killpg(self.group, Signal::SIGKILL);
            let _ = self.child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(1), self.child.wait()).await;
            self.armed = false;
        }
    }
    impl Drop for ChildGroup {
        fn drop(&mut self) {
            if self.armed {
                let _ = killpg(self.group, Signal::SIGKILL);
                let _ = self.child.start_kill();
            }
        }
    }

    struct RunDirectory(PathBuf);
    impl Drop for RunDirectory {
        fn drop(&mut self) {
            // Workers are trusted; the run directory contains only transient IPC.
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    async fn send(wire: &mut Wire, value: Value) -> Result<(), PluginError> {
        let text = serde_json::to_string(&value)
            .map_err(|_| PluginError::Protocol("cannot encode frame".into()))?;
        if text.len() > MAX_FRAME_BYTES {
            return Err(PluginError::Protocol("frame exceeds byte limit".into()));
        }
        wire.send(text)
            .await
            .map_err(|_| PluginError::Protocol("cannot write frame".into()))
    }

    async fn receive(wire: &mut Wire) -> Result<Value, PluginError> {
        let frame = wire
            .next()
            .await
            .ok_or_else(|| {
                PluginError::Protocol("worker disconnected before a final response".into())
            })?
            .map_err(|_| PluginError::Protocol("invalid or oversized frame".into()))?;
        let value: Value = serde_json::from_str(&frame)
            .map_err(|_| PluginError::Protocol("invalid JSON".into()))?;
        if value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") || !value.is_object() {
            return Err(PluginError::Protocol("JSON-RPC 2.0 object required".into()));
        }
        Ok(value)
    }

    fn response(frame: &Value, id: &str) -> Result<Option<Value>, PluginError> {
        if frame.get("id").and_then(Value::as_str) != Some(id) {
            return Err(PluginError::Protocol(
                "response id does not match invocation".into(),
            ));
        }
        if frame.get("result").is_some() && frame.get("error").is_some() {
            return Err(PluginError::Protocol(
                "response contains result and error".into(),
            ));
        }
        if let Some(error) = frame.get("error") {
            // Raw messages from a trusted worker are bounded; providers never
            // include URLs, headers or remote response bodies in these messages.
            let message: String = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("plugin error")
                .chars()
                .take(500)
                .collect();
            return Err(PluginError::Remote {
                code: error.get("code").and_then(Value::as_i64).unwrap_or(-32603),
                message,
            });
        }
        Ok(frame.get("result").cloned())
    }

    impl PluginHost {
        pub(super) async fn call_unix<F, A>(
            &self,
            manifest: &Manifest,
            context: &InstanceContext,
            request: (&str, Value),
            cancellation: CancellationToken,
            events: Option<mpsc::Sender<RpcEvent>>,
            authorize: &F,
        ) -> Result<Value, PluginError>
        where
            F: Fn() -> A + Send + Sync,
            A: Future<Output = Result<(), PluginError>> + Send,
        {
            let (method, params) = request;
            let executable = tokio::fs::canonicalize(&manifest.executable)
                .await
                .map_err(|e| PluginError::Spawn(e.to_string()))?;
            if let Some(expected) = &manifest.sha256 {
                let mut file = tokio::fs::File::open(&executable).await?;
                let mut hash = Sha256::new();
                let mut buffer = [0u8; 65536];
                loop {
                    let count = file.read(&mut buffer).await?;
                    if count == 0 {
                        break;
                    }
                    hash.update(&buffer[..count]);
                }
                if !hex::encode(hash.finalize()).eq_ignore_ascii_case(expected) {
                    return Err(PluginError::ChecksumMismatch);
                }
            }
            tokio::fs::create_dir_all(&self.runtime_dir).await?;
            tokio::fs::set_permissions(&self.runtime_dir, std::fs::Permissions::from_mode(0o700))
                .await?;
            let nonce = format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            );
            let directory = self.runtime_dir.join(&nonce[..16]);
            tokio::fs::create_dir(&directory).await?;
            tokio::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).await?;
            let run_directory = RunDirectory(directory);
            let socket_path = run_directory.0.join("rpc");
            if socket_path.as_os_str().len() >= 104 {
                return Err(PluginError::Protocol("runtime path is too long for a Unix socket; choose a shorter runtime directory".into()));
            }
            let listener = UnixListener::bind(&socket_path)?;
            tokio::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
                .await?;
            let mut command = Command::new(&executable);
            command
                .args(&manifest.args)
                .current_dir(&run_directory.0)
                .env_clear()
                .env("PATH", std::env::var_os("PATH").unwrap_or_default())
                .env("RSTRBOT_SOCKET", &socket_path)
                .env("RSTRBOT_TOKEN", &nonce)
                .env(
                    "RSTRBOT_CONTEXT",
                    serde_json::to_string(context)
                        .map_err(|_| PluginError::Protocol("invalid context".into()))?,
                )
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .process_group(0);
            let child = command
                .spawn()
                .map_err(|e| PluginError::Spawn(e.to_string()))?;
            let pid = child
                .id()
                .ok_or_else(|| PluginError::Spawn("worker exited during spawn".into()))?;
            let mut group = ChildGroup {
                child,
                group: Pid::from_raw(pid as i32),
                armed: true,
            };
            let exchange = async {
                let (stream, _) = tokio::time::timeout(
                    Duration::from_millis(manifest.timeouts.handshake_ms),
                    listener.accept(),
                )
                .await
                .map_err(|_| PluginError::Timeout("connection"))??;
                let credential = stream.peer_cred()?;
                if credential.uid() != nix::unistd::Uid::current().as_raw() {
                    return Err(PluginError::Protocol("socket peer uid mismatch".into()));
                }
                if credential.pid().is_some_and(|peer| peer as u32 != pid) {
                    return Err(PluginError::Protocol("socket peer process mismatch".into()));
                }
                let mut wire =
                    Framed::new(stream, LinesCodec::new_with_max_length(MAX_FRAME_BYTES));
                let hello_id = uuid::Uuid::new_v4().to_string();
                send(
                    &mut wire,
                    json!({"jsonrpc":"2.0","id":hello_id,"method":"initialize","params":{
                        "protocol_version":PROTOCOL_VERSION,"token":nonce,"context":context
                    }}),
                )
                .await?;
                let hello = tokio::time::timeout(
                    Duration::from_millis(manifest.timeouts.handshake_ms),
                    receive(&mut wire),
                )
                .await
                .map_err(|_| PluginError::Timeout("handshake"))??;
                let hello = response(&hello, &hello_id)?
                    .ok_or_else(|| PluginError::Protocol("missing initialize result".into()))?;
                if hello.get("protocol_version").and_then(Value::as_str) != Some(PROTOCOL_VERSION)
                    || hello.get("instance_id").and_then(Value::as_str)
                        != Some(context.instance_id.as_str())
                    || hello.get("workspace_id").and_then(Value::as_str)
                        != Some(context.workspace_id.as_str())
                    || hello.get("generation").and_then(Value::as_u64) != Some(context.generation)
                {
                    return Err(PluginError::Protocol(
                        "handshake version or instance mismatch".into(),
                    ));
                }
                let domain = method.split('.').next().unwrap_or_default();
                if !hello
                    .get("capabilities")
                    .and_then(Value::as_array)
                    .is_some_and(|caps| caps.iter().any(|v| v.as_str() == Some(domain)))
                {
                    return Err(PluginError::CapabilityDenied(method.into()));
                }
                // Handshake/credential preparation may have outlived the
                // caller's original grant. This is the last core check before
                // the trusted worker receives an instruction with side effects.
                tokio::time::timeout(Duration::from_secs(10), authorize())
                    .await
                    .map_err(|_| PluginError::Timeout("authorization"))??;
                let request_id = uuid::Uuid::new_v4().to_string();
                send(
                    &mut wire,
                    json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":params}),
                )
                .await?;
                let deadline = tokio::time::sleep(Duration::from_millis(manifest.timeouts.call_ms));
                tokio::pin!(deadline);
                loop {
                    let frame = tokio::select! {
                        biased;
                        _ = cancellation.cancelled() => {
                            let _ = tokio::time::timeout(Duration::from_millis(50), send(&mut wire,json!({"jsonrpc":"2.0","method":"cancel","params":{"request_id":request_id}}))).await;
                            return Err(PluginError::Cancelled);
                        },
                        _ = &mut deadline => return Err(PluginError::Timeout("invocation")),
                        frame = receive(&mut wire) => frame?,
                    };
                    if let Some(notification) = frame.get("method").and_then(Value::as_str) {
                        if frame.get("id").is_some()
                            || !matches!(
                                notification,
                                "model.delta"
                                    | "tool.progress"
                                    | "platform.progress"
                                    | "task.progress"
                            )
                            || frame.pointer("/params/request_id").and_then(Value::as_str)
                                != Some(request_id.as_str())
                        {
                            return Err(PluginError::Protocol(
                                "unsolicited request or unrelated event".into(),
                            ));
                        }
                        if let Some(sender) = &events {
                            let event = RpcEvent {
                                method: notification.into(),
                                params: frame.get("params").cloned().unwrap_or(Value::Null),
                            };
                            sender
                                .try_send(event)
                                .map_err(|_| PluginError::Backpressure)?;
                        }
                    } else {
                        return response(&frame, &request_id)?
                            .ok_or_else(|| PluginError::Protocol("missing final result".into()));
                    }
                }
            };
            let result = tokio::select! {
                biased;
                _ = cancellation.cancelled() => Err(PluginError::Cancelled),
                result = exchange => result,
            };
            group.cleanup().await;
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[tokio::test]
    async fn queued_authorization_is_rechecked_before_worker_start() {
        let directory = tempfile::tempdir().unwrap();
        let host = PluginHost::new(directory.path().into());
        let occupied = host.slots.acquire_many(8).await.unwrap();
        let granted = AtomicBool::new(true);
        let checks = AtomicUsize::new(0);
        let manifest = Manifest {
            id: "fixture".into(),
            version: "1".into(),
            executable: directory.path().join("must-never-start"),
            args: vec![],
            capabilities: vec!["platform".into()],
            permissions: vec![],
            sha256: None,
            timeouts: crate::Timeouts::default(),
        };
        let context = InstanceContext {
            workspace_id: "workspace".into(),
            instance_id: "route".into(),
            generation: 1,
        };
        let call = host.call_authorized(
            &manifest,
            &context,
            ("platform.send", serde_json::json!({"text":"must-not-send"})),
            CancellationToken::new(),
            None,
            || async {
                checks.fetch_add(1, Ordering::SeqCst);
                if granted.load(Ordering::SeqCst) {
                    Ok(())
                } else {
                    Err(PluginError::Cancelled)
                }
            },
        );
        tokio::pin!(call);
        // Explicitly poll into the full semaphore queue; no timing sleep is
        // needed to establish that revocation occurs while this call waits.
        assert!(futures_util::poll!(&mut call).is_pending());
        assert_eq!(checks.load(Ordering::SeqCst), 0);
        granted.store(false, Ordering::SeqCst);
        drop(occupied);
        assert!(matches!(call.await, Err(PluginError::Cancelled)));
        assert_eq!(checks.load(Ordering::SeqCst), 1);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }
}
