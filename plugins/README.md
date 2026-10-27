# Bundled plugin packages

Build the executable plugins with `cargo build -p rstrbot-plugin --bins`.
The main program can discover them alongside its own executable.

| Binary | Capabilities | Use |
| --- | --- | --- |
| `rstrbot-openai` | `model` | Real OpenAI-compatible HTTP models, discovery, streaming and tools |
| `rstrbot-diagnostic` | `model`, `tools` | Explicit deterministic fixture and safe echo tool; not an AI model |
| `rstrbot-webhook` | `platform` | Real outgoing text webhook and separately scoped MCP stdio service |

All worker invocations use a private Unix socket and a fresh host identity.
Use absolute executable paths in installed manifests. Do not put secrets in
manifest arguments. New runtime executables are a host administrator operation.

See [the public protocol and SDK guide](../docs/PLUGIN_PROTOCOL.md) for request
schemas, SDK examples, cancellation, credential handling and truthful limits.
