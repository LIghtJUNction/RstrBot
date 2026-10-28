---
name: rstrbot-development
description: Use when changing RstrBot core, web UI, API, persistence, deployment, or repository documentation.
---

# RstrBot development

## Start here

1. Read the root `AGENTS.md` and preserve unrelated work in the checkout.
2. Start with the root recipe entrypoint, capability coverage, acceptance harness, and affected recursive package contracts. In an existing generated implementation, also read its actual `README.md`, `docs/API_CONTRACT.md`, and relevant module docs; absent generated files are not hidden recipe dependencies.
3. Use the recorded environment contract and the generated implementation's actual `mise.toml`/locks when available. Do not silently raise or lower the supported toolchain.
4. Check `git status` and the current API contract before editing. The contract and OpenAPI schema are the source of truth for UI choices and backend behavior.

## Product and implementation boundaries

- Keep authorization, validation, and durable state enforcement on the server. A hidden or disabled UI control is not an access check.
- Do not invent resource fields, enum values, provider types, capability support, success states, or retry safety. Derive finite choices from the contract or server response; keep genuinely open text fields editable.
- Treat secrets as write-only. Never put credentials in logs, rendered summaries, links, or errors.
- Preserve the distinction between rejected, failed, unknown, and completed external operations. Do not replay a mutation after an uncertain response unless its contract provides idempotency.
- Keep English and Chinese copy in separate language branches. Use short, plain wording and light original humor only where it does not obscure risk or status.
- Prefer spacing and grouping over decorative rules. Keep keyboard focus, screen-reader names, touch targets, reduced-motion behavior, and narrow-screen layouts usable.
- Keep the root `recipes/rstrbot/BUILD_PROMPT.md` current when a user requests a product improvement; standalone generated source carries `docs/RECIPE_BUILD_PROMPT.md`, and old blueprint prompts remain historical. Record the requirement and its observable acceptance conditions without claiming it is already implemented.

## Verification

Use the smallest relevant local checks first. In a generated implementation with the corresponding script, the repository-wide local check is `mise exec -- ./scripts/check.sh`. Generate missing acceptance tools and record their actual commands; recipe material checks remain separate. Report which checks ran and their result; a successful build does not prove browser, production, or device acceptance.

Browser-local drafts and versions are private user state: scope them by instance origin and authenticated user, retain unattributed legacy records without automatic adoption, and clear stale views on session changes. Use the real read-only schedule preview. Preserve uncertain requests for record review and explicit idempotent continuation, never automatic replay. Run `tests/browser/verify_feedback.mjs` and relevant configuration regressions against an isolated instance; report real-provider and native-device acceptance separately.

Keep current v1/1.0 behavior separate from future v2/1.1-draft contracts. Use the competitive runbook and OpenClaw feedback records for fixed versions, issue status, actual downstream evidence, and unexecuted cases. Static material validation, closed historical issues, and simulated users do not prove runtime success or comparative usability. Sync generated tutorials and embedded skills whenever these contracts change.

For 1.14 provider rejections, use the supplemental PROVIDER_ERRORS contract and four-SDK vectors. Preserve generic fallback for older hosts/workers, never expose response bodies, and do not treat an upstream 401 as local sign-out. Report SDK runtimes not actually tested.
