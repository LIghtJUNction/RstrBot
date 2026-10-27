# RstrBot 0.1.2 实现交付报告

> 此文保留 0.1.2 的历史交付与验证。当前 0.1.3 的增量改动、实际对照复现和验收结果见 [0.1.3 改进报告](IMPROVEMENTS_0_1_3.md)。

设计与交付日期：2026-10-28（Asia/Taipei）。

## 交付结论

交付包含 Rust 核心、SQLite/PostgreSQL 存储、React/TypeScript WebUI、OpenAI 兼容模型插件、Webhook 平台插件、显式诊断插件、四语言 SDK、Tauri 2 桌面源码、锁文件、测试、原始蓝图与可解压运行的 Linux x86_64 release 包。内置核心与 Rust 插件运行不需要 Node.js、Python 或 .NET。

版本范围是开发预览。52 项原需求中 41 项部分实现、9 项未实现、1 项桌面源码待原生验收、1 项需求追踪完整保留。完整第二阶段产品尚未全部实现；[ACCEPTANCE.md](ACCEPTANCE.md) 逐条保留已有行为、证据和缺口。

设计包全部材料原样保留。校验清单内 19 个文件的 SHA-256 全部匹配，清单自身一并保留；BUILD_PROMPT 与验收目录的摘要另外从 Google Drive 读回核对。52 项需求、215 个原需求族、287 条原验收及 48 条 issue 研究没有删去。

## 可以实际使用的功能

| 范围 | 当前行为 |
| --- | --- |
| 初始化与认证 | 15 分钟的一次性本机初始化凭据；管理员及首个工作区原子建立；Argon2 密码校验、HttpOnly Cookie / Bearer 会话、本机密码恢复、受保护配置与密钥文件。 |
| 多用户与工作区 | 五种角色、最后所有者保护、私有 / 工作区资源、当前成员权限检查。工作区所有者无法通过普通资源、事件或导出接口读取其他成员私有内容。 |
| 双数据库 | SQLite 与 PostgreSQL 共用存储和业务语义；资源版本 CAS、顺序事件、操作凭据、预算、备份及恢复均持久化。已存在数据库的连接探测只读，并要求可识别的版本标记。 |
| 聊天及执行 | 保存完整历史、分页读取、模型输入投影、按游标读取流事件、结构化只读工具调用与续轮、取消、重置代次、编辑分支、分批运行恢复和稳定请求去重。WebUI 使用持久事件轮询；供应商插件支持 HTTP/SSE。 |
| 模型 | 独立 OpenAI 兼容 HTTP 插件，实际模型连接测试，供应商字段与工具 ID 保留，模型切换时去除不兼容字段；另有清楚标注为非 AI 的 diagnostic 测试提供方。 |
| 用量与预算 | 模型调用前事务预留，实际结果到达后结算；已知费用与未知用量分开，未知预留不自动归零。字节估算预留不被宣传为供应商账单的绝对硬上限。 |
| 插件 | 安装包、工作区实例和执行进程分离；受信任本机 manifest 安装及可执行文件摘要，Unix 私有 socket、握手、消息上限、超时、取消及进程组清理。 |
| 知识与记忆 | UTF-8 文本 / Markdown 原文导入、Unicode 分块、先授权后词法检索、来源引用及删除；显式可编辑、固定、停用和归档的长期记忆。 |
| 计划任务 | 一次性、固定间隔、IANA 时区日历计划，数据库认领，`skip` 错过策略，运行时复查权限，恢复后不重放未知任务。 |
| 出站投递 | 实例管理员批准持久 Webhook 目标，成员显式发送，原登录凭据与当前授权复核，独立私有回执。`accepted`、`rejected`、`unknown` 和 `not_sent` 分开；同键重试不再次发送，未知结果须核对。 |
| 独立 MCP | Webhook adapter 可通过 `--mcp-stdio` 独立启动；目标范围固定，意图写盘后再发送，重启后不重放未知操作。 |
| 可选 Valkey | 默认关闭；目前只提供可丢弃的在线状态缓存和实际可用性检查。权限、费用、运行与任务事实保存在数据库中。 |
| WebUI | 中文 / 英文、深色 / 浅色、工作区切换、聊天及运行、知识 / 记忆 / 任务、模型 / 插件 / 渠道、成员 / 预算 / 审计 / 备份；原创折线粒子欢迎页。 |
| 接口规范 | 完整描述当前 37 条路径、46 个操作（44 个业务 API 加两个健康检查）、96 个 schema；`GET /api/v1/openapi.json` 提供同一静态 OpenAPI 3.1 文档。 |

