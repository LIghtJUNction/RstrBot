# RstrBot 0.2.0：界面重设计与多方式登录

日期：2026-10-28（Asia/Taipei）。基线：`7d5f77a756340a9a2f616d20a6edb3aca8069a8f`。本轮实现用户提出的界面重设计、邮箱配置、Google 登录，以及多枚物理或同步 Passkey 的绑定与换绑。版本仍为开发预览；原设计包及其未完成需求继续保留。

配置入口与部署步骤见 [账户与登录指南](AUTHENTICATION.md)。这一页说明实现选择、验证依据及边界。

## 1. 新的界面结构

界面采用深炭色背景、暖白文字和低饱和金色强调，减少装饰性动效。侧栏按工作、知识、连接、工作区和账户分组；登录、聊天、数据表单与移动端导航共用间距、边框、按钮和状态样式，保留中英文与浅色主题。

新增两类页面：

| 页面 | 使用者 | 主要操作 |
| --- | --- | --- |
| 账户与安全 | 每位已登录用户；无需加入工作区 | 验证或更换邮箱、改密码、绑定 Google、管理多个 Passkey |
| 认证与邮件 | 实例管理员 | 固定公开地址与 RP ID、Google OAuth 客户端、SMTP 传输与测试邮件 |

用户可以在同一账户保留密码、Google 和多枚 Passkey。新增方式只改变身份验证入口，不改变账户、工作区、资源归属与后端授权规则。页面不会把隐藏管理按钮当成权限控制。

跨标签页共享 Cookie 换号时，当前页面先停止渲染旧账户数据，再取消和清除缓存。身份确认窗口绑定明确的操作代次；关闭、切换账户或开始另一项操作后，旧回执不能继续执行新操作。管理员的未保存配置保持自己的基准 revision，后台刷新不会悄悄覆盖草稿；保存期间锁定输入，远端修改需要显式重新读取。

## 2. 邮箱配置与恢复

管理员设置发信地址、SMTP 主机、端口、用户名、密码和 TLS/STARTTLS。线上传输校验服务器证书；明文模式只允许字面 loopback 地址，用于本机中继与测试。测试邮件走实际 SMTP 发送路径，成功仅表示中继已接受。

用户修改邮箱采用待验证状态：服务端生成高熵一次性令牌，只保存摘要；邮件包含十五分钟有效的确认链接。链接采用 URL 片段，页面要求用户主动确认，并绑定发起请求的原始登录会话。更换期间旧地址仍有效，成功后才替换。已验证邮箱可作为密码登录别名，不能与其他用户的用户名或已验证地址冲突。

忘记密码入口对未知地址、账户投递冷却或限额和正常申请统一返回接受结果；全局认证入口的速率或并发限制仍可返回 429。重置记录同时绑定发信时的邮箱版本与密码版本，成功后撤销全部旧会话和挑战；旧链接无法覆盖后来已经修改的密码或邮箱。Google 返回的邮箱显示信息不会自动变成本地找回密码地址。

邮箱验证与密码重置邮件具有账户冷却和小时/日限额；公共认证入口另外具有速率与并发限制。管理员测试邮件使用认证入口限制，不套用用户验证邮件的账户配额。SMTP/OAuth 密钥在库内使用实例密钥加密，读取接口只返回是否配置。修改密钥的接收目标时必须重新提供或明确清除，不会把原密钥转发到新的 SMTP 主机或 OAuth 客户端。

## 3. Google 登录与绑定

使用 Google OpenID Connect 的授权码流程、PKCE S256、nonce、一次性 state 和独立的浏览器绑定 Cookie。服务器验证 RS256 签名、Google issuer、audience/azp、时间范围与 nonce，再以稳定的 `sub` 识别用户。

本版保持管理员创建本地账户的模式。用户先登录已有账户，再明确绑定 Google；之后才可从登录页使用 Google 登录。相同邮箱不会触发自动合并，未知 Google 身份不会自动注册。绑定流程在跳转前及回调提交时核验原会话，状态只能消费一次。

