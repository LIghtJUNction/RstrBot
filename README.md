# RstrBot

面向个人与小团队的私有 AI 工作区：在自己的服务器上管理对话、模型、工具、知识、记忆和计划任务。Rust 核心提供统一业务 API；React / TypeScript WebUI 与模型、工具、平台插件通过明确协议协作。

**当前版本 0.1.3 是开发预览。第二阶段蓝图包含的完整产品范围尚未全部实现或通过验收。** 本仓库保留原始设计、52 项需求以及 215 个需求族 / 287 条原验收映射；已实现范围与缺口见 [验收追踪](docs/ACCEPTANCE.md)。本版修复认证撤销竞态、模型与工具派发时的授权检查，以及跨页面迟到响应与拒绝后缓存恢复问题。改动、对照复现和实际验证见 [0.1.3 改进报告](docs/IMPROVEMENTS_0_1_3.md) 与 [版本变更](CHANGELOG.md)；旧回执仍只代表各自记录的版本。

## 0.1.3 改进

- **改密码不会被较早请求反悔。** 登录签发与改密提交都原子复核实际验证过的密码版本；改密、恢复密码或禁用账户后，已通过旧密码校验但尚未提交的请求不能再签发会话或覆盖新密码。昂贵的密码计算仍在事务外执行。
- **排队的模型和工具调用继续受当前权限约束。** 获得 worker 槽位后、握手完成并发送业务 RPC 前都重查授权。模型页面的真实连接测试也绑定原登录凭据；注销、改密、过期或管理员降权会阻止后续派发并取消相应在途测试。已发生的调用保留已知费用或未知状态，不自动重发。
- **离开聊天页后，迟到响应不再影响新视图。** 页面或工作区切换使旧异步操作失效；只创建了空会话且尚未发送消息时，不会在离开后继续触发模型请求。已经提交的聊天仍可在原会话完成，其回执不能清掉新视图的草稿。
- **被拒绝的缓存内容不会因网络错误重新出现。** 明确的未认证、无权限或不存在响应会撤下旧资源；后续网络错误不能恢复这些旧内容，必须再次成功取得授权数据。普通断网不自动删除草稿。

新增旧密码竞态和真实模型测试撤权回归，并扩展真实浏览器流程；双数据库与可选缓存继续使用共同集成矩阵。测试数量及通过状态以 [0.1.3 回执](docs/evidence/v0.1.3/) 为准，不代表完整设计包已经完成。

## 交付内容

| 内容 | 用途与当前边界 |
| --- | --- |
| Rust 服务与三个独立 worker | Linux 上可运行的核心、OpenAI 兼容模型插件、diagnostic 测试插件和 Webhook 出站插件 |
| WebUI | 初始化、登录、多工作区、聊天及模型/插件/知识/记忆/计划任务/成员/预算/备份页面 |
| SQLite / PostgreSQL 存储 | 使用共同的权限、资源、事件、预算与恢复逻辑；Valkey 为可选缓存 |
| 四种语言 SDK | Rust、Python、TypeScript、C# 源码，沿用相同协议向量，0.1.1 有四语言 Linux 通过记录；历史构建的执行结果以对应回执为准；新版本 CI 重新执行共同协议向量 |
| Tauri 2 桌面工程 | 本机 sidecar 与远程 HTTPS 工作区源码；本次没有已验证的桌面安装器，Windows 本机插件路径尚未支持 |
| 设计与验证资料 | 原蓝图、API/插件协议、运维说明、CI 配置、测试脚本与实际测试回执 |

已提供的执行链路包括模型流输出、只读工具调用和续轮、取消/撤权、运行事件恢复、UTF-8 文本/Markdown 检索、显式长期记忆、一次/间隔/时区计划任务、受管理员授权的 Webhook 投递及持久回执。模型费用缺失时保留“未知”；发送结果不确定时保留 `unknown` 并禁止自动重放。

