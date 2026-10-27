# Rust SDK

The public Rust SDK is the `rstrbot-plugin` crate's `sdk` module. It is the same
implementation used by the bundled providers, not a duplicate wire stack.

Add `rstrbot-plugin = { path = "../../crates/rstrbot-plugin" }`, `serde_json`, and
`tokio` to a plugin package. Implement an async function accepting
`(String, serde_json::Value, sdk::PluginCall)` and returning
`Result<serde_json::Value, sdk::RpcFailure>`, then call
`sdk::serve(&["tools"], handler).await`.

The compiled `rstrbot-diagnostic` binary contains the minimal read-only `echo`
tool example and is included in the shared conformance harness.
Cancellation is exposed as `call.cancellation`; streaming uses
`call.emit("tool.progress", value).await`. Never infer authority from untrusted
method parameters. `call.context` is assigned by the host.
