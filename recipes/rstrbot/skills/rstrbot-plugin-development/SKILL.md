---
name: rstrbot-plugin-development
description: Use when building, reviewing, or documenting RstrBot executable plugins or language SDKs.
---

# RstrBot plugin development

## Read the contract first

- In the complete recursive recipe, locate the `rstrbot-plugin-protocol`, `rstrbot-sdk`, and `rstrbot-plugin-host` packages and read their entrypoints, authoritative contracts, and acceptance cases. These are the reconstruction inputs.
- In an existing generated implementation, also read its actual `docs/PLUGIN_PROTOCOL.md` and `sdk/README.md`; those generated files are not prerequisites for consuming this Markdown recipe.
- Choose the existing Rust, Python, TypeScript, or C# SDK. Implement the public protocol; do not depend on private core objects.
- Treat the package manifest, installed package, workspace instance, and one invocation as separate things.

## Capability and trust rules

- A capability declaration describes what a plugin requests; it does not grant permission. The core checks installed declarations and current user authority for each operation.
- Workers are trusted native programs in this release. A separate process is not a malicious-code sandbox. Do not describe it as one.
- Keep provider credentials scoped to the provider call. Ordinary tool invocations must not receive them.
- Do not add worker-initiated core-admin methods, arbitrary executable installation through workspace settings, or network listeners for the local worker protocol.
- Validate initialization context, nonce, protocol version, request IDs, frame limits, method parameters, and the single terminal response. Bound queues and event output.
- Propagate cancellation and stop child processes. Cancellation cannot reverse a remote side effect; retain an unknown/in-flight outcome for reconciliation and never silently replay it.
- Redact keys, authorization headers, private URL paths, and remote response bodies from diagnostics.

## SDK conformance

Keep the current 1.0 reference and vectors unchanged. The extension protocol's 1.1-draft, SDK capability contract, host lifecycle/compatibility contracts, and core extension API describe future negotiated capabilities. Bounded host services grant no core-admin access. Freeze the unmodified compatibility fixtures by digest, then record import, behavior, recovery, and cleanup separately; a verified source hash does not prove compatibility. Update SDK documentation and embedded tutorials with the same distinction.

Build the selected example using the pinned toolchain. In a generated implementation that provides this harness, run:

```sh
python3 tests/plugin_conformance.py --sdk <rust|python|typescript|csharp>
```

For a full local check, use `mise exec -- ./scripts/check.sh` when that script actually exists. Otherwise generate the required harness and record its real command. An unavailable runtime is `not_executed`, not a pass. Update the protocol document, SDK README, and conformance vectors together when changing shared behavior.

For 1.14 provider rejections, use the supplemental PROVIDER_ERRORS contract and four-SDK vectors. Preserve generic fallback for older hosts/workers, never expose response bodies, and do not treat an upstream 401 as local sign-out. Report SDK runtimes not actually tested.

The 1.15 generated implementation is the `generated/rstrbot-implementation/` source submodule. Verify its URL and pinned commit; recipe archives must not include implementation source or Git metadata.