`accepted` 表示 HTTP 接收端接受请求，不表示最终收件人已经阅读。插件进程隔离提供故障边界；当前管理员安装的插件是可信代码，没有宣称恶意代码沙箱。

## 当前版本实际验证

本轮只引用 [v0.1.2 的回执](evidence/v0.1.2/)。核心 release SHA-256：`8e4fc6c5dde77008dd13c18902eb932da7330b86a4c65f2e956ea0405d8b7af2`。源码与最终 WebUI 输入见 [build-inputs.json](evidence/v0.1.2/build-inputs.json)，实际二进制和脚本指纹保存在各回执中。历史回执不累计为本版通过数量。

| 检查 | 实际结果 | 证据与边界 |
| --- | --- | --- |
| Rust 工作区 | 63 项通过，0 失败，0 忽略；格式、全目标严格 Clippy、严格 Rustdoc、release 构建通过 | [Rust 回执](evidence/v0.1.2/rust-verification.json)；Linux 本机 |
| PostgreSQL 存储 | 23 项同套业务测试通过；另有 5 项 SQLite 路径/只读探测单元测试 | [存储回执](evidence/v0.1.2/store-postgresql-verification.json)；独立临时 schema |
| HTTP / 数据库 / 缓存 / 恢复 | 22 组全部通过，合计 184 次场景组执行 | [完整矩阵](evidence/v0.1.2/integration-matrix.json)；跨配置重复场景不代表独立需求完成率 |
| 持久流式记录 | 3 条同时执行的真实 HTTP/SSE 流，150 个片段完整落库；重启后一致且没有再次请求上游 | [流式回执](evidence/v0.1.2/stream-persistence-sqlite.json)；本地协议夹具 |
| 大历史恢复 | runs、deliveries、schedules 各 10,025 条历史记录后仍可重启；扫描到末尾未完成记录并保留原意图 | [重启回执](evidence/v0.1.2/history-recovery-sqlite.json)；容量回归，不是固定硬件性能承诺 |
| 排队投递撤权 | 两种数据库均通过注销、改密、到期、已在途结果四组检查 | [SQLite](evidence/v0.1.2/delivery-session-revocation-sqlite.json) / [PostgreSQL](evidence/v0.1.2/delivery-session-revocation-postgres.json)；真实八槽队列及本地 HTTP 接收器 |
| 四语言 SDK | Rust / Python / TypeScript / C# 各 5 项，共 20 项通过 | [协议回执](evidence/v0.1.2/sdk-conformance.json)；相同五项向量，不等于全部能力等价 |
| 独立 Webhook MCP | 7 项真实进程与本地 HTTP 检查通过 | [MCP 回执](evidence/v0.1.2/mcp-stdio.json)；包括目标限制、重启与不重放 |
| 前端 | 14 项状态与权限测试、严格 TypeScript、Vite 构建通过 | [前端回执](evidence/v0.1.2/web-tests-build.json)；Node 24.22.1 |
| Chromium | 14 组真实操作通过；初始化、工具续轮、草稿、分页、私有隔离、路由停用、降权、深浅色和中文 320px | [浏览器回执](evidence/v0.1.2/browser.json) / [实际截图](screenshots/v0.1.2/)；未代表读屏、全部浏览器或原生桌面 |
| OpenAPI | 37 条路径、46 个操作、96 个 schema 的静态契约检查通过 | [接口回执](evidence/v0.1.2/openapi.json)；含 DTO、引用与示例检查 |
| 原设计完整性 | SHA256SUMS 列出的 19 个文件全部逐字节匹配；校验清单自身也保留 | [设计完整性](evidence/v0.1.2/blueprint-integrity.json) |

