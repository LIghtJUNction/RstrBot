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

The 1.15 generated implementation is the `generated/rstrbot-implementation/` source submodule. Verify its URL and pinned commit; recipe archives must not include implementation source or Git metadata.

In 1.16, the user confirmed MaiBot as the comparison target. Default local titles must not cause an extra model request; any optional model title requires explicit user opt-in and cost/data disclosure. Check RSTRBOT-22 against the request ledger.

In 1.17, R13-OC-185423 checks optional failover with one admitted user message, real request ledgers, and genuine conflict/unknown-outcome negative cases. Missing implementation remains a gap and is not executed.

In 1.18, restore the last admitted model per conversation, require an explicit choice for missing or inaccessible models, and reject private models in shared conversations. Verify destination, restart, privacy and idempotency under RSTRBOT-24.

1.19 按 RSTRBOT-25 检查长草稿视口重排，区分末尾编辑与手动滚动阅读；模拟浏览器单路径不能代替输入法、实体手机和全矩阵验收。

1.20 按 RSTRBOT-26 核对已接纳但确认丢失后的刷新、参数/身份隔离和显式新意图；实际消息/运行/模型请求计数不可由浏览器提示代替。

1.21 按 RSTRBOT-27 测试多标签页意图协调和过期确认；不支持协调时安全降级，跨设备与未跑的竞态不得标通过。

1.22 按 RSTRBOT-28 验证知识原文仅进入授权模型上下文，不复制到新引用记录；旧引用响应去除原文。模型回答、旧数据库与备份、在途撤权不归入已解决范围。

1.23 按 RSTRBOT-29 核对资料入口、当前授权、原文版本与 UTF-8 字节位置；候选来源不代表模型已引用，失效或切换不能显示旧原文。

1.24 按 RSTRBOT-31 区分已发布索引与旧读快照；多连接、外部索引和未运行后端分别取证，不能把单实例词法通过扩成全部路径。

1.24 按 RSTRBOT-32 注入已就绪索引损坏，确认检索明确异常、模型零调用、修复可恢复，且无权和停用来源不干扰其他受众。

1.25 按 RSTRBOT-33 核对计划终态与最近运行结果，并用隔离模型账本核对失败和进程中断后的重启次数；不得用计划完成标签掩盖执行失败。
