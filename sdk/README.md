# Plugin SDKs

All four language SDKs implement the same JSON-RPC 2.0 protocol and use the
same `conformance.json` vectors. They do not depend on private core objects.

* Rust: `rstrbot_plugin::sdk`, used by the bundled executable plugins.
* Python: `python/rstrbot_sdk`, with metadata-only `rstrbot.plugins` discovery.
* TypeScript: `typescript/src/index.ts`, built with its checked-in npm lockfile.
* C#: `csharp/RstrBot.Plugin`, with the standalone `EchoPlugin` example.

Run `python3 tests/plugin_conformance.py` from the repository root after building
the desired examples. `--require-all` fails when a runtime or example is absent;
without it the report explicitly records missing SDKs as `not_executed`.

The harness checks the actual processes, not just matching method signatures.
Cancellation/process-tree and model stream tests are in the Rust plugin crate.
Native OS acceptance, sandboxing and legacy ecosystem compatibility are separate.