测试使用明确标注的 diagnostic 进程与本地 OpenAI 兼容协议服务、Webhook 接收器。没有使用真实收费模型或聊天平台账户；不能据此声称商业模型质量、账单或真实平台接入已验证。Linux 原生桌面安装、Windows/macOS 安装升级、长期负载和全量可访问性没有本轮通过回执。GitHub Actions 的远端结果另行读取，不把本地通过改写为远端 CI 通过。

## 本轮关键实现与修正

1. **持久流记录继续推进授权查询。** SQLite 只有一个业务连接时，暂停的授权查询可能持有它，使流片段保存等待超时。现在同时推进保存和授权检查，撤权依旧优先；真实三条并发流的 150 个片段已验证完整、顺序正确且重启可恢复。
2. **历史总量不再阻止启动。** 内部恢复使用 `(created_at,id)` 游标与首批上界，每页最多 128 条并以 16 MiB 存储 JSON 预算提前分批，处理前释放数据库读取连接。全部 run、delivery、schedule 恢复成功后才绑定 HTTP 监听，避免把新受理任务误当崩溃遗留。
3. **排队发送继续受原凭据约束。** HTTP Webhook 发送携带仅在内存的会话租约，在获得进程槽位后及握手后业务 RPC 前重新核验会话、工作区角色、路由版本/状态和网络策略。退出、改密、到期后不能借原队列继续首次发送；已进入外部阶段的结果保留真实回执或 `unknown`，不自动重放。
4. **前端跟随真实角色。** 实例管理员身份不能替代工作区角色。成员读取他人的共享会话时没有写入口；管理员降权后退出管理页并清缓存。工作区管理员可停用已有批准路由，但不能批准新目标或重新启用它。
5. **初始化与聊天细节。** SQLite/PostgreSQL 选择始终可见；选择尚未应用时不能误用旧数据库建立管理员。保留阅读位置、补载历史锚点和草稿，普通数值字段不再被错误转成日期。
6. **验收覆盖交付程序。** 所有本轮 HTTP、SDK、MCP 与浏览器结果使用固定的 0.1.2 release 二进制，并核对执行前后的摘要。矩阵 runner 验证候选、测试脚本与回执指纹；旧版失败记录保留为历史，不能复用为本版通过。
7. **修复 Windows 默认 SQLite 文件 URL。** 首轮远端 CI 发现，直接拼接文件路径会把盘符和文件名特殊字符误当作 URL 结构。默认路径和测试夹具现在使用同一完整路径编码，保留实际文件名；用户提供的合法 URL 不变。旧失败配置不会自动重写，需保留原数据库并显式纠正 `database_url`，不能删除数据库重建。初轮失败回执保存在 [github-actions-initial.json](evidence/v0.1.2/github-actions-initial.json)，最终 Windows CI 是否通过以修复后实际运行结果为准。

此前实现的共享上下文过滤、私有代理快照保护、计划任务所有权、密钥与目的地绑定、未知费用冻结、原子会话受理及版本 CAS 继续保留。对应回归纳入本轮验收。详细变更见 [CHANGELOG.md](../CHANGELOG.md)。

## Linux 运行包使用

当前运行包是 **release / optimized 构建的开发预览**，文件名与 `MANIFEST.json` 明确标注。优化编译不改变开发预览的功能范围，也不代表完整生产验收。它适用于与本次验证兼容的 Linux x86_64 / glibc 环境；其他架构、Alpine/musl 或较旧 glibc 应在目标环境重新构建。它不作为性能测量或签名桌面安装器。

解压后保留四个可执行文件与相邻 `web/` 目录，在解压目录执行：

