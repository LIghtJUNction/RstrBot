# OpenClaw issue 需求研究：RstrBot 产品蓝图输入

采样日：**2026-10-28（Asia/Taipei）**。本文是需求研究，不是对参考软件的审计结论，也不表示 RstrBot 已实现或已测试。

## 范围和方法

官方仓库通过 GitHub 插件读取仓库元数据核实为 [openclaw/openclaw](https://github.com/openclaw/openclaw)，仓库 ID 1103012935，公开、未归档、默认分支 main。

进行了 8 组只读检索，返回 80 个结果行、去重 71 个候选；精选 **18 个 issue，逐一读取全文**，其中 9 个 open、9 个 closed。相关评论重点核对作者纠正、关闭原因、修复说明和维护者要求；长讨论没有逐条全文阅读。另直接读取 6 个相关 PR 元数据，核实是否合并。所有源码复现/现场运行均为来源方陈述，本次独立复现数量为 **0**。没有向外部仓库写入任何内容。

检索先用近期和高互动词发现候选，再增加标题限定避免正文命中噪声。结果不是完整清单，不能用样本估计故障率或市场份额。重复评论、推广回复和自动审查不作为独立用户数量。状态以本次 issue/PR API 读取为准，正文与旧评论不覆盖新状态。

| 查询 | 排序 | 上限 |
|---|---|---|
| repo:openclaw/openclaw is:issue multi-user | interactions | 10 |
| repo:openclaw/openclaw is:issue duplicate tool | interactions | 10 |
| repo:openclaw/openclaw is:issue cron missed | interactions | 10 |
| repo:openclaw/openclaw is:issue created:>=2026-10-01 | created | 10 |
| repo:openclaw/openclaw is:issue is:closed "cron" in:title | updated | 10 |
| repo:openclaw/openclaw is:issue "multi-user" in:title | interactions | 10 |
| repo:openclaw/openclaw is:issue "duplicate" in:title | interactions | 10 |
| repo:openclaw/openclaw is:issue "memory" in:title | interactions | 10 |

## 可直接吸收的产品结论

- **多用户必须是核心资源边界。** 私有会话、任务、附件、检索结果与密钥权限必须贯穿到后台回调和平台发送；只分 session ID 或目录名不构成隔离。证据：OC-01 至 OC-04。
- **兼容加载器不能拥有绕过权限的捷径。** 原生、旧插件、MCP、计划任务使用同一授权结果，并展示被移除能力的原因。证据：OC-05、OC-13。
- **任务成功与消息已送达是不同状态。** 去重、顺序、补跑、取消和投递不确定性必须可观察。不要许诺没有外部幂等支持的“全链路恰好一次”。证据：OC-08 至 OC-10、OC-14。
- **插件进程独立仍需错误隔离和完整回收验证。** 初始化、退出、超时、清理均可能伤及核心，必须故障注入并在三平台分别验收。证据：OC-15、OC-18。
- **SQLite 是受支持的完整模式。** 需要锁等待、WAL、容量和恢复诊断；选择 PG 也不能代替上层隔离与事务设计。可选 Valkey 不应成为任务、权限或用户内容的唯一事实来源。SQLite/PG/Valkey 的具体产品选择来自本轮用户要求；这些 issue 提供可靠性验收依据。证据：OC-09、OC-10、OC-16、OC-17。
- **保留用户意图比自动修复看起来成功重要。** 迁移不能丢静默时段/权限限制，记忆确认不能覆盖原始来源，工具收尾不能重新开放副作用。证据：OC-06、OC-11、OC-12。

以上是 RstrBot 的设计判断，不是来源项目已经作出的承诺。P0 表示首个可对外使用版本的正确性边界，P1 表示完整产品的必要能力；优先级不照搬上游标签。

## 证据条目

### OC-01 · 多用户身份和密钥权限 · P0

**来源：** [#8081 Feature Request: Multi-user permission management with role-based access control](https://github.com/openclaw/openclaw/issues/8081)  
**状态：** closed / not_planned；最后更新 2026-05-04T04:36:16Z。  
**报告内容：** 提出家庭/团队共享时区分管理员、成员、访客，并隔离凭据、配置和审计。  
**后续与关闭原因：** 状态为 closed/not_planned。末段自动审查仍要求明确共享网关信任模型；未找到证明所请求 RBAC 已实现的关闭说明。不能把关闭解释为功能完成。  
**证据等级：** 需求请求；后续审查指出部分控制不等于完整多用户边界；需求高；当前产品覆盖程度未独立复核；本次未复现。  
**RstrBot 实现目标：** 身份、工作空间成员关系、资源授权和密钥使用权由核心统一判定；允许使用供应商密钥不等于允许读取明文；平台群成员与后台管理员不能混为一类。  
**验收条件：** 用两个工作空间、三个角色遍历 Web/API/CLI/插件回调：未授权读取、修改、导出密钥或越界列举均被拒绝；审计可追到真实发起人。

核对评论：[1](https://github.com/openclaw/openclaw/issues/8081#issuecomment-4320888589)、[2](https://github.com/openclaw/openclaw/issues/8081#issuecomment-4356823263)。


### OC-02 · 后台推送和发送目标授权 · P0

**来源：** [#20305 [Bug]: message tool allows cross-user sends in multi-tenant deployments — no per-agent scoping](https://github.com/openclaw/openclaw/issues/20305)  
**状态：** closed / not_planned；最后更新 2026-03-26T04:17:21Z。  
**报告内容：** 报告多个代理共用网关时可向其他用户发送消息，并从任务元数据发现其他目标。  
**后续与关闭原因：** closed/not_planned；最后评论明确为不活跃自动关闭，未提供修复证明。报告声称做过现场审计，本次未复现。  
**证据等级：** 报告者行为证据；关闭原因已核对为 stale；需求高；漏洞是否仍存在未验证；本次未复现。  
**RstrBot 实现目标：** 回复和主动推送共享服务端目标授权；任务持久化保存发起主体、工作空间、目的地、授权范围，执行时复查撤销状态。  
**验收条件：** A 用户任务向 B 的私聊目标发送、列出 B 的任务、伪造 session_id 或通过兼容加载器绕过均失败；合法跨会话推送仍可用。

核对评论：[1](https://github.com/openclaw/openclaw/issues/20305#issuecomment-4072218129)、[2](https://github.com/openclaw/openclaw/issues/20305#issuecomment-4107863845)。


### OC-03 · 检索与记忆的数据隔离 · P0

**来源：** [#3259 [RFC] memory_search: Add user-scoped memory isolation for multi-user deployments](https://github.com/openclaw/openclaw/issues/3259)  
**状态：** closed / not_planned；最后更新 2026-03-01T04:08:10Z。  
**报告内容：** 请求 memory_search/memory_get 按真实用户过滤私有记忆，报告共享目录搜索泄露另一用户笔记。  
**后续与关闭原因：** closed/not_planned；关闭说明称优先稳定性、近期不考虑，后来仍有相同需求。不是修复结论。  
**证据等级：** 有复现步骤的功能请求；本次未复现；需求高；当前缺陷未验证；本次未复现。  
**RstrBot 实现目标：** RAG、全文/向量检索、记忆与摘要以工作空间和可见性为授权边界；过滤须在候选生成与对象读取时执行，提示词不能承担隔离。  
**验收条件：** A 的私有片段不出现在 B 的搜索结果、引用、自动补全、缓存或导出中；明确共享后的内容可检索，撤销共享后失效。

核对评论：[1](https://github.com/openclaw/openclaw/issues/3259#issuecomment-3829839404)、[2](https://github.com/openclaw/openclaw/issues/3259#issuecomment-3977049203)。


### OC-04 · 媒体附件的实际隔离 · P0

**来源：** [#186216 [Feature]: Scope staged inbound media per session when the sandbox workspace is read-write](https://github.com/openclaw/openclaw/issues/186216)  
**状态：** open / —；最后更新 2026-10-27T19:00:07Z。  
**报告内容：** 报告会话共用可写工作区时，另一线程可列出并打开旧照片；会话 ID 分开并未隔离文件。  
**后续与关闭原因：** open。作者后续撤回“共享挂载内建子目录”及“只加留存期”的方案：仍可读共享路径，且保留期影响历史回放。自动审查仅源码检查。  
**证据等级：** 报告者行为步骤＋作者收窄范围；无独立复现；需求高；当前机制有报告和源码审查支持；本次未复现。  
**RstrBot 实现目标：** 附件对象位于共享可写工作区之外；文件工具、shell、下载 URL 和历史回放均执行会话/工作空间访问控制；保留期与权限分开。  
**验收条件：** 会话 B 通过路径猜测、文件列举、shell 或下载链接均不能读 A 的私有附件；A 的授权历史回放和当前上传仍正常。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186216#issuecomment-6450871231)、[2](https://github.com/openclaw/openclaw/issues/186216#issuecomment-6450972151)。


### OC-05 · 一致的插件安装授权 · P0

**来源：** [#186189 [Bug]: plugins install from npm skips the capability review when a Gateway is running](https://github.com/openclaw/openclaw/issues/186189)  
**状态：** open / —；最后更新 2026-10-27T18:10:07Z。  
**报告内容：** 报告运行中 Gateway 安装 npm 插件只确认来源，跳过离线流程会显示的能力列表。  
**后续与关闭原因：** open。唯一自动审查指出源码存在流程差异，但管理员鉴权与制品校验仍生效；未做现场复现，也未证明未认证越权。  
**证据等级：** 报告者复现步骤＋自动源码审查；产品决定待定；需求高；差异有源码审查支持；本次未复现。  
**RstrBot 实现目标：** 所有安装入口使用同一不可变制品与能力变更计划；来源信任、能力授权、启用三者明确分开，运行时加载器亦适用。  
**验收条件：** 核心运行/停止两种状态下 CLI、GUI、API 展示相同能力变化；未批准能力不能启用；制品或能力改变使旧确认失效。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186189#issuecomment-6449913289)。


### OC-06 · 可追溯记忆与人工确认 · P1

**来源：** [#7707 Feature Request: Memory Trust Tagging by Source](https://github.com/openclaw/openclaw/issues/7707)  
**状态：** closed / not_planned；最后更新 2026-10-22T17:38:43Z。  
**报告内容：** 请求按来源标记记忆，避免网页、模型推断和第三方内容被当作可信长期指令。  
**后续与关闭原因：** closed/not_planned；末条关闭建议由记忆插件负责，必要时提出具体核心接口。讨论强调来源与人工确认应分开；未证明某种权重能阻止注入。  
**证据等级：** 功能请求＋产品范围分流；非已验证安全修复；需求高；特定防护有效性未验证；本次未复现。  
**RstrBot 实现目标：** 保存原始来源、作者、时间、版本和单独审核状态；模型不能自我提升可信度。保留、删除、撤销、导出和检索可见性有明确规则。  
**验收条件：** 用户确认后原始来源仍保留；撤回确认能改变可用状态；低信任检索文本无法改变工具授权；记录删除后索引和派生副本同步失效。

核对评论：[1](https://github.com/openclaw/openclaw/issues/7707#issuecomment-4687195726)、[2](https://github.com/openclaw/openclaw/issues/7707#issuecomment-6347213698)。


### OC-07 · 热重载后的身份与存储一致性 · P0

**来源：** [#186186 [Bug]: Native Talk consults keep writing the old session store after a hot route change](https://github.com/openclaw/openclaw/issues/186186)  
**状态：** open / —；最后更新 2026-10-27T18:05:01Z。  
**报告内容：** 报告切换会话存储后，旧语音回调仍向旧库更新元数据；普通语音入口已拒绝同一旧目标。  
**后续与关闭原因：** open。原报告在 Linux 回调测试中使用真实 SQLite、模拟模型执行，明确未证明跨用户泄露或真实供应商行为；自动审查未运行测试。  
**证据等级：** 范围明确的报告者测试；源码审查呼应；需求高；报告复现范围有限；本次未复现。  
**RstrBot 实现目标：** 新工作准入检查配置代次、目标资源与主体权限；后台回调不能绕过当前路由。已接受工作使用可解释的快照生命周期，数据库迁移另走受控流程。  
**验收条件：** 切换路由或撤销授权后，旧回调的新任务在写入及模型执行前被拒绝；未改目标继续正常；已接受任务按声明的完成/取消策略结束。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186186#issuecomment-6449806250)。


### OC-08 · 会话顺序、去重与背压 · P0

**来源：** [#111897 [Bug]: Two concurrent runs for the same session lane both complete and deliver duplicate/redundant replies under load](https://github.com/openclaw/openclaw/issues/111897)  
**状态：** open / —；最后更新 2026-10-27T11:31:41Z。  
**报告内容：** 报告高负载下同会话产生两次独立回答。原文把原因归为会话队列竞争，但没有完整关联执行时间线。  
**后续与关闭原因：** open。MEMBER 评论要求关联入口、队列代次、运行和投递 ID；自动审查仍称当前根因未确立。大量相同评论文本不能视作独立用户佐证。  
**证据等级：** 重复回答报告可信度中等；具体并发根因待证；需求高；根因低至中；本次未复现。  
**RstrBot 实现目标：** 定义入口去重、同会话执行顺序、跨用户公平队列、工作所有权和输出投递状态；外部平台不支持幂等时暴露投递不确定状态。  
**验收条件：** 重复 webhook、超时重试、进程重启与队列拥塞下，同一输入不产生重复副作用；同会话有序，不同会话不会被一个慢任务无限阻塞。

核对评论：[1](https://github.com/openclaw/openclaw/issues/111897#issuecomment-5027355876)、[2](https://github.com/openclaw/openclaw/issues/111897#issuecomment-5142710998)、[3](https://github.com/openclaw/openclaw/issues/111897#issuecomment-5691456992)。


### OC-09 · 计划任务的漏跑策略 · P1

**来源：** [#179927 Cron: a missed slot is silently dropped on normal scheduler ticks when the stored next run time is stale](https://github.com/openclaw/openclaw/issues/179927)  
**状态：** closed / completed；最后更新 2026-10-27T03:22:47Z。  
**报告内容：** 正常调度 tick 遇到过时的持久化 nextRun 值，会重算未来时间而漏掉应补跑的时段。  
**后续与关闭原因：** closed/completed。旧自动评论仍写待修复，但直接核对 PR #179928 已于 2026-10-27T03:22:45Z 合并；本次没有运行产品。  
**证据等级：** 有隔离复现的报告＋修复合并元数据；需求高；合并状态已核验，发布覆盖未核验；本次未复现。  
**RstrBot 实现目标：** 任务明确 skip/coalesce/bounded-catch-up、时区、停机恢复、并发重叠及失败回退策略；正常 tick 和重启恢复使用一致语义。  
**验收条件：** 模拟错时、停机、过时 nextRun 和已完成时段：按设置补跑或明确记为跳过，已完成时段不再次执行。

核对评论：[1](https://github.com/openclaw/openclaw/issues/179927#issuecomment-6327394728)；修复/关联 PR：[#179928](https://github.com/openclaw/openclaw/pull/179928)（已合并）。


### OC-10 · 任务所有权恢复和真实健康诊断 · P0

**来源：** [#185399 [Bug]: A 0-byte orphan cron-authority lock file is never reclaimed — every cron.* RPC fails permanently; restart and doctor --fix cannot recover](https://github.com/openclaw/openclaw/issues/185399)  
**状态：** closed / completed；最后更新 2026-10-27T03:52:35Z。  
**报告内容：** 报告空的残留任务锁导致重启后所有任务 RPC 仍失败，诊断却宣称恢复成功。  
**后续与关闭原因：** closed/completed。自动审查说明 main 移除该锁；PR #168772 已核实合并。不能采用“空锁一定死掉，可直接删”的建议，因为可能仍有活写者。  
**证据等级：** 报告＋修复合并元数据；无独立复现；需求高；主干修复关系有元数据支持；本次未复现。  
**RstrBot 实现目标：** 优先数据库所有权/事务等成熟原语，支持可证明的故障恢复；健康检查逐项报告任务和插件能力，修复失败不能显示全绿。  
**验收条件：** 在取得所有权前后强制中断进程，重启可恢复且不抢活执行者；延迟重试的旧修改有过期/版本保护；doctor 返回真实失败状态。

核对评论：[1](https://github.com/openclaw/openclaw/issues/185399#issuecomment-6433306488)；修复/关联 PR：[#168772](https://github.com/openclaw/openclaw/pull/168772)（已合并）。


### OC-11 · 迁移不得扩大自动化权限 · P0

**来源：** [#186218 [Bug]: Heartbeat migration deletes an invalid quiet-hours window, so the automation runs with the defaults window or unrestricted](https://github.com/openclaw/openclaw/issues/186218)  
**状态：** open / —；最后更新 2026-10-27T19:02:00Z。  
**报告内容：** 报告修复/迁移过程删除无效静默时段，导致继承错误默认值或改成不限时执行。  
**后续与关闭原因：** open，影响报告时 main，未声称稳定版带该迁移；自动审查仅源码印证，候选修复仍需合并验证。  
**证据等级：** 报告者隔离 CLI 复现＋自动源码审查；需求高；已披露版本和验证限制；本次未复现。  
**RstrBot 实现目标：** 配置和数据库迁移保留原始意图；无效时间、权限、预算不能默默回落到更宽松值。提供变更预览、备份、失败回滚和可重试诊断。  
**验收条件：** 导入无效时段/未知权限/损坏字段时保持旧数据、禁用受影响自动执行并指出位置；修正后原子迁移，不丢记录或重复创建任务。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186218#issuecomment-6451005649)。


### OC-12 · 工具循环与供应商收尾协议 · P1

**来源：** [#186205 Bedrock settled-turn finalization omits toolConfig, killing turns that end on a tool call](https://github.com/openclaw/openclaw/issues/186205)  
**状态：** closed / completed；最后更新 2026-10-27T18:36:09Z。  
**报告内容：** 报告 Bedrock 收尾请求保留工具调用块却没有 toolConfig，导致已执行工具后的最终回答失败。  
**后续与关闭原因：** closed/completed；PR #166720 于 2026-10-27T05:05:46Z 合并已核验。修复是在禁用工具收尾时转换已完成证据，并非重新开放工具；发布版本覆盖未独立核验。  
**证据等级：** 日志报告＋修复合并元数据；供应商线上行为未测试；需求高；主干修复状态已核验；本次未复现。  
**RstrBot 实现目标：** 模型插件声明工具/媒体/流式能力，续轮、压缩、重试和终结阶段保留合法证据；终结阶段不能新增工具副作用。  
**验收条件：** 对每类供应商测试工具调用→结果→最终回答；收尾验证失败可诊断，但不得重跑已成功工具；本地协议测试与真实供应商测试分别标记。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186205#issuecomment-6450473940)；修复/关联 PR：[#166720](https://github.com/openclaw/openclaw/pull/166720)（已合并）。


### OC-13 · 有效工具目录与可解释失败 · P1

**来源：** [#120385 [Bug]: Code Mode tool catalog built incomplete for scheduled/cron and memory-flush turns — MCP tools missing ("Unknown tool id"), surfaced only as "⚠️ 🛠️ Exec failed"](https://github.com/openclaw/openclaw/issues/120385)  
**状态：** open / —；最后更新 2026-10-20T22:19:43Z。  
**报告内容：** 早期报告称计划任务莫名丢 MCP 工具；作者随后纠正为任务 allowlist 导致，另有异步媒体续轮问题。  
**后续与关闭原因：** open。#174696 的媒体续轮修复已核实合并；余下缩减现象缺当前版本配对证据。原文“排除了 allowlist”已撤回，不能继续当事实。  
**证据等级：** 报告者因果纠正＋部分修复已合并；剩余待证；需求高；剩余缺陷根因未验证；本次未复现。  
**RstrBot 实现目标：** 目录明确区分已安装、健康、启用、被授权和本次可调用；异步/计划/子代理沿用真实主体范围，减少工具须有原因；错误保留阶段和原始分类。  
**验收条件：** 同一主体、同一策略的交互和计划任务显示一致有效能力；拒绝/缺失原因可见；工具失败不得伪报整任务成功，也不能用通配符自动提权。

核对评论：[1](https://github.com/openclaw/openclaw/issues/120385#issuecomment-5223526688)、[2](https://github.com/openclaw/openclaw/issues/120385#issuecomment-5224322960)、[3](https://github.com/openclaw/openclaw/issues/120385#issuecomment-6301982372)、[4](https://github.com/openclaw/openclaw/issues/120385#issuecomment-6303055705)；修复/关联 PR：[#174696](https://github.com/openclaw/openclaw/pull/174696)（已合并）。


### OC-14 · 流式消息和最终结果一致 · P1

**来源：** [#186191 [Bug]: Matrix partial streaming with block streaming sends a stray draft after the last block, then redacts it (block offsets ignore the separators)](https://github.com/openclaw/openclaw/issues/186191)  
**状态：** open / —；最后更新 2026-10-27T18:17:47Z。  
**报告内容：** 报告 Matrix 多段落流式回复结尾多发草稿再删除；渲染文本长度与原始分隔符坐标不同。  
**后续与关闭原因：** open。自动审查给出源码对应关系但未做网络/客户端实测；报告提供事件序列和临时关闭草稿流的绕行。  
**证据等级：** 报告者事件证据＋自动源码审查；需求高；端到端未独立复现；本次未复现。  
**RstrBot 实现目标：** 消息段、原始文本、编辑版本和最终输出有统一标识；平台适配器只做能力映射，明确不支持编辑的平台降级行为。  
**验收条件：** 多段落、Unicode、长文本、限流、中断/重连下无重复尾部、无多余删除占位，历史记录与用户最终所见一致。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186191#issuecomment-6450101153)。


### OC-15 · 插件故障隔离与进程回收 · P0

**来源：** [#144911 [Bug]: MCP server init timeout crashes the Gateway — unhandled rejection "service child cleanup identity lost" in child cleanup path](https://github.com/openclaw/openclaw/issues/144911)  
**状态：** closed / completed；最后更新 2026-10-03T01:08:09Z。  
**报告内容：** 报告 stdio MCP 初始化超时后清理失败未被处理，导致整个网关退出。  
**后续与关闭原因：** closed/completed。COLLABORATOR 末条记录 Linux/arm64 实际网关两次超时后 PID/健康/工具入口仍正常并干净退出；#144941 已核实合并，本次未重复该测试。  
**证据等级：** 报告＋有权限贡献者公开运行验证＋合并元数据；需求高；修复有公开实测，非本次实测；本次未复现。  
**RstrBot 实现目标：** 运行时插件初始化、通信、退出和回收各阶段有超时与错误隔离；子进程树归属可验证，单插件故障不关闭无关会话。  
**验收条件：** 构造不响应握手、崩溃、僵死孙进程和清理异常；核心健康、无关会话持续可用；三个操作系统分别证明回收和权限边界。

核对评论：[1](https://github.com/openclaw/openclaw/issues/144911#issuecomment-5748541344)、[2](https://github.com/openclaw/openclaw/issues/144911#issuecomment-5963946461)；修复/关联 PR：[#144941](https://github.com/openclaw/openclaw/pull/144941)（已合并）。


### OC-16 · SQLite 运维与版本约束 · P0

**来源：** [#143524 [Bug]: Agent SQLite WAL grows to 1.4–2.8 GB in days despite wal_autocheckpoint=1000; blocks gateway startup (Windows, 2026.9.2/9.3)](https://github.com/openclaw/openclaw/issues/143524)  
**状态：** closed / completed；最后更新 2026-10-26T14:02:19Z。  
**报告内容：** Windows 报告 WAL 增长伴随服务启动阻塞；早期怀疑长期读事务，不能据此断言 SQLite 本身不适合。  
**后续与关闭原因：** closed/completed；作者最后回报同一应用更换 Node 运行时并离线整理后连续 84 小时无相同症状。运行时绑定归因是报告者结论；相关应用守护 PR #184076 仍 open。  
**证据等级：** 长期现场证据＋作者运行时 A/B 与关闭回执；本次未复现；运维需求高；特定根因未独立验证；本次未复现。  
**RstrBot 实现目标：** SQLite/PG 均有版本门槛、阻塞查询/连接诊断、备份一致性及崩溃恢复测试；WAL/连接/磁盘监控面向真实服务，不能只凭进程存在判定就绪。  
**验收条件：** 长读事务、写争用、磁盘接近满和中断恢复时给出有界等待及准确健康状态；在线备份恢复后提交记录完整；异常结束后可安全启动。

核对评论：[1](https://github.com/openclaw/openclaw/issues/143524#issuecomment-6406469006)、[2](https://github.com/openclaw/openclaw/issues/143524#issuecomment-6407176908)、[3](https://github.com/openclaw/openclaw/issues/143524#issuecomment-6417921332)；修复/关联 PR：[#184076](https://github.com/openclaw/openclaw/pull/184076)（未合并）。


### OC-17 · 按字节计量的存储预算 · P1

**来源：** [#114612 [memory-core] SQLite unbounded growth: memory_index_chunks + memory_embedding_cache tables have no retention policy, will fill disk over time](https://github.com/openclaw/openclaw/issues/114612)  
**状态：** open / —；最后更新 2026-10-27T12:27:01Z。  
**报告内容：** 记忆索引和 embedding 缓存增长造成磁盘与启动压力，主会话清理没有覆盖插件表。  
**后续与关闭原因：** open；后续自动审查指出 main 已有行数上限和二进制存储，原标题的“完全无淘汰、JSON 向量”已过时；按字节预算和仍存在源文件的保留语义待定。  
**证据等级：** 现场测量＋后续纠正；当前源码未独立复核；需求高；原始无限缓存根因不可当当前事实；本次未复现。  
**RstrBot 实现目标：** 插件数据纳入工作空间磁盘预算；区分不可重建用户数据与可重建索引缓存，按字节/年龄/活跃模型管理并说明实际物理回收。  
**验收条件：** 反复重建索引、切换 embedding 模型和轮转密钥后磁盘保持声明预算；清缓存不删用户原文，重新检索可恢复且不串租户。

核对评论：[1](https://github.com/openclaw/openclaw/issues/114612#issuecomment-5098293830)、[2](https://github.com/openclaw/openclaw/issues/114612#issuecomment-5744425552)、[3](https://github.com/openclaw/openclaw/issues/114612#issuecomment-5981090572)、[4](https://github.com/openclaw/openclaw/issues/114612#issuecomment-6144450953)。


### OC-18 · 原生桌面真正验收 · P1

**来源：** [#186215 [Bug]: macOS New Thread recursively calls createSession and exhausts memory](https://github.com/openclaw/openclaw/issues/186215)  
**状态：** open / —；最后更新 2026-10-27T18:54:23Z。  
**报告内容：** 报告 macOS 原生 New Thread 进入递归并持续分配内存，而同一网关的 Web 新会话正常。  
**后续与关闭原因：** open；有报告者符号化栈和受控构建复现，自动审查确认源码路径；本地修法尚无成功原生内存验收，不能称已修复。  
**证据等级：** 报告者原生复现＋自动源码审查；本次未运行；需求高；修复与其他平台覆盖未验证；本次未复现。  
**RstrBot 实现目标：** 桌面与 Web 使用同一业务能力，但分别验收真实安装包、导航路径、sidecar 生命周期和资源上界；动效不阻塞基本操作。  
**验收条件：** Windows/macOS/Linux 安装后完成创建会话、流式聊天、离线/重连、升级回退和退出回收；一次点击只有一次创建，重复操作内存可回到稳定区间。

核对评论：[1](https://github.com/openclaw/openclaw/issues/186215#issuecomment-6450856153)。

## 应避免的误读

1. #20305、#3259、#8081 的关闭不能写成“原生多用户已经解决”；它们分别有自动过期、优先级分流或未明确落实的背景。
2. #186205 的修复保留最终收尾禁用工具的约束；盲目把工具配置加回去可能允许继续执行副作用。
3. #186216 作者已经否定“共享目录内再建会话子目录”和“只靠删除过期附件”能够完成访问隔离。
4. #111897 的重复回复值得做回归场景，但当前证据不足以锁定为单线程或某一队列算法造成，重复粘贴的评论不提高因果可信度。
5. #143524 的最新关闭回执指向特定 Node SQLite 绑定/运行时版本，不能拿旧标题证明 SQLite 原理上不能用，也不能据此宣称改用 Rust 自动解决全部持久化问题。
6. #114612 后续已修正无限缓存、JSON 向量等早期描述，剩余需求是字节预算、保留语义与物理空间回收。
7. #120385 原始“allowlist 已排除”的结论被作者撤回；产品应给出有效能力和拒绝原因，不能因工具缺失自动扩大权限。
8. #185399 的空锁可能处在活写者的创建窗口；清理应证明所有权安全，不能以“空文件”直接判死。
9. 参考项目选择单一可信操作员边界并不等于实现错误；RstrBot 原生多用户是不同的明确产品承诺，应由核心自行承担。

## 复用边界

研究文件只保留简短转述和链接，没有搬运代码、长日志、用户标识或凭据。完整结构化索引见同目录 `openclaw-issues.json`。正式构建提示词可使用 OC 编号追溯，但不需要把参考项目名称写入产品品牌或用户界面。

