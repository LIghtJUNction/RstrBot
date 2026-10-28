---
name: rstrbot-development
description: Use when changing RstrBot core, web UI, API, persistence, deployment, or repository documentation.
---

# RstrBot development

## Start here

1. Read the root `AGENTS.md` and preserve unrelated work in the checkout.
2. Read `README.md`, `docs/API_CONTRACT.md`, and the relevant module docs before changing behavior.
3. Use the versions in `mise.toml`. Do not silently raise or lower the supported toolchain.
4. Check `git status` and the current API contract before editing. The contract and OpenAPI schema are the source of truth for UI choices and backend behavior.

## Product and implementation boundaries

- Keep authorization, validation, and durable state enforcement on the server. A hidden or disabled UI control is not an access check.
- Do not invent resource fields, enum values, provider types, capability support, success states, or retry safety. Derive finite choices from the contract or server response; keep genuinely open text fields editable.
- Treat secrets as write-only. Never put credentials in logs, rendered summaries, links, or errors.
- Preserve the distinction between rejected, failed, unknown, and completed external operations. Do not replay a mutation after an uncertain response unless its contract provides idempotency.
- Keep English and Chinese copy in separate language branches. Use short, plain wording and light original humor only where it does not obscure risk or status.
- Prefer spacing and grouping over decorative rules. Keep keyboard focus, screen-reader names, touch targets, reduced-motion behavior, and narrow-screen layouts usable.
- Keep docs and `docs/blueprint/BUILD_PROMPT.md` current when a user requests a product improvement. Record the requirement and its observable acceptance conditions without claiming it is already implemented.

## Verification

Use the smallest relevant local checks first. The repository-wide local check is `mise exec -- ./scripts/check.sh`. Report which checks ran and their result; a successful build does not prove browser, production, or device acceptance.