52 项需求中仍保留 **9 项未实现**：旧插件兼容（RB-020）、旧 HTTP/数据/界面兼容（RB-021）、OpenCode/Pi 加载（RB-022）、群聊策略（RB-027）、子代理（RB-030）、技能资源与沙箱（RB-034）、媒体与文件传输（RB-037）、可视化编排（RB-048）、外部工作流与搜索（RB-049）。此外，通用副作用工具审批、向量索引等部分实现需求中的完整能力也尚未交付。当前本机插件是管理员安装的可信程序，进程隔离不等于恶意代码沙箱。

## 使用 Linux 运行包

解压对应版本的 `RstrBot-<版本>-linux-x86_64-<构建类型>.zip` 完整运行包，保留四个可执行文件彼此相邻，以及相邻的 `web/` 目录。它是核心服务与 WebUI 的可执行归档，不是原生桌面安装器。运行已构建的 Rust 核心和这三个 worker 不需要安装 Node.js、Python 或 .NET。需要可写且仅自己可访问的数据目录、正常的系统网络配置，以及匹配架构的 glibc 运行环境。

运行包面向 Linux x86_64；准确的构建环境、二进制摘要和执行范围见随包 `MANIFEST.json` 与对应版本回执。GitHub Actions 的 Browser acceptance 工作流提供 `linux-debug-development-preview` 调试归档；调试归档不是优化发行版。0.1.1 的历史验证环境为 glibc 2.39；这条历史记录不替代当前版本的目标环境验收。其他发行版、较旧 glibc、ARM 或 Alpine/musl 应在目标环境重新构建与运行验收；不能把一个 Linux 包视为所有 Linux 环境的安装证明。调试构建与优化构建由文件名及 `MANIFEST.json` 的 `profile` 区分，调试包不作为性能验收结果。

在解压目录执行：

```sh
chmod u+x rstrbot rstrbot-openai rstrbot-diagnostic rstrbot-webhook
./rstrbot --data-dir "$HOME/.local/share/rstrbot"
```

打开 <http://127.0.0.1:8787>。首次启动需要本机初始化凭据；在另一个终端读取：

```sh
cat "$HOME/.local/share/rstrbot/bootstrap.token"
```

将凭据填入初始化页面，创建管理员账户和第一个工作区。凭据有效期为 15 分钟，尚未初始化时重新启动会生成新凭据；它不写入访问 URL。完成初始化后使用账户登录。服务在前台运行，Ctrl-C 请求正常退出。

数据根可以通过 `--data-dir /absolute/path` 单独指定。配置、数据库和密钥均保存在数据根，升级程序文件时保留这份数据。一个数据根同时只允许一个核心或维护命令，已经启动服务时执行备份/恢复等本机命令应先停止服务。

### 接入第一个模型

在“模型”中新建配置，选择 **OpenAI 兼容**，填写供应商的 API base URL、实际模型 ID、API Key 和输出上限。API Key 写入后加密保存，普通读取只返回是否已配置。模型页面的“测试”会真的调用配置的供应商，可能产生费用。供应商、价格与调用权限由部署者配置；本包不包含外部服务账户或密钥。

如果只想检查本机链路，选择 **诊断测试（非 AI）**，模型 ID 可填 `diagnostic`，不需要外部 API Key。它返回明确标注的确定性测试内容。要检查工具续轮，在“插件”中新建并启用 `diagnostic` 包的工作区实例，然后在聊天中发送 `/diagnostic/tool hello`。这个结果只证明执行链路，不代表真实模型质量或商业端点已经连通。

默认只允许公开 HTTPS 模型端点。本地模型需要实例管理员同时启用 `--allow-private-network` 和该模型的私网访问选项。普通工作区成员不能通过模型参数扩大网络访问范围。

更换模型供应商或完整 API 地址时，必须重新填写密钥，或明确勾选“清除已保存密钥”。已有密钥不会自动转发到新地址；私网访问授权同样绑定实例管理员批准的供应商和目标。Webhook 的保存令牌遵守相同规则。

### 开始使用

