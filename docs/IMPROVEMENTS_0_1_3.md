# RstrBot 0.1.3 改进与验证

日期：2026-10-28（Asia/Taipei）。基线：`1462be28850297484f162d86e22fdec2220dff0b`。本轮在既有 0.1.2 上修改实现，保留完整第二阶段蓝图；开发预览的功能完成分类不变。

本轮的核心问题很直接：**请求排队或计算期间，权限和页面已经改变，旧请求却仍按之前的状态继续。** 修复把校验放到真正提交、派发或写入当前页面之前，并给拒绝后的缓存保留明确状态。

## 1. 密码验证与会话签发之间的竞态

旧流程先在事务外验证密码，再开启事务签发会话。如果中间发生改密或本机恢复，清空会话后仍可能被旧登录插入一条有效会话。旧改密流程也可能在新密码已经生效之后，用较早校验成功的旧密码覆盖它。

现在把实际验证的用户 ID 和密码 hash 作为内部凭据快照。签发前执行带 `password_hash = verified_hash AND disabled = 0` 条件的用户行更新；同一短事务内读取当前用户信息并创建会话。改密也使用条件更新，然后在同一事务中撤销会话。较早的校验结果失效时返回未授权，不能覆盖新密码。

Argon2 校验和新密码 hash 计算仍在事务外运行；没有新增全局业务锁或数据库 schema。改密按用户 ID 验证，去掉了以前为验证密码而临时登录、再撤销临时会话的副作用。

四条确定性回归把操作停在“密码已验证、尚未提交”处，分别验证改密、本机恢复、迟到改密覆盖、禁用账户与当前管理员标记；不用睡眠时间猜测先后顺序。它们在 SQLite 与 PostgreSQL 上执行。

对应文件：`crates/rstrbot-store/src/auth.rs`、`crates/rstrbot-store/README.md`。

## 2. 模型和工具在实际派发时重查权限

普通运行原来主要依靠每 250ms 一次的检查，排队或握手结束后仍可能先发出 RPC。模型页面的连接测试还缺少原登录凭据租约：账号仍有管理员角色，不代表已经退出的那次登录还可以继续调用模型。

现在模型、工具发现和工具执行统一使用宿主已有的两道授权门：取得 worker 槽位后检查一次，握手完成且业务 RPC 尚未发送时再检查。模型连接测试绑定请求实际使用的 Cookie/Bearer 会话，检查当前管理员角色和模型版本，并在运行期间与返回结果前继续核验。

原版失败对照使用真实本机核心、OpenAI worker 和同步屏障控制的本机上游：八个已到上游的请求占满槽位，第九个请求排队。退出第九个请求的登录后释放槽位，旧版仍访问了上游，并返回 `success: true`、11 微美元测试费用。修复版中该请求不会到达上游；已发送请求发生注销、改密、过期或降权时会终止，缺失回执的费用保持未知，独立有效管理员仍可正常使用。测试费用和密钥均为合成夹具。

撤权不能撤回已经送到供应商的请求。实现先保存可确认的费用与回执，再决定是否向调用者返回结果；不确定的费用不自动归零，也不会自动重发。两次授权检查不能被描述为与外部供应商共同参与的原子事务。

对应文件：`src/engine.rs`、`src/api.rs`、`tests/model_test_revocation.py`。新脚本加入 `.github/workflows/ci.yml` 与 `scripts/verify_integration.py`。

## 3. 迟到页面响应与拒绝后的缓存

聊天组件在离开页面或工作区时递增自己的视图代次。已提交聊天可以在原会话完成，但旧组件的响应不能更新新视图或删除它保存的草稿。若只提交了空会话创建、还未发送聊天，离开原视图后就停止后续模型请求，保留未发送输入。取消旧读取与撤销后端已接受的操作仍是不同事情。

旧版浏览器对照真实复现了这个问题：挂住上一条聊天的 HTTP 回执，离开并重新进入页面，输入新草稿，再放行旧回执，持久草稿被清成空字符串。修复后保留原草稿。另有真实回归确认，挂起的空会话创建响应在工作区切换后不会触发 `/chat`。

资源或历史请求收到 401/403/404 时，精确查询的可展示缓存变为拒绝状态。后续 503 或网络错误、组件重新挂载都不能恢复旧内容；只有成功读取授权数据后才可恢复。迟到的聊天提交回执也不能覆盖这个拒绝状态。普通断网不会自动删除用户草稿。

浏览器回归明确等待资源 metadata 的 403 与 history 的 404 都到达，再切换为 503，随后卸载、重新挂载并恢复正常响应；列表使用同类流程，防止只验证一个缓存。

对应文件：`web/src/Chat.tsx`、`web/src/api.ts`、`web/src/data.ts`、`tests/browser/verify_ui.mjs`。

## 4. 最终验证

