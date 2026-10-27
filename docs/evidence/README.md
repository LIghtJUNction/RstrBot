# RstrBot 验证证据

## 当前交付：0.1.2

本轮固定 Linux x86_64 release 的证据位于 [v0.1.2/](v0.1.2/)。详细范围与限制见 [实现报告](../IMPLEMENTATION_REPORT.md) 和 [52 项追踪](../ACCEPTANCE.md)。

- [Rust 严格门槛](v0.1.2/rust-verification.json)：63 项测试，格式、Clippy、Rustdoc、release 构建。
- [PostgreSQL](v0.1.2/store-postgresql-verification.json)：23 项业务行为，另有 5 项 SQLite 路径/只读探测单元测试。
- [真实服务矩阵](v0.1.2/integration-matrix.json)：22 组，184 次场景组执行，全部通过。
- [流片段持久化](v0.1.2/stream-persistence-sqlite.json)、[大历史重启](v0.1.2/history-recovery-sqlite.json)、[排队发送撤权](v0.1.2/delivery-session-revocation-sqlite.json)。
- [四语言 SDK](v0.1.2/sdk-conformance.json)：相同五项向量，各 5 项通过。[独立 MCP](v0.1.2/mcp-stdio.json)：7 项通过。
- [前端](v0.1.2/web-tests-build.json)：14 项状态/权限测试、严格类型检查与构建。[浏览器](v0.1.2/browser.json)：14 组实际流程。
- [OpenAPI](v0.1.2/openapi.json)、[设计完整性](v0.1.2/blueprint-integrity.json)、[输入摘要](v0.1.2/build-inputs.json)。

每个执行回执保留其适用程序与脚本摘要。测试使用合成账户、明确的 diagnostic 提供方和本地 HTTP 服务，未调用商业模型账户。通过场景不能累计为完整需求完成率。原生桌面安装、三平台生产运行和真实外部账号没有本轮通过证明。

## 历史证据

根目录和 [v0.1.1/](v0.1.1/) 保存此前实现及失败调查。特别是 0.1.1 的 `integration-matrix.json` 实际为 failed；旧报告中的通过描述不能覆盖该事实，也不作为 0.1.2 的结果。包含 before/initial 的记录用于说明修复前状态。历史截图保留在 `docs/screenshots/`，本轮截图单独位于 `docs/screenshots/v0.1.2/`。

## 复现

从干净源码运行 `bash scripts/check.sh`。准备独立 PostgreSQL/Valkey 后使用 `--full-matrix`，环境变量要求见脚本。`scripts/verify_integration.py` 可以对已构建的固定 release 程序执行同一矩阵。浏览器步骤见 `tests/browser/README.md`。CI 运行情况必须读取实际 GitHub Actions，不由配置文件存在推断。

初轮 GitHub CI 的 Windows 文件 URL 失败保留在 [github-actions-initial.json](v0.1.2/github-actions-initial.json)。本地第一次复跑因受限环境不允许绑定测试套接字而失败的日志保留为 `cargo-test-sandbox-blocked.log`；获准执行真实本机服务测试后的 `cargo-test.log` 为最终结果。最终远程 CI 状态以交付文件 `release.json` 中对应提交的运行链接为准。