Google 登录建立普通会话，但敏感设置仍要求最近五分钟内的本地密码或 Passkey 证明。Google 官方说明，应用不能强制要求 Google 账户重新认证，因此不能把新签发 ID Token 或新的本地会话时间冒充近期用户验证。[Google Security bundle](https://developers.google.com/identity/siwg/security-bundle)

JWT 验证使用 `jsonwebtoken` 的 AWS LC 实现；没有通过忽略 RustSec 告警来启用受影响的 RSA 依赖路径。依赖审核使用仓库现有严格策略，新依赖的 0BSD 许可已加入允许列表，告警忽略列表保持为空。

## 4. 多枚 Passkey、同步状态与换绑

Passkey 注册和认证使用 `webauthn-rs`。挑战保存在数据库，绑定用途、用户和认证配置版本；注册与再次确认身份的挑战还绑定原始会话，登录挑战不要求已有会话。挑战限时且一次性消费，多进程或重启不会把进程内状态误当成唯一依据。服务器要求用户验证，核验挑战、源地址、RP ID 和签名后才签发会话。

页面提供自动选择、安全密钥、此设备和 Android Google Password Manager 兼容模式。认证器偏好只是交给浏览器的提示；实际可用的物理密钥、手机或云端凭据由浏览器和操作系统决定。页面展示协议中已验证的备份标志，不据此猜测厂商。私钥的生成、保管与同步不由 RstrBot 承担。

多枚凭据各自保存名称、创建和使用时间。重命名不改密钥；删除使旧凭据和由它签发的会话失效。换绑先完成新凭据验证，再在同一事务中添加新项、移除旧项；提交前取消或验证失败不会先删掉旧凭据。已提交事务不能靠关闭弹窗回滚，丢失回执时需重新读取列表。

凭据版本单调增加。删除后即使相同 credential ID 再次注册，旧的认证快照也无法重新获得效力。凭据管理提交继续核验用户状态、原会话和近期认证；注册与换绑另外核对挑战记录中的认证配置版本，不能把旧配置下的验证结果提交到新配置。

登录使用“先输入用户名或已验证邮箱，再选 Passkey”。本版不提供无用户名自动填充或跨 RP 凭据迁移。实例中任一账户已有 Passkey 时固定公开源地址（含协议和端口）与 RP ID；域名调整须先安排所有相关账户的备用登录方式，移除实例全部旧凭据后再注册新站点凭据。

## 5. 数据升级与恢复

schema 1 在启动时事务升级到 schema 2，保留原有账户、工作区和业务资源。备份从 `rstrbot-db-v1` 扩展为 v2，增加邮箱、Google 与 Passkey 身份表，同时保留加密的认证配置。会话、验证挑战、邮件令牌和发送限流状态不进入备份。

v1 备份仍可导入并升级；恢复继续进入既有隔离模式。备份校验检查邮箱别名、唯一凭据、凭据大小与版本计数，拒绝矛盾身份状态。恢复加密配置仍需要对应的 `secrets.key`，不能仅复制数据库后丢弃密钥。

双数据库复用同一业务实现与验收：SQLite 和 PostgreSQL 都验证会话与凭据撤销、原子换绑、单次挑战、配置 CAS、密码恢复、schema 升级和备份导入。

## 6. 验证证据

最终结果集中在 [验证汇总](evidence/v0.2.0/verification.json)，程序与源码指纹分别保存在 [binaries.json](evidence/v0.2.0/binaries.json) 和 [build-inputs.json](evidence/v0.2.0/build-inputs.json)。所有本机 HTTP 与浏览器验收使用同一份固定 Linux x86_64 debug 核心及相邻 worker；没有在测试期间重新编译正在验收的程序。

| 验证层 | 覆盖范围 | 证据 |
| --- | --- | --- |
| Rust 严格门槛 | 95 项工作区测试；fmt、Clippy 全目标拒绝警告、Rustdoc 拒绝警告及构建 | [rust-verification.json](evidence/v0.2.0/rust-verification.json) |
| PostgreSQL 存储 | 44 项 PostgreSQL 认证/业务/身份测试，另有 5 项 SQLite 路径与只读探测单元测试 | [store-postgresql.log](evidence/v0.2.0/store-postgresql.log) |
| 实际服务矩阵 | HTTP、双数据库、Valkey 开关/故障、恢复迁移与新 SMTP 身份流程 | [integration-matrix.json](evidence/v0.2.0/integration-matrix.json) |
| 前端 | API 错误隔离、状态、权限、WebAuthn 编解码、TypeScript 与生产页面构建 | [前端回执](evidence/v0.2.0/web-verification.json) |
| 浏览器 | 原工作台流程及实际 WebAuthn 注册/登录/换绑、邮箱、草稿和身份边界 | [工作台](evidence/v0.2.0/browser.json)、[账户安全](evidence/v0.2.0/security-browser.json) |
| HTTP 契约 | 56 条路径、67 个操作、124 个 schema 与源码路由/DTO/示例核对 | [openapi.json](evidence/v0.2.0/openapi.json) |
| 依赖与原蓝图 | RustSec、许可、来源策略；原始设计 19 个文件摘要 | [依赖审核](evidence/v0.2.0/dependency-policy.log)、[蓝图完整性](evidence/v0.2.0/blueprint-integrity.json) |

矩阵中的场景执行会在不同数据库或缓存模式重复，不表示独立需求数量。远端 CI 结果以本提交的 [GitHub Actions](https://github.com/LIghtJUNction/RstrBot/actions) 实际状态为准，配置文件存在本身不等于运行通过。

### 自动化的实际边界

HTTP 邮箱测试使用真实本机 SMTP 会话和合成收件地址，不发送到真实邮箱。Google 测试使用真实签名的合成 RSA ID Token、验证器及回调失败边界；没有使用真实 Google 项目完成授权交换。Passkey 浏览器测试使用 Chromium 的 USB 与平台虚拟认证器、用户验证和备份标志；它不证明某个实体型号、手机系统或云同步服务已完成实机验收。

上线时需要用正式 HTTPS 域名、SMTP 服务和 Google OAuth 项目实际走一遍，再测试目标硬件与设备。普通 U2F 仅触摸密钥不满足独立 Passkey 登录所要求的用户验证。开发环境使用 `http://localhost`，当前库不接受 IP 形式的 RP。

本次没有新增原生桌面安装、Windows named pipe、macOS 签名、读屏、长期负载或优化发行性能验收。原验收表中的 9 项未实现需求没有因本次账号功能被改成已完成。

## 7. 复现

按 [README](../README.md) 安装锁定工具链与 OpenSSL 开发依赖，构建 WebUI 和四个 Rust 程序。测试入口：

```sh
cargo test --workspace --all-targets --locked
python3 tests/identity_integration.py --binary target/debug/rstrbot --output test-results/identity.json
node tests/browser/verify_ui.mjs --binary target/debug/rstrbot --evidence-dir test-results/browser
node tests/browser/verify_security.mjs --binary target/debug/rstrbot --evidence-dir test-results/security
```

完整服务矩阵使用 `scripts/verify_integration.py`，设置独立 PostgreSQL/Valkey 环境；不要指向生产数据库。`scripts/check.sh` 包含新身份 HTTP 回归，浏览器安装与详细参数见 [浏览器说明](../tests/browser/README.md)。
