# LangBot 用户需求证据 → RstrBot 产品目标

**采样日：2026-10-28（Asia/Taipei）。** 本研究本质上是把真实反馈转换为“用户希望什么行为、怎样才算做对”，不是继承另一个项目的内部实现。

## 范围与可信度

通过已选 GitHub 插件只读核实仓库 **[langbot-app/LangBot](https://github.com/langbot-app/LangBot)**，仓库 ID 575321313，默认分支 master；原始 [仓库 API](https://api.github.com/repos/langbot-app/LangBot) 的 updated_at 为 2026-10-27T19:13:35Z。完整读取 15 条 issue 正文及 31 条可见评论，9 条开放、6 条关闭。对关闭含义不明确的条目追踪时间线、关联 PR 或迁移后的 issue。

这是目的抽样，不能据此推断需求占比、全仓库缺陷数量或某产品总体质量。未独立运行上游故障与补丁；“报告复现”“维护者确认”“PR 合并”和“本次已实测”严格区分。本次没有实现代码、平台安装包或 GitHub CI 实测。

查询：

- `repo:langbot-app/LangBot is:issue`：按更新时间降序取前 25。
- `repo:langbot-app/LangBot is:issue multi user`：取前 10。
- `repo:langbot-app/LangBot is:issue 权限`：取前 15。
- `repo:langbot-app/LangBot is:issue 知识库`：按更新时间降序取前 15。
- 额外定向读取 #2761、#2747、#2540、#2511、#2433、#2410，再依引用补齐相关讨论。

多用户、SQLite/PostgreSQL 初始化选型及可选 Valkey 首先来自用户明确要求；本样本为具体行为和验收边界提供补充依据。没有把“搜到某条反馈”当作这个功能只属于某一项目的证据。机器人评论不当作维护者确认，其执行指令、广告及个人/会话标识均未保留。

## 对新产品最有价值的决策

- **状态要诚实。** 生成完成、结果持久化和外部平台收到消息是不同事实，页面应能解释卡在哪一步。
- **默认轻量，但不能牺牲正确性。** SQLite 和 PG 共享业务契约；Valkey 可选，权限与关键任务状态以持久存储为准。
- **统一权限，不混淆能力角色。** HTTP、MCP、插件和 CLI 复用同一操作授权；Runner、工具和监听器分别表达可用范围。
- **协议保留真实信息。** 工具结果、供应商私有字段、二进制资源及完成事件都要经过各语言 SDK 和持久化路径验证。
- **可恢复才算完成。** 文件生成后必须交付确认；外部执行结果未知时不能自动重放副作用。

以下“实现目标”和“验收”均为对 RstrBot 的设计推导，不是宣称上游已经如此实现。P0 表示首个可靠版本前需要满足的正确性边界；P1 表示首版对应功能应具备的产品能力。

## 证据索引

| ID | 主题 | Issue | 状态 | updated_at（UTC） | 优先级 |
|---|---|---|---|---|---|
| LB-01 | 执行、消费、持久化和投递状态 | [#2761](https://github.com/langbot-app/LangBot/issues/2761) | open | 2026-10-23T03:30:11Z | P0 |
| LB-02 | SQLite 与 PostgreSQL 一致的持久化语义 | [#2747](https://github.com/langbot-app/LangBot/issues/2747) | open | 2026-10-22T13:45:30Z | P0 |
| LB-03 | 按平台特性合并流式更新 | [#2540](https://github.com/langbot-app/LangBot/issues/2540) | open | 2026-09-23T14:15:48Z | P1 |
| LB-04 | 多轮工具循环与副作用去重 | [#2511](https://github.com/langbot-app/LangBot/issues/2511) | open | 2026-09-07T16:13:35Z | P0 |
| LB-05 | 平台适配器独立插件 | [#2433](https://github.com/langbot-app/LangBot/issues/2433) | open | 2026-09-04T05:08:59Z | P1 |
| LB-06 | Skill 说明、只读资源和执行解耦 | [#2410](https://github.com/langbot-app/LangBot/issues/2410) | closed | 2026-10-26T08:48:25Z | P1 |
| LB-07 | 二进制资源引用与真实消费 | [#2775](https://github.com/langbot-app/LangBot/issues/2775) | open | 2026-10-26T08:48:06Z | P1 |
| LB-08 | 配置与执行权限一致，区分能力角色 | [#2629](https://github.com/langbot-app/LangBot/issues/2629) | closed | 2026-10-19T15:15:34Z | P0 |
| LB-09 | 多用户统一应用 API 和密钥范围 | [#2639](https://github.com/langbot-app/LangBot/issues/2639) | open | 2026-10-17T06:59:34Z | P0 |
| LB-10 | 可选 Valkey 与部署级限额 | [#2241](https://github.com/langbot-app/LangBot/issues/2241) | open | 2026-06-12T20:55:25Z | P1 |
| LB-11 | 产物与备份交付可靠性 | [#2634](https://github.com/langbot-app/LangBot/issues/2634) | closed | 2026-10-23T02:59:36Z | P0 |
| LB-12 | 每轮上下文预算与长工具结果 | [#2066](https://github.com/langbot-app/LangBot/issues/2066) | closed | 2026-05-13T06:59:00Z | P1 |
| LB-13 | 供应商不透明状态的协议往返 | [#1899](https://github.com/langbot-app/LangBot/issues/1899) | closed | 2026-06-19T15:26:14Z | P0 |
| LB-14 | 知识库版本和会话刷新体验 | [#2030](https://github.com/langbot-app/LangBot/issues/2030) | open | 2026-04-29T00:53:33Z | P1 |
| LB-15 | 实例隔离与真实就绪检查 | [#2713](https://github.com/langbot-app/LangBot/issues/2713) | closed | 2026-10-27T12:56:28Z | P1 |

## 逐项需求与验收

### LB-01 · 执行、消费、持久化和投递状态

**来源：** [[Bug]: Runner 完成事件已到达仍被判超时，终态落库失败也被归为 timeout](https://github.com/langbot-app/LangBot/issues/2761)；open；更新 2026-10-23T03:30:11Z。已读完整正文及 0 条评论。

**证据与状态：** COLLABORATOR 报告预算内完成事件已进入 Host，但下游消费延迟或终态写库失败后被统一记为 timeout；正文提供受控反例与分段诊断，明确尚无产品修复。 开放；未找到完成修复证据。

**用户问题：** 用户收到答案但任务显示超时，无法判断是否可重试，副作用可能重复。

**RstrBot 实现目标（P0）：** 分别表达模型/Runner 执行、关键结果持久化、消息投递和后台消费状态；使用可信接收时刻及运行代次判定执行预算，保留原始失败阶段。

**验收：**

- 注入预算内完成但消费者延迟、真正执行超时、终态提交失败三类故障，返回不同原因；未经校验或未持久化结果不得报完整成功。
- 取消、过期代次和非法完成事件不能覆盖合法终态；重启后账本与对外状态一致。

**依赖：** Run 状态机与事件序号；可信进程间协议；事务持久化与取消 fencing。

**边界：** 不把插件自报完成时间视为可信，也不以增大超时掩盖消费堵塞。

### LB-02 · SQLite 与 PostgreSQL 一致的持久化语义

**来源：** [[Bug]: 多群并发下 AgentRun 事件落库遇到 SQLite 写锁，正确回复后仍被标为失败](https://github.com/langbot-app/LangBot/issues/2747)；open；更新 2026-10-22T13:45:30Z。已读完整正文及 0 条评论。

**证据与状态：** COLLABORATOR 报告并发多群运行中 SQLite 锁争用导致正确回包与失败终态并存；异步公平排队实验减少锁错误，但报告明确其不是产品补丁，亦未解决全部执行超时。 开放；实验结论受单 Core、诊断负载范围限制。

**用户问题：** 轻量部署遇到并发后错误，迁移数据库后又担心行为变动或丢记录。

**RstrBot 实现目标（P0）：** SQLite 单机模式为所有关键写入提供有界准入、短事务和幂等追加；PG 模式保留同一业务一致性契约。先保证事件/状态，再评估吞吐。

**验收：**

- 两种数据库执行同一并发、重复 sequence、回滚、取消、重启恢复场景；关键事件不得丢失，唯一终态可验证。
- 达到队列上限时返回可理解的忙碌状态和重试语义；记录队列等待、DB 错误和阶段耗时，不能重放模型/工具副作用。

**依赖：** 数据库初始化与迁移；Run 账本；幂等键与写入准入。

**边界：** 不把实验负载数当产品容量；进程内锁不宣称覆盖多进程；PG 不是放弃 SQLite 正确性的理由。

### LB-03 · 按平台特性合并流式更新

**来源：** [企微智能机器人适配器逐 chunk 推送导致长回答流式被拉长数倍，建议内置节流合并](https://github.com/langbot-app/LangBot/issues/2540)；open；更新 2026-09-23T14:15:48Z。已读完整正文及 1 条评论。

**证据与状态：** 报告者称逐 chunk 推送拉长企微长回答；第三方评论指出累计快照每次重发造成额外流量并提出节流，但明确没有真实速率限制证据，示例字节量非实测。 开放；评论称拟提交修复，不能视为已合并。

**用户问题：** 模型很快，聊天平台仍缓慢刷屏或长时间显示正在输入。

**RstrBot 实现目标（P1）：** 平台适配器声明增量/完整快照、编辑及频率能力；合并可替代的中间更新，保留首段、最终正文和可见失败。

**验收：**

- 用固定 token 序列和慢消费者模拟，最终正文完整；中间更新数量受配置上限约束，首段与结束帧无不必要排队。
- 模型生成延迟、排队和平台发送延迟分别显示；断连后不会重复发送已确认消息。

**依赖：** 流式事件契约；适配器能力声明；投递状态与去重。

**边界：** 不把评论的 0.3 秒当规范最佳值；按真实平台限制和测试决定默认值。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2540#issuecomment-5796464584)。

### LB-04 · 多轮工具循环与副作用去重

**来源：** [[Bug]: LangBot Models 的 Claude 模型在工具执行成功后重复发起 tool call](https://github.com/langbot-app/LangBot/issues/2511)；open；更新 2026-09-07T16:13:35Z。已读完整正文及 2 条评论。

**证据与状态：** MEMBER 提供同一 Run 内反复查询/发送欢迎消息、绕过调度直接请求仍复现的报告，并在后续评论确认模型网关问题。 开放；有维护者归因，没有关闭/修复验收结论。

**用户问题：** 一次触发造成多次通知或付费操作，用户不知道任务为何停不下来。

**RstrBot 实现目标（P0）：** 模型可继续合法多步调用，但运行预算、重复循环检测和外部副作用幂等必须由运行时保障；保留 provider 原始协议诊断。

**验收：**

- 回放重复工具请求不会重复执行带同一业务幂等键的发送操作；不同合法步骤仍可继续。
- 连续无进展触发明确停止状态；用真实选定供应商验证多轮历史和工具结果往返。

**依赖：** 工具能力与副作用声明；Run/调用标识；模型供应商插件。

**边界：** 不把“工具成功即强制结束”当通用解法；重复内容不等于全部重复调用都应丢弃。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2511#issuecomment-5560044477)、[2](https://github.com/langbot-app/LangBot/issues/2511#issuecomment-5573275765)。

### LB-05 · 平台适配器独立插件

**来源：** [[Feature]: 适配器能不能做成插件那种形式 现在集成在源码里面不是很方便](https://github.com/langbot-app/LangBot/issues/2433)；open；更新 2026-09-04T05:08:59Z。已读完整正文及 1 条评论。

**证据与状态：** 用户要求适配器可像插件独立安装；MEMBER 评论确认在计划中，同时说明适配器插件化有复杂度。 开放；Roadmap 标签不是已实现证明。

**用户问题：** 新平台或适配器修复必须连同本体更新，扩展与发布成本高。

**RstrBot 实现目标（P1）：** 适配器作为可安装能力插件，独立版本、配置、健康与回滚；同一授权操作可被宿主调用，也可单独启动为 MCP 服务。

**验收：**

- 安装/升级/回滚一个适配器无需重新编译 Core；崩溃不会终止其他适配器。
- 宿主模式与独立 MCP 模式对同一消息操作有一致授权、目标校验和结果状态；支持范围可发现。

**依赖：** 插件运行时与清单；统一操作契约；独立服务身份。

**边界：** MCP 是外部调用形式；不能自动获得所有平台账号权限，也不能用它替代全部内部事件协议。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2433#issuecomment-5289327996)。

### LB-06 · Skill 说明、只读资源和执行解耦

**来源：** [[Feature]: Decouple Skill activation and read-only resources from Box sandbox execution](https://github.com/langbot-app/LangBot/issues/2410)；closed / completed；更新 2026-10-26T08:48:25Z。已读完整正文及 1 条评论。

**证据与状态：** 原提案指出纯说明/只读材料被沙箱依赖阻挡；关闭评论只确认激活、固定修订、UTF-8 读取解耦，并将二进制资源转至 #2775。 closed/completed；明确是限定范围完成，二进制消费与 live E2B 未由本关闭证明。

**用户问题：** 只想读取操作指南却必须部署容器或获得代码执行权限。

**RstrBot 实现目标（P1）：** Skill 激活只加载授权说明；读取只读资源独立授权；执行脚本时才申请执行环境。整个 Run 固定资源版本。

**验收：**

- 没有沙箱时可激活并读取合法文本资源；exec/write 仍拒绝。
- 更新 Skill 不改变进行中 Run 的材料；跨 workspace/run、路径穿越和超额读取被拒绝。

**依赖：** 能力授权；不可变资源版本；Skill 包与资源读取契约。

**边界：** 文件少不代表无需执行权限；按操作能力分权，不按包中是否只有一个文件判断。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2410#issuecomment-6411932558)。

### LB-07 · 二进制资源引用与真实消费

**来源：** [[Feature]: Read binary Skill resources through authorized references without Box](https://github.com/langbot-app/LangBot/issues/2775)；open；更新 2026-10-26T08:48:06Z。已读完整正文及 0 条评论。

**证据与状态：** 后续提案要求在无 Box 时获得已激活 Skill 的图片/PDF 受控引用，并明确“拿到引用”不等于模型能消费；当前仍为能力缺口。 开放；未实现、未完成真实消费验收。

**用户问题：** 界面显示文件存在，但模型读不到，或只能依靠不受控公开 URL。

**RstrBot 实现目标（P1）：** 图片/PDF 以带授权、有效期、大小/MIME、固定修订的资源引用提供；实际供应商/附件通道显式解析，能力不匹配时给出替代路径。

**验收：**

- 沙箱关闭时用真实 PNG/PDF，从发现到取回固定字节再到模型或附件消费完整贯通。
- 撤权、过期、跨租户、旧代次和非法路径失败；不能把文件系统路径或大块 base64 当普通工具文本输出。

**依赖：** 资源存储与生命周期；供应商多模态能力；租户/Run 授权。

**边界：** 这是独立端到端目标，不能因文本资源通过就宣称二进制能力完成。

### LB-08 · 配置与执行权限一致，区分能力角色

**来源：** [流水线可保存未授权的 Agent 运行器，执行时才失败](https://github.com/langbot-app/LangBot/issues/2629)；closed / completed；更新 2026-10-19T15:15:34Z。已读完整正文及 1 条评论。

**证据与状态：** 原报告主张将 Runner 纳入扩展 allowlist；MEMBER 关闭评论明确推翻该方案：Runner 选择/执行独立于扩展 allowlist，工具/监听器/命令仍受其控制。PR #2723 已合并。 closed/completed；采用修正后的权限边界，而非原建议。

**用户问题：** 表单允许选择，运行却因另一个不直观开关失败；粗粒度开关误伤必要组件。

**RstrBot 实现目标（P0）：** Runner、工具、监听器和管理操作各自声明授权角色；配置页以服务端能力可用性解释冲突，执行时仍重新校验 workspace、安装、目标和操作权限。

**验收：**

- 有权选择 Runner 但未授予某工具时，Runner 可启动而该工具不可调用；修改授权后长任务不会继续使用已撤能力。
- HTTP、UI 与插件对相同非法配置返回同类明确错误；配置可见不代表操作已授权。

**依赖：** 身份/成员角色；能力目录；配置验证与运行时校验。

**边界：** 不把“Runner 与 allowlist 分离”理解为 Runner 没有任何授权约束。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2629#issuecomment-6270646067)。

### LB-09 · 多用户统一应用 API 和密钥范围

**来源：** [[Feature]: 抽出统一的 LangBot API layer，供 HTTP、插件、MCP、CLI 和产品内助手使用](https://github.com/langbot-app/LangBot/issues/2639)；open；更新 2026-10-17T06:59:34Z。已读完整正文及 2 条评论。

**证据与状态：** 讨论要求 HTTP/插件/MCP/CLI/助手共享契约；MEMBER 确认 workspace 范围、API key 使用统一权限定义且只能授予账户已有权限子集，并说明这不是实施验收结论。 开放；设计已确认，未证明全部入口完成。

**用户问题：** 换入口权限不同，助手或插件能越过网页限制；团队密钥难以收回。

**RstrBot 实现目标（P0）：** 可信身份与 workspace 绑定统一进入业务操作边界；用户、服务账户、API key 均检查操作及资源范围。RstrBot 明确定义创建者撤权后密钥权限收缩与共享服务账户移交。

**验收：**

- 同一调用通过 HTTP/MCP/SDK/CLI 得到一致业务结果与错误；跨 workspace、伪造身份、超额授权全部拒绝。
- 撤销成员、角色或 key 后，新调用和重新取得能力的运行不能继续扩大权限；后台任务保留明确身份和授权来源。

**依赖：** 多用户/工作区模型；角色与资源权限；版本化操作目录。

**边界：** 原讨论未规定创建者后续角色改变时 key 是否同步降权；本蓝图必须自行作出明确决策，不冒称上游已决定。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2639#issuecomment-6099160634)、[2](https://github.com/langbot-app/LangBot/issues/2639#issuecomment-6230958664)。

### LB-10 · 可选 Valkey 与部署级限额

**来源：** [[Feature]: Distributed rate-limiter algorithm backed by Valkey](https://github.com/langbot-app/LangBot/issues/2241)；open；更新 2026-06-12T20:55:25Z。已读完整正文及 1 条评论。

**证据与状态：** CONTRIBUTOR 提出跨 worker 共享限额，指出单进程内存计数不能代表整个部署的上限；建议可选 Valkey，仍为开放提案。 开放；没有实现/实测完成证据。

**用户问题：** 增加实例后同一用户获得数倍额度，或辅助组件断线导致核心功能不可用。

**RstrBot 实现目标（P1）：** 单节点无 Valkey 可完整运行；多节点明确选择共享限额/租约后端。Valkey 仅加速或协调，权威权限、预算与任务记录不能只存在于缓存。

**验收：**

- 两个进程共同竞争同一额度时总通过数不超过配置；重连/TTL/重启行为可解释。
- Valkey 失联时，普通缓存可降级；硬预算、权限和防重控制不能静默 fail-open，执行明确拒绝或数据库后备。

**依赖：** 部署模式；限额策略与故障语义；数据库权威状态。

**边界：** 不照搬提案的统一 fail-open，也不固定其 Python 客户端或窗口算法。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2241#issuecomment-4694591255)。

### LB-11 · 产物与备份交付可靠性

**来源：** [[Bug]: Box 导出静默跳过不可读文件，随后仍清理 outbox](https://github.com/langbot-app/LangBot/issues/2634)；closed / completed；更新 2026-10-23T02:59:36Z。已读完整正文及 2 条评论。

**证据与状态：** 报告指出不可读输出被当作空目录并进入清理；关闭评论称 #2703/#2641 修复并完成非 root、真实 Docker 等验收。本研究未独立重跑。 closed/completed；有带范围的修复验收说明。

**用户问题：** 任务说文件已准备好，却没有附件，原文件还可能被清理。

**RstrBot 实现目标（P0）：** 产物生成、校验、持久化、授权交付、清理为可观察步骤；区分空结果、不可读、超额和交付失败。备份恢复沿用完整清单与校验，不将部分导出视为完整备份。

**验收：**

- 注入权限不足、磁盘满、超额、网络失败时保留可恢复产物并明确失败，不提前清理。
- 非 root 运行验证输出字节、两次 Run 隔离、未选文件保留、重试投递与恢复后校验。

**依赖：** 资源存储；交付确认；备份清单与恢复策略。

**边界：** 扩展到备份是 RstrBot 的设计推导，不是该 issue 已验证备份系统。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2634#issuecomment-6356315369)。

### LB-12 · 每轮上下文预算与长工具结果

**来源：** [[Feature]: LocalAgentRunner 增加 context-window 感知的上下文管理与按需压缩](https://github.com/langbot-app/LangBot/issues/2066)；closed / completed；更新 2026-05-13T06:59:00Z。已读完整正文及 2 条评论。

**证据与状态：** 提案要求按模型窗口管理每次调用，并将长工具输出归约；关闭评论实际将开发迁到 langbot-local-agent#3，后者仍开放。 原 issue closed/completed 表示迁移；后续 issue open，不能写成已完成功能。

**用户问题：** 长对话或一轮连续调用挤爆窗口，成本上涨、重要信息被随机截断。

**RstrBot 实现目标（P1）：** 每轮调用前基于当前模型预算输入/输出；完整工具日志作为可恢复产物，模型先收到摘要与引用；压缩保留最近任务、权限和未完成工具配对。

**验收：**

- 用小窗口模型及更小 fallback 验证每轮预算，不能拆散 tool call/result 或丢任务约束。
- 超窗可进行一次有界压缩后重试；正常短会话不强制摘要，原始日志仍可授权读取。

**依赖：** 供应商模型能力；上下文与产物记录；有界重试。

**边界：** 预算估计与真实供应商计费/tokenizer 误差应可见；不要把摘要等同长期记忆或新子代理。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2066#issuecomment-4079201191)、[2](https://github.com/langbot-app/LangBot/issues/2066#issuecomment-4438206455)。

### LB-13 · 供应商不透明状态的协议往返

**来源：** [[Bug]: Gemini 无法调用工具，需要思路签名 Function call is missing a thought_signature in functionCall parts.](https://github.com/langbot-app/LangBot/issues/1899)；closed / completed；更新 2026-06-19T15:26:14Z。已读完整正文及 2 条评论。

**证据与状态：** 用户报告 Gemini 工具调用签名丢失；MEMBER 定位消息实体丢弃供应商字段。经时间线核实 Core #2265、SDK #83 均已合并。机器人早期“自行生成签名”建议不采纳。 closed/completed；修复合并已核实，未独立复测在线 Gemini。

**用户问题：** 单次聊天正常，第二轮工具调用却因中间层丢字段失败。

**RstrBot 实现目标（P0）：** 统一消息模型允许受约束的 provider 私有状态原样保存和回传，流式/非流式、SDK 和持久化路径一致；不能伪造签名或把私有状态暴露给无关用户。

**验收：**

- 供应商合约样例经过多轮工具、序列化、保存恢复和流式合并后保持所需状态；真实供应商选定模型通过集成验收。
- 不支持的扩展字段有版本/能力错误，不能静默丢弃；更换供应商不能误用旧签名。

**依赖：** 消息及扩展字段契约；供应商插件；SDK 合约测试。

**边界：** 这是历史已修复案例，不描述为 LangBot 当前未修复缺陷。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/1899#issuecomment-4750698826)。

### LB-14 · 知识库版本和会话刷新体验

**来源：** [用 dify 连接钉钉应用出现知识库内容没有及时更新](https://github.com/langbot-app/LangBot/issues/2030)；open；更新 2026-04-29T00:53:33Z。已读完整正文及 16 条评论。

**证据与状态：** 用户报告上游知识库变更后 IM 端仍回答旧内容，并反馈重启无效；后续又提到引擎不可见。机器人猜测多次变化，MEMBER 仅要求补充，根因未确认。 开放；无可验证根因或修复，作为体验需求信号。

**用户问题：** 用户不知道当前会话用了哪个知识版本，无法可靠重新开始或诊断索引。

**RstrBot 实现目标（P1）：** 显示知识源/索引修订与就绪状态；提供保留历史的会话重新开始；区分本地检索、远程检索和 provider 端会话，说明更新何时生效。

**验收：**

- 更新文档并索引完成后，新检索使用已发布版本；旧 Run 按固定版本继续或显式终止，不能混用。
- 删除/撤权资料不出现在新检索及缓存中；失败索引有原因和重试，前端区分已安装引擎与运行可用。

**依赖：** RAG 索引版本；会话生命周期；授权与缓存失效。

**边界：** 不认定缓存、Dify 或并发就是根因；此案例不支持 LangBot 全面缺乏多用户隔离的断言。

关键评论：[1](https://github.com/langbot-app/LangBot/issues/2030#issuecomment-4154434362)。

### LB-15 · 实例隔离与真实就绪检查

**来源：** [[Bug]: stdio 插件 Runtime 固定使用 5401，阻碍同机隔离运行多个 Core 实例](https://github.com/langbot-app/LangBot/issues/2713)；closed / completed；更新 2026-10-27T12:56:28Z。已读完整正文及 0 条评论。

**证据与状态：** 报告第二实例 API 健康但插件默认调试端口冲突；经时间线和 PR API 核实 #2782 已于 2026-10-27 合并并关闭该 issue。 closed/completed；修复合并已核实，未独立做跨平台实测。

**用户问题：** 开发、测试或多租户隔离实例无法共存，健康页却声称正常。

**RstrBot 实现目标（P1）：** 实例目录、socket/pipe、调试监听及子进程归属独立；不需要的调试入口默认关闭；就绪检查按核心、运行时、适配器分项展示。

**验收：**

- 同机启动两套独立实例并运行插件，路径/端口/消息/数据互不串扰；停止一套不终止另一套。
- 插件运行时断开时业务就绪降级并定位组件；Linux/macOS/Windows 均需原生执行证据。

**依赖：** 标准路径与进程生命周期；平台 IPC；组件健康与桌面 sidecar。

**边界：** 上游 PR merged 不等于 RstrBot 的 named pipe 权限、全子进程树清理和桌面包已验证。

## 状态纠偏与后续来源

不能按 issue 是否关闭来判断整个能力是否完成：

- #2410 仅关闭激活和文本资源范围；二进制实际消费仍在 [#2775](https://github.com/langbot-app/LangBot/issues/2775)。
- #2629 的原始限制建议被后续设计推翻；应依据 [关闭评论](https://github.com/langbot-app/LangBot/issues/2629#issuecomment-6270646067) 和已合并 [#2723](https://github.com/langbot-app/LangBot/pull/2723) 理解 Runner 与扩展权限。
- #2066 的关闭代表议题迁移，后续 [langbot-local-agent#3](https://github.com/langbot-app/langbot-local-agent/issues/3) 在采样时仍 open。
- #1899 的历史故障已有 Core [#2265](https://github.com/langbot-app/LangBot/pull/2265) 和 SDK [#83](https://github.com/langbot-app/langbot-plugin-sdk/pull/83) 合并修复；用于制定协议回归验收。
- #2713 已由 [#2782](https://github.com/langbot-app/LangBot/pull/2782) 合并关闭；用于设计 RstrBot 多实例及真实就绪检查，不宣称其仍未修复。

完整结构化记录见同目录 `langbot-issues.json`。仅保存必要的公开项目元数据、短摘要、来源与产品推导，没有复制长日志、用户账号、令牌或附件中的个人信息。