- 在聊天页选择模型，创建会话并发送消息；进行中的运行可停止。重置保留历史并开始新的上下文代次，分支会建立新会话。
- 在“知识库”导入 UTF-8 文本或 Markdown。当前检索为词法检索，引用显示来源片段；PDF/Office、图片和音视频尚不支持。
- 在“长期记忆”显式保存可编辑内容，按需停用或归档。它不会自动把全部聊天写成永久记忆。
- 计划任务需要先有模型与结果会话。一次任务使用绝对时间，日历任务使用指定 IANA 时区；错过的任务按 `skip` 策略处理。
- 实例管理员创建账户，再由有权限的工作区成员添加角色。私有资源按创建者及当前成员资格授权，工作区所有者不会自动获得其他成员的私聊内容。
- 管理员批准并启用 Webhook 路由后，可在“机器人与渠道”中显式发送，并查看自己的投递历史。发送请求使用固定幂等键；不确定时保留原操作。`accepted` 只表示目标 HTTP 服务接受；不表示接收人阅读，也不声明目标支持幂等。接口与回执语义见 [HTTP 契约](docs/API_CONTRACT.md)。

共享会话只自动检索共享知识、共享记忆和共享工具实例，也只能选择共享代理。私有会话仍可使用本人获准的私有资料。共享计划任务可供成员查看，只有任务所有者能改变其执行配置；有管理权限的人可以删除共享任务。退出登录、凭据过期或修改密码会取消相应凭据发起的对话运行；独立授权的计划任务继续检查所有者当前权限，不依赖某次浏览器登录。

## 从源码构建

仓库提供 `rust-toolchain.toml`、`mise.toml`、`Cargo.lock`、WebUI 和 TypeScript SDK 的 npm 锁文件。Rust 工具链固定为 1.99.1，Node.js 固定为 24.22.1；开发配置中的 Python 版本与各次测试实际使用版本分别记录，不把工具配置文件当作执行证明。

已经安装相应 Rust 工具链、Node.js 24 与本机 C 编译/链接工具时：

```sh
npm --prefix web ci
npm --prefix web run build
cargo build --workspace --bins --locked
./target/debug/rstrbot --web-dir web/dist
```

构建优化版本与可执行归档：

```sh
cargo build --release --workspace --bins --locked
python3 scripts/package.py --profile release
```

打包脚本只收集已构建的程序、静态页面与说明；不会构建桌面安装器，也不应包含真实数据根、数据库、会话、密钥或个人凭据。平台名、构建类型、文件摘要以产物清单为准。

桌面是独立工作区，不会随核心构建自动安装 GTK/WebKit/Tauri 依赖。桌面准备与本机编译步骤见 [桌面工程说明](docs/DESKTOP.md)。远程连接只打开明确指定的 HTTPS 工作区，不启动本机核心；本机模式使用相同的核心程序和业务页面。

## 数据库、部署与备份

默认 SQLite 适合单核心安装。首次部署前可通过 `RSTRBOT_DATABASE_URL` 选择 PostgreSQL；初始化页面直接显示两种数据库选项，支持测试已有 SQLite 文件或空 PostgreSQL schema，连接测试只读，不执行迁移，拒绝选择已有无关 schema。当前核心实际连接仍需初始建表权限；独立迁移角色、RLS 和多核心部署还没有完成验收。

Valkey 可选。配置 `RSTRBOT_VALKEY_URL` 后会显示 `connecting`、`available` 或 `degraded` 等实际状态，缓存中断不会成为授权或预算事实来源。运行 `rstrbot check` 可检查数据库和恢复状态；这个命令明确不测试供应商连通性。

远程访问应通过 HTTPS 反向代理，把核心 HTTP 端口限制在代理能够访问的范围。`--secure-cookie` 仅添加 Cookie 安全属性，不为核心端口提供 TLS。默认 loopback 启动适合本机使用；非 loopback 启动要求明确设置安全 Cookie。Host/Origin 转发规则与具体部署步骤见 [运维说明](docs/OPERATIONS.md)。

当前 `--secure-cookie`、`--allow-private-network` 的正向设置会保存到配置；省略标志不会把已有 `true` 关闭。关闭时应先停止服务，明确编辑数据根中受保护的 `config.json`，把相应字段设为 `false` 后重新启动。`valkey_url` 同样可通过受保护配置设为 `null` 清除。不要把数据库密码、供应商密钥或初始化凭据提交到 Git。

本机数据库备份需要先停止服务，并写入新的文件：