```sh
chmod u+x rstrbot rstrbot-openai rstrbot-diagnostic rstrbot-webhook
./rstrbot --data-dir "$HOME/.local/share/rstrbot"
```

打开 <http://127.0.0.1:8787>。从本机终端读取 `bootstrap.token`，在 15 分钟内完成管理员和首个工作区初始化：

```sh
cat "$HOME/.local/share/rstrbot/bootstrap.token"
```

在“模型”中新建 OpenAI 兼容连接并填写自己的实际供应商信息，或选择“诊断测试（非 AI）”检查本机执行链路。首次诊断模型 ID 可填 `diagnostic`。启用 diagnostic 插件实例后，可用 `/diagnostic/tool hello` 检查工具续轮。

远程使用应配置 HTTPS 反向代理。`--secure-cookie` 只改变 Cookie 属性，不为核心 HTTP 端口提供 TLS。私网模型或 Webhook 要求实例管理员显式设置相应开关。操作系统数据根、连接切换、配置开关关闭、备份恢复与密码救援详见 [OPERATIONS.md](OPERATIONS.md)。

## 明确未交付或未验证的范围

- 旧插件 / 旧 HTTP / 旧界面兼容，OpenCode、Pi 加载器，没有实现或真实样本验收。
- 双向聊天平台、群聊回复策略、多机器人、完整主动多目标路由，当前仅有 Web 对话与有限 Webhook 出站。
- 通用副作用工具审批、权限递减子代理、协作任务空间、任意代码沙箱、通用 MCP client，尚未交付。
- PDF / Office / URL 解析、向量嵌入、重排、图片 / 音视频 / 文件附件、可恢复媒体上传，尚未交付。
- 滚动摘要、完整自动长期记忆、可视化工作流及外部工作流插件，尚未交付。
- 独立服务身份、单独限权的个人 API Key、OIDC / Passkey、外部账号验证和完整凭据代理 / 轮换，尚未交付。
- 数据库快照不包含 `secrets.key`、外部插件状态或附件；登录会话令牌不导出。它不等于完整应用灾备，恢复原密钥和外部状态须单独处理。
- Tauri 2 工程仅完成源码和资源校验；没有已验证 Linux / Windows / macOS 桌面安装器、签名、升级或卸载结果。Windows 本机插件 transport 尚不支持。
- 浏览器已完成 Chromium、深浅主题和中文 320px 的指定流程与实际截图；200% 缩放、读屏、Firefox / Safari、长时间负载、磁盘满、容量和固定硬件性能尚未验收。
- 仅支持单核心部署；PostgreSQL 不意味着已经支持多个核心副本并发运行。

上述缺口保留在 52 项矩阵中，便于下一轮按同一编号继续实现。没有用空返回、恒成功适配器或修改原需求来掩盖未完成能力。

## 交付位置与版本识别

源码上传目标：[LIghtJUNction/RstrBot](https://github.com/LIghtJUNction/RstrBot)。本轮运行包、源码 ZIP、Git bundle、校验和及解压启动回执：[RstrBot-0.1.2-20261028](https://drive.google.com/drive/folders/1spSV5Rp7Akc1t08dthKUKCrv54GBy4Vd)。

运行包 `MANIFEST.json` 记录版本、实际源码提交和每个文件的 SHA-256；发行目录的独立验证文件记录 ZIP 摘要、脱离源码目录的解压启动、相邻插件调用与退出结果。源码 ZIP 从最终 Git 提交导出。最终 GitHub 提交、CI 状态与上传读取回执以发行清单及对话中的实际链接为准。

原始构建要求：[BUILD_PROMPT.md](blueprint/BUILD_PROMPT.md)。接口与运行方式：[API_CONTRACT.md](API_CONTRACT.md) / [OPERATIONS.md](OPERATIONS.md)。插件协议：[PLUGIN_PROTOCOL.md](PLUGIN_PROTOCOL.md)。