结果以 [本轮汇总](evidence/v0.1.3/verification.json) 与原始日志为准；所有旧版失败对照都有独立文件名，不计入修复版通过数量。最终运行使用固定的 Linux x86_64 debug 核心及三个相邻 worker，摘要逐个与集成矩阵核对。源码输入摘要见 [build-inputs.json](evidence/v0.1.3/build-inputs.json)。

| 检查 | 实际结果 | 证据 |
| --- | --- | --- |
| Rust 工作区 | 67 项通过，0 失败，0 忽略 | [cargo-test.log](evidence/v0.1.3/cargo-test.log) |
| PostgreSQL 存储 | 27 项 PostgreSQL 业务/认证回归通过，另有 5 项 SQLite 路径/探测回归 | [store-postgresql.log](evidence/v0.1.3/store-postgresql.log) |
| HTTP / 双数据库 / 可选缓存 / 恢复 | 24 组全部通过，合计 196 次场景组执行 | [integration-matrix.json](evidence/v0.1.3/integration-matrix.json) |
| 新模型测试撤权 | SQLite、PostgreSQL 各 6 项通过，已包含在上述矩阵 | [SQLite](evidence/v0.1.3/model-test-revocation-sqlite.json)、[PostgreSQL](evidence/v0.1.3/model-test-revocation-postgres.json) |
| 格式、Clippy、Rustdoc | 全目标 Clippy 与 Rustdoc 均拒绝警告；全部通过 | [格式](evidence/v0.1.3/cargo-fmt.log)、[Clippy](evidence/v0.1.3/cargo-clippy.log)、[Rustdoc](evidence/v0.1.3/cargo-doc.log) |
| WebUI | 14 项状态/权限测试、严格 TypeScript 与 Vite 构建通过 | [状态测试](evidence/v0.1.3/web-tests.log)、[构建](evidence/v0.1.3/web-build.log) |
| Chromium | 18 组真实流程通过，程序、页面与脚本摘要一致 | [browser.json](evidence/v0.1.3/browser.json) |
| OpenAPI | 37 条路径、46 个操作、96 个 schema 静态契约通过 | [openapi.json](evidence/v0.1.3/openapi.json) |
| 原设计完整性 | SHA256SUMS 内 19 个文件全部一致；BUILD_PROMPT 与提供的 Drive 当前文本一致 | [blueprint-integrity.json](evidence/v0.1.3/blueprint-integrity.json) |

196 次场景执行包含相同场景在不同数据库或缓存配置中的重复，不是 196 个独立需求。浏览器截图由测试产物与 CI artifact 保存。GitHub 的工作流定义和本地通过不等于远端 CI 成功；远端结果应查看该提交的 [Actions](https://github.com/LIghtJUNction/RstrBot/actions)。

原版失败对照：[模型测试排队撤权](evidence/v0.1.3/model-test-revocation-before.json)、[迟到响应清空草稿](evidence/v0.1.3/browser-baseline-failure.json)。

## 5. 仍保留的边界

这是可靠性改进，不改变原验收表中 9 项未实现及其余部分实现的分类；旧生态兼容、媒体、子代理、完整副作用工具审批和原生桌面等合同继续保留。没有以版本号升级代表完整设计包已经完成。

本轮使用 diagnostic 及本机 HTTP/OpenAI 兼容夹具，没有调用真实收费服务或真实聊天平台账户。没有新增优化发行包、原生桌面安装、Windows named pipe、macOS 签名、长期负载或读屏验收。桌面元数据升级不表示桌面工程已经通过原生验收。四语言 SDK 的完整新提交结果由现有远端 CI 再验证，本轮本地没有将历史 SDK 回执冒充新版本结果。

“host 已经收到成功供应商回执、正在清理进程时恰好撤权”这一特定交错本轮只有源码流程审阅，没有确定性实测；已发送但未收到回执的撤权场景有实际回归。原预算估算、恢复隔离和未知结果不重放的边界保持。

## 6. 复现入口

按 README 安装锁定工具链与依赖，先构建 WebUI 和相邻 worker。新增回归：

```sh
cargo test -p rstrbot-store --all-targets --locked
python3 tests/model_test_revocation.py --binary target/debug/rstrbot --output test-results/model-test-revocation.json
```

设置指向隔离测试环境的 `RSTRBOT_TEST_DATABASE_URL` 和 `RSTRBOT_PSQL` 后，同一脚本会创建并清理自己的 PostgreSQL schema。存储 Rust 测试的独立 schema 沿用现有诊断保留规则，使用专门的临时数据库即可在结束时整体清理。

完整矩阵使用 `scripts/verify_integration.py`；浏览器使用 `tests/browser/verify_ui.mjs`。执行最后一轮 `cargo build --workspace --bins --locked` 后冻结四个程序再启动 HTTP/浏览器检查：`cargo test` 可能因为测试依赖特性合并而重建相邻 worker，不应在真实服务验收期间改写正在核对的二进制。