```sh
./rstrbot --data-dir "$HOME/.local/share/rstrbot" backup \
  --output /absolute/protected/path/rstrbot-db.json
```

备份包含数据库事实，**不包含** `secrets.key`、附件或外部插件数据；需要分别受保护地保存。登录会话令牌不导出，恢复只能进入空目标并默认暂停外部执行。恢复原密钥、核对未知费用及外部投递回执后，再由管理员明确退出恢复模式；恢复不会自动重发不确定的消息。完整恢复与账户密码救援步骤见 [运维说明](docs/OPERATIONS.md)。

## 验证与支持范围

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
npm --prefix web test
python3 tests/plugin_conformance.py
python3 tests/plugin_mcp.py
python3 tests/integration.py --output test-results/integration.json
python3 tests/delivery_integration.py --output test-results/delivery.json
python3 tests/delivery_session_revocation.py --output test-results/delivery-session-revocation.json
python3 tests/stream_persistence.py --output test-results/stream-persistence.json
python3 tests/history_recovery.py --output test-results/history-recovery.json
python3 tests/shared_context_privacy.py --output test-results/shared-context-privacy.json
python3 tests/schedule_authority.py --output test-results/schedule-authority.json
python3 tests/session_revocation.py --output test-results/session-revocation.json
python3 tests/model_target_authority.py --output test-results/model-target-authority.json
python3 tests/model_test_revocation.py --output test-results/model-test-revocation.json
```

运行测试前先构建程序和 WebUI。PostgreSQL 行为测试使用一次性的测试 schema；参数与环境变量见测试脚本和 CI 配置，勿指向生产数据库。四 SDK 缺少运行时会被记录为 `not_executed`，使用 `--require-all` 才会把缺失项作为失败。核心测试不会自动验证独立的 Tauri 工程。

当前回执保存到 [docs/evidence/v0.1.3](docs/evidence/v0.1.3/)，记录环境、二进制摘要和检查结果；[docs/evidence](docs/evidence/) 中其余记录保留为历史。旧回执只适用于它记录的二进制；CI 工作流存在不代表 GitHub Actions 已运行成功。本地 HTTP 测试使用 diagnostic 和模拟上游，不能替代真实收费供应商或聊天平台账号测试。

浏览器验收入口见 [tests/browser](tests/browser/README.md)，包含真实服务下的初始化、聊天、工具轨迹、分页、身份切换与桌面/320px 页面，以及新加入的数据库选择、共享会话只读和管理员降权回归。新增迟到响应、工作区切换与拒绝后缓存恢复回归，实际结果以 0.1.3 最终浏览器回执为准。当前没有桌面安装/升级/签名回执，没有 Windows 插件 named pipe / 进程树验收，也没有读屏和全部浏览器兼容性验收。已执行与未执行的区别、全部功能缺口见 [52 项验收追踪](docs/ACCEPTANCE.md)。

## 项目导航

| 路径 | 内容 |
| --- | --- |
| `src/` | HTTP API、运行引擎、计划任务、知识处理、投递、状态与 CLI |
| `crates/rstrbot-store/` | SQLite / PostgreSQL、认证授权、事件、操作记录、预算与备份 |
| `crates/rstrbot-plugin/` | Unix worker 宿主、公开 Rust SDK 和三个独立插件程序 |
| `web/` | React / TypeScript 前端与聚焦测试 |
| `desktop/` | Tauri 2 启动器和 sidecar 资源准备脚本 |
| `sdk/` | 四种语言协议实现、示例与一致性向量 |
| `tests/` | 真实进程、HTTP、MCP、缓存恢复和投递验证入口 |
| `docs/blueprint/` | 保留原始设计、研究、52 项需求与完整需求映射 |
| `docs/` | [API 契约](docs/API_CONTRACT.md)、[插件协议](docs/PLUGIN_PROTOCOL.md)、[WebUI](docs/UI.md)、[运维](docs/OPERATIONS.md)、[桌面](docs/DESKTOP.md)、[验收](docs/ACCEPTANCE.md) |

## 许可

项目的发布许可见 [LICENSE](LICENSE)，第三方依赖及引用材料保留各自许可与来源。原始设计包和引用的研究材料不表示本项目拥有第三方项目代码或品牌。
