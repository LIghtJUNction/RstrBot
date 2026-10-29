# OpenClaw问题到配方的首批回归线索

查询日期：2026-10-29。先经GitHub仓库搜索确认openclaw/openclaw；连接器/CLI曾有网络EOF，随后使用GitHub正文页和恢复后的issue/全部评论接口核验下列5条。选择范围为模型配置、调度重启、投递与会话归属，不是全部issue调查。

这些是上游历史报告，当前5条均closed，不能称为OpenClaw现存缺陷或RstrBot已复现。选其可复用的用户目标作为验收加固；当前RstrBot比较状态均not_executed。旧机器人评论中的“保持open”不能覆盖后来关闭状态；具体发行修复未经核验的保持未知。

| 来源/报告版本 | 当前状态与讨论边界 | 转换后的最小合同/验收（尚待运行） |
| --- | --- | --- |
| [OC-42640](https://github.com/openclaw/openclaw/issues/42640)，2026.3.8，重启后再次执行已完成时刻 | closed；[关闭评论](https://github.com/openclaw/openclaw/issues/42640#issuecomment-4316720735)给出564f820efa2b及回归测试，本轮未独立核验发行包 | RB-032/031、CMP-06：记录某一计划时刻已执行→重启/崩溃恢复→同意图不再调用模型或投递；计数与持久记录交叉验证。保留本项目skip策略，不照搬上游catch-up |
| [OC-59301](https://github.com/openclaw/openclaw/issues/59301)，2026.3.11，连续重启时任务未留执行记录 | closed/completed，页面链接#71547；[早期评论](https://github.com/openclaw/openclaw/issues/59301#issuecomment-4318666224)曾要求独立设计，不能说当前仍open或已知发行版本已修复 | RB-032/051、CMP-06/12：副作用前先留可恢复执行事实，启动/执行交界被终止后可见interrupted/unknown；不静默丢失，也不自动重放未知外部操作。稳定窗口属于待评估策略，不强加任意数字 |
| [OC-69160](https://github.com/openclaw/openclaw/issues/69160)，2026.04.15，第二供应商覆盖原模型映射 | closed/completed；[关闭评论](https://github.com/openclaw/openclaw/issues/69160#issuecomment-4320320212)指明合并语义、测试和v2026.4.23，属于评论给出的修复证据 | RB-023/024/041、CMP-02：先保存A再添加B，两者ID/模型/选择保留；取消/验证失败不覆盖旧配置；显式替换与新增分开，有冲突时保留草稿 |
| [OC-91420](https://github.com/openclaw/openclaw/issues/91420)，2026.6.1，失败重试借其他会话中继，重复并污染记录 | closed/completed；[机器人评论](https://github.com/openclaw/openclaw/issues/91420#issuecomment-4649207376)为较早源码分析，明确未实机复现，不能等同当前故障或最终修复结论 | RB-029/030/031、CMP-07/11：投递目标/会话/授权绑定，拒绝或未知不得借不相关私聊兜底；注入失败后核对目的端次数、其他会话零污染、重启后可核对。runner静默与显式工具发送分别定义 |
| [OC-84544](https://github.com/openclaw/openclaw/issues/84544)，2026.5.12，来源渠道与当前入口混淆 | closed/completed；[机器人评论](https://github.com/openclaw/openclaw/issues/84544#issuecomment-4497804634)仅源码复现，未实际Discord→WebChat；关闭后当前行为须另验 | RB-027/038/040/050、CMP-05/10：稳定会话身份、原来源、当前入口和获准投递目的地分开；Web查看不改写原外部路由，模型不从ID字符串推断权限或目的地；跨入口结果与ACL实际读回 |

## 纳入方式

首批只新增回归合同，不声称上述5类行为已经在RstrBot全部通过。CMP-02/05已有局部配置/账号隔离证据，不能替代这里的新增组合；子代理/跨平台渠道等当前缺能力按CURRENT_SCOPE记缺口。Pro须逐条补上受影响组件的自足合同和验收ID，去重到现有RB需求而不删除52/215/287历史。

每轮继续检索共同任务对应的近期问题，记录open/closed/duplicate/fixed/not-planned及修复证据；未读正文/关键评论不能导入要求。不下载issue附件中的业务日志、不复制上游代码/隐私数据、不在上游发帖或触发修复机器人。纯安全/体验目标可以纳入，未经验证的现象仍标外部线索。

## 1.13 再核验与可落盘回归合同

核验日期：2026-10-29（Asia/Taipei）。本轮通过GitHub issue、全部评论和已链接PR的只读接口重新核对原5条；未下载业务日志附件，未执行OpenClaw、未接触真实渠道。下面新增的RstrBot要求来自用户既有52项能力与这些外部线索的交集，属于本项目设计取舍；不能把上游实现建议当作已证明的根因或必须复制的内部结构。

证据分层：`report`=报告者描述；`source_review`=代码/文档分析；`upstream_test_report`=上游声称运行的测试；`merged`=PR合并元数据；`release_claim`=发行/评论指出包含；`local_e2e`=本轮独立实机。OpenClaw 的 OC-177909 未在本项目复现；本项目现有本地检查只覆盖已完成请求之后，同键不同正文在模型派发前被拒绝的路径。它不能验证 OpenClaw 的冲突分类/回退，也不能代替“模型返回后保存冲突”路径。OC-185423 的明确 401 正常回退仍只是另一条局部路径，不代表 OC-177909 或完整回退合同通过。

| OC编号 | 本轮读取的正文与关键评论 | 最新状态/修复来源 | 本轮可得结论及限制 |
| --- | --- | --- | --- |
| OC-42640 | [正文](https://github.com/openclaw/openclaw/issues/42640)报告2026.3.8重启重复执行；[唯一评论](https://github.com/openclaw/openclaw/issues/42640#issuecomment-4316720735)说明前次计划时刻检查与回归测试 | closed/completed；2026-04-24关闭；评论引用[564f820efa2bb81ae49b8f55a93e41492ad31917](https://github.com/openclaw/openclaw/commit/564f820efa2bb81ae49b8f55a93e41492ad31917) | 历史报告+源码审查修复依据；本轮未跑对应测试、未证明最早包含的发行版本。上游catch-up方案不是RstrBot采用skip的反例 |
| OC-59301 | [正文](https://github.com/openclaw/openclaw/issues/59301)报告2026.3.11重启循环中在途计划无运行记录；[旧评论](https://github.com/openclaw/openclaw/issues/59301#issuecomment-4318666224)说保留独立设计议题 | closed/completed；[PR71547](https://github.com/openclaw/openclaw/pull/71547)于2026-04-25合并，merge=`924271385ba905c523b263f32dbcdb8ba373fc87`；PR说明把启动时保留的运行标记记录为中断失败并停用中断一次性任务 | 旧“保持open”评论已被后续合并/关闭覆盖。PR报告的测试不是本轮回执；最早发行版本未知。不把“重启后可重试”照搬成unknown外部操作可重放 |
| OC-69160 | [正文](https://github.com/openclaw/openclaw/issues/69160)报告2026.04.15依次添加供应商覆盖模型表；[唯一关闭评论](https://github.com/openclaw/openclaw/issues/69160#issuecomment-4320320212)区分默认合并和显式替换 | closed/completed；2026-04-25关闭；评论引用`7f6452897e25543c9263b67aabf140379d1c0b0c`和[v2026.4.23](https://github.com/openclaw/openclaw/releases/tag/v2026.4.23) | 本轮核实该release存在；“该版本含此修复”仍按关闭评论归因，未独立安装验证。后续类似问题也不能自动视为同一根因 |
| OC-91420 | [正文](https://github.com/openclaw/openclaw/issues/91420)报告2026.6.1借持久会话中继导致重复/污染；[旧机器人评论](https://github.com/openclaw/openclaw/issues/91420#issuecomment-4649207376)明确没有高置信实机复现 | closed/completed；[PR91453](https://github.com/openclaw/openclaw/pull/91453)于2026-06-13合并，merge=`4c23d1d5978e54c31e87579574746aad9d5bfddf` | PR缩窄危险run队列拒绝后的回退，保留合法cron父会话与普通会话路径，并附生产工具+合成注册表证明。不能说旧机器人分析就是当前失败；本轮没有复跑，最早发行包含未知 |
| OC-84544 | [正文](https://github.com/openclaw/openclaw/issues/84544)报告2026.5.12来源会话与Web入口混淆；[旧机器人评论](https://github.com/openclaw/openclaw/issues/84544#issuecomment-4497804634)仅源码分析 | closed/completed；[PR88755](https://github.com/openclaw/openclaw/pull/88755)于2026-05-31合并，merge=`61e9961abb771cb12885e055d50d698d3886cbcd` | PR增加origin/active/deliveryContext，正文的生产工具+临时存储证明明确未连接真实Discord/WebChat。本轮未复跑；不改原会话键/路由是需保留的反例 |

### OC-42640

<a id="oc-42640"></a>

**R13-OC-42640，P0，RB-008/031/032/051，CMP-06/12。** 责任：core计划认领与恢复，store发生时刻唯一约束，host派发前复核，web展示执行事实。输入为已完成的一次计划发生时刻及重启；输出为原run/receipt与之后真正新时刻，不产生第二模型/工具/投递。发生时刻身份使用计划ID、计划版本和计划UTC瞬间；时区策略先确定瞬间，不能用进程启动时间替代。

验收：用F-TIME执行一次cron，独立F-MODEL/F-SINK各记录1次；在终态后重启3次且回拨时钟穿过同一时刻，再读取账本，原发生时刻仍各1次；推进到下一合法时刻后才允许新调用。重启前撤销所有者或停用路由，即使有新时刻也不能发起新外部动作。G2确认丢失时保持unknown，不借重启补发；预算只保留实际已知结算/未知冻结，不重复预留。成功不靠删计划记录，失败保留发生时刻/run/receipt对应关系供恢复。当前 `not_executed`。

### OC-59301

<a id="oc-59301"></a>

**R13-OC-59301，P0，RB-008/012/032/051，CMP-06/12。** 责任：core启动恢复与store耐久接纳。输入为计划被认领后连续被终止；输出为可读取的接纳/执行阶段、中断或结果未知。规则是“先把本次工作存在这件事写稳，再进行外部动作”；无法写稳时不派发。无需为所有设备强加固定5分钟稳定窗口。

验收：分别在G0、G1、G2、G3终止进程，每种3次循环。G0无持久接纳且无动作；G1有中断事实且下游0；G2独立sink已接收而核心未拿到确认，恢复显示unknown且不重放；G3保留已知接受及保存异常。随后正常重启能列出每次已接纳工作的最后状态，不能默默推进next_run丢掉它。恢复期间默认暂停新外部执行，所有者撤销后不得恢复派发；新意图只能经当前授权进入新发生时刻/显式新操作。预算不因无最终回执退回零；任何丢失已接受事实或重复外发为failed。当前 `not_executed`。

### OC-69160

<a id="oc-69160"></a>

**R13-OC-69160，P1，RB-013/023/024/041，CMP-02。** 责任：core模型资源与CAS、store原子保存、web向导/有限建议。输入为已有供应商A和新增B；输出为分别有稳定身份的A/B及原有选择。新增按资源ID合并，显式替换/清除单独确认且只影响所指资源；相同显示名或相同model_id不能作为覆盖依据。

验收：先A后B、先B后A都保留两者；读取配置后分别真实调用并核对地址/凭据指纹。B空列表、错误密钥、取消、超时、迟到发现均不修改A；两标签旧版本保存返回冲突且保留当前草稿。撤销管理员权限后的保存/发现结果不落盘；换目标不带走旧秘密。重启后选择和资源仍独立。发现不生成且不预留模型预算；测试调用按实际次数记费，未知usage不写零。该合同不保证任一第三方模型速度，也不据列表出现宣称协议兼容。当前 `not_executed`。

### OC-91420

<a id="oc-91420"></a>

**R13-OC-91420，P0，RB-029/030/031/038，CMP-07/11。** 责任：core意图/目的地/运行授权快照、host受控派发、store回执与历史归属。输入为投递到明确目标T1的请求；输出为T1对应操作状态。当前run拒绝、结束或unknown都不是借用其他私聊/持久会话T2的授权。任何允许的内部父子交接必须是预先显式批准的同任务空间关系，并作为新一阶段可追踪，不能由字符串后缀推导。

验收：T1接受后掉ack、拒绝队列、run结束、撤权、超时、重启各一次，核对T1计数符合真实发生事实、T2计数0、T2历史摘要不变；重复同intent只返回原结果。增加正对照：具有独立批准的工具发送可以完成，即使runner自动宣告关闭；runner静默不授予工具发送权，也不撤回已获准工具事实。取消在发送前禁止调用，发送后不能伪造撤回。父子结果、生成、投递、保存分别显示；已确认写操作不因模型重试重复执行，unknown人工核对期间无新增费用/派发。当前 `not_executed`。

### OC-84544

<a id="oc-84544"></a>

**R13-OC-84544，P0，RB-003/027/038/040/050，CMP-05/10。** 责任：core会话上下文、host平台适配器、protocol可信元数据、web当前入口展示。输入为在合成外部群创建、随后从Web继续的同一会话；输出为稳定conversation_id、初始origin、当前run ingress与获准delivery_target。当前入口只对该在途run有效；查询另一个非活动会话不能带上查询者的active字段。模型可见文本摘要与结构化元数据同时明确这几种身份，敏感目的地按ACL裁剪。

验收：外部创建→Web只读查看→Web继续→外部继续，稳定ID与初始origin不变；查看不改原路由，发送仍必须明确目的地授权。模型输入/工具结果里的current入口与实际run一致，不能靠ID里“discord”等字串推断权限。更换当前操作者、撤销群成员与重启后再读，越权历史/路由均不可见；旧run迟到不能改新run入口。费用归实际run授权主体而非origin名称。增加非活动查询反例，active为空/不提供且不能误报Web。真实Discord仍未测，合成适配器证据不得升级为外部渠道通过。当前 `not_executed`。

## 有界扩展检索与反例处理

2026-10-28通过GitHub issue搜索（`is:issue`、`updated:>=2026-09-28`、仓库限`openclaw/openclaw`，updated降序）执行三组有限查询。这里只做候选筛选，不声称穷尽或按搜索结果数量推断缺陷频率。

| 查询 | 本轮返回上限/候选 | 处理 |
| --- | --- | --- |
| `(cron OR delivery)` | 5：184709、184474、128875、186700、186704 | 初筛；词法搜索结果不是严格因果分类；语音关键词/邀请创建等超出此次最小回归范围，不导入要求 |
| `cron restart` | 3：177909、177656、121232 | 177909与模型路由、恢复、预算直接交叉，读取完整正文与6条评论；其余未深读，保留为未纳入而非判定无关缺陷 |
| `inbound hooks` | 3：186700、115076、92473 | 186700与群聊/插件隐私交叉，读取正文与唯一评论并核官方文档；其余未深读、不导入新结论 |

共11个返回条目、10个去重候选，深读2个新增条目；没有下载报告附件。后续每轮可用相同上限或预先记录的新上限，不以无限检索拖延本次交付。检索命中仅用于选择阅读对象；纳入要求以完整正文/关键评论/当前状态和本项目适用性为准。

### OC-177909

<a id="oc-177909"></a>

[正文](https://github.com/openclaw/openclaw/issues/177909)报告2026.10.5的本地transcript幂等冲突被当模型失败引发跨提供方回退；2026-10-29 查询时 issue 仍 open。[PR177956](https://github.com/openclaw/openclaw/pull/177956)已于2026-10-26合并，merge=`9ca1643decade716fee56231949e557ad9b44b7c`；其上游报告称使用构建后的 Gateway、真实 SQLite 所有者冲突和本地 HTTP/SSE 端点做前后对照，明确没有复现原 Slack/重启/压缩触发。[PR178269](https://github.com/openclaw/openclaw/pull/178269)查询时仍 open。v2026.10.8 [发行说明](https://github.com/openclaw/openclaw/releases/tag/v2026.10.8)包含“session admission races no longer trigger model fallback”的概括说明，但没有把该句明确链接到 #177909/#177956；具体发行是否包含此修复仍记 `unknown`。已读的[源码审查](https://github.com/openclaw/openclaw/issues/177909#issuecomment-6281118620)未运行原触发；[后续评论](https://github.com/openclaw/openclaw/issues/177909#issuecomment-6433588182)区分本地冲突误触发回退与真实提供方故障后接纳失败，不能混成同一根因。

**R13-OC-177909，P0，RB-008/023/024/025/026，CMP-02/11/12。** 责任：core统一错误域、store幂等接纳、host模型尝试计数。新增规范：授权、参数、持久化、幂等冲突与提供方错误分别编码；只有明确允许的提供方错误才进入已批准回退链，本地状态冲突不会改善于换模型。持久接纳使用同一规范化业务正文指纹；日志脱敏是观察投影，不得改变业务比较语义或保存明文秘密。

验收：真实store中用同键不同正文制造冲突，测试“模型调用前”路径上游计数0与“模型返回后保存冲突”路径计数最多原1次；第二提供方始终0，已存原文不被覆盖，run明确报告本地冲突。重启/重试原键不重复生成。对照合法真实上游失败：在无已知输出/副作用且策略允许时按固定尝试上限回退，每次先核授权和预算；取消/撤权停止下一次。分别计已知成本、未知使用、真实请求次数，不把失败归为免费，也不宣称修复了全部触发冲突的源头。当前 `partial`：隔离 SQLite 的本地 HTTP 集成与 HTTP/SSE 夹具覆盖同键不同正文的预派发拒绝，确认 409、无新模型请求、run/消息/事件不变；报告见 [E2E 记录](../../docs/E2E_2026-10-29.md#r13-oc-177909)。模型返回后保存冲突、备用提供方账本/真实回退正对照、原 Slack/重启/压缩触发和完整 CMP-02 仍 `not_executed`；RstrBot 当前没有自动模型回退能力，不能将“备用提供方零请求”声称为本轮观察结果。

### OC-186700

<a id="oc-186700"></a>

[正文](https://github.com/openclaw/openclaw/issues/186700)报告2026.10.6入站消息hook不触发而其他hook正常，当前open；[唯一关键评论](https://github.com/openclaw/openclaw/issues/186700#issuecomment-6467672130)指出缺少已开启配置的复现，建议按配置支持处理。本轮独立读取[固定v2026.10.6文档](https://github.com/openclaw/openclaw/blob/fe06181dc5ecc01cf8904abbfc26925925efefbc/docs/channels/whatsapp.md#plugin-hooks-and-privacy)，确认入站观察默认关闭、需显式账户/渠道opt-in。该反例不支持“OpenClaw钩子坏了”，也不支持为了易用而默认向插件泄露私信；尚无本轮实机证明。

**R13-OC-186700，P1，RB-003/019/027/038/041/047，CMP-03/10。** 责任：host钩子准入、core账号级观察权限、web配置生效值/禁用原因、SDK文档。输入为观察插件、平台账号和显式观察授权；输出为能力三态及获准事件。插件已安装/已注册只表示支持，默认不包含读取私信正文/标识的权力。UI显示“未授权读取此账号的入站内容”及授权归属；错误注册路径有诊断，不能假装成功或无声忽略。

验收：默认关闭时插件收到0个正文/身份事件，但普通消息流程可用；账户A显式启用后只收到A获准范围，每个事件有稳定event_id及本实例generation；账户B仍0。重启维持同一授权，重复上游事件按可声明的重投/去重规则处理，不能产生第二业务副作用。禁用/撤销后排队事件在回调前重新检查，不泄露迟到内容；失败/超时/取消不阻断主处理且记录可见诊断。hook使用只读能力，不自动触发模型或外发；若用户另行批准昂贵动作，按独立运行授权/预算。当前 `not_executed`。

## 回归证据与持续更新规则

上述7个ID都写入新增验收集合，原5条不会因上游closed而删除。执行时采用 [COMPETITIVE_RUNBOOK.md](COMPETITIVE_RUNBOOK.md) 的G0～G4屏障、独立sink计数、实例/账号隔离与结果记录。每条记录来源状态检查日期、产品/commit、合成输入摘要、实际命令/步骤、下游读回、故障与恢复、当前授权、费用已知程度和passed/failed/blocked/not_executed。

持续循环为：有限检索→去重/阅读→分层记录状态和修复→判断适用性/反例→最小规范变化→组件验收→重建后独立实测→更新证据。新增要求不能授权真实外发或扩大个人数据收集；不能为了“超越”关闭竞品安全设置、篡改RstrBot范围或省略缺能力。相同根因复用合同/测试ID并追加来源，不重复堆砌新功能；无证据保持候选。上游修复后保留有价值的防回归验收，同时更新状态，不把历史问题重新宣传为现存失败。

## OC-185423 · 明确上游拒绝后的合法备用调用

[问题正文](https://github.com/openclaw/openclaw/issues/185423)报告 OpenClaw 2026.10.7、Windows 10、定时任务中主模型失败后，备用尝试在本地 transcript 接纳阶段发生幂等冲突，尚未发出备用 HTTP 请求；issue 于 2026-10-29 查询时仍 open。报告列出单个网关的多次事件，不能当作普遍发生率或本次实测。[唯一评论](https://github.com/openclaw/openclaw/issues/185423#issuecomment-6433645581)是只读自动审查，没有执行复现；其指出已合并分类修复不证明该接纳路径修复，同时另有健康备用调用成功的对照，原报告的普遍性说法证据不足。与 OC-177909 的“本地冲突被误归因导致错误回退”是相反的失败路径，不合并成同一触发或推断相同根因。

**适用性及边界**：RstrBot 的未来供应商回退合同适用；目前未证明本项目存在该缺陷，也未证明已有回退功能通过。不是所有超时都可重发：上游可能已收到请求或发生费用时结果为 unknown，仍禁止自动回退。

**R13-OC-185423 / P1 条件验收**：在已明确启用且预算允许的两提供方本地夹具中，第一方对一次聊天或一次计划 occurrence 返回可证明未产生外部生成的明确 401，第二方可用；保存一次用户消息/意图，执行后第二方账本恰为一次并可读回回复，前者拒绝一次，消息/运行/attempt 和权限归属可追溯；刷新及重启不再请求。调整仅属于执行尝试的字段不得让同一已接纳用户消息冲突；相同键更改正文、主体、工作区或权限则拒绝，且后备账本为零。本地接纳冲突不触发下一供应商；一个派发可能已发生的超时不得自动转移；取消、预算不足、撤权及未启用策略不得启动备用请求。所有未有能力/夹具的运行结果为 not_executed，不能用合同或上游报告代替实测。

**2026-10-29 本地复测**：OpenClaw `v2026.10.8` 隔离 Gateway 的调试触发和自动调度的一次性任务各运行一次，均由主提供方 HTTP 401 回退到本地夹具并返回 `READY`。账本为两次主提供方拒绝和两次回退成功，完整配置与限制见 [E2E 记录](../../docs/E2E_2026-10-29.md)。这未复现 issue 报告的 transcript 接纳冲突；原问题截至测试日仍 open，本结果不证明普遍修复。RstrBot 仍无已实现的回退能力，本条验收在本项目仍是 `not_executed`。

## OC-153139 · 长草稿视口变化后的编辑位置

2026-10-29 读取 [正文](https://github.com/openclaw/openclaw/issues/153139) 和 [自动审查评论](https://github.com/openclaw/openclaw/issues/153139#issuecomment-5744864461)：该问题 open；报告针对固定提交 `cbf935baba66e746612902a9c99d1e72d9ff7b0a` 在 Chromium 模拟竖屏→横屏→竖屏，文本与选择区未丢失，但长草稿末尾编辑位置不可见。报告区分手动滚动查看旧内容的控制组，未测实体移动设备或其他浏览器。评论称未在更新的 main 实际复现；本项目未复测 OpenClaw，不能推断其当前发行版仍有该问题。

**R13-OC-153139 / RSTRBOT-25**：用户希望尺寸变化不妨碍继续编辑，同时手动阅读旧内容不被打断；完整边界见根/Web验收。RstrBot 本轮 Chromium 模拟长草稿末尾往返的单一路径通过，其他插入点、手动阅读、输入法、实体设备与浏览器未测；不能将该局部结果视为整项通过。

## OC-157827 · 合并证据不能只指向第一个来源

2026-10-29 读取 [问题正文与评论](https://github.com/openclaw/openclaw/issues/157827)：状态 open。报告在 OpenClaw 2026.9.5/macOS 上称多来源片段合并后暂存时只指向第一条，作者随后澄清最终长期记忆是否错误写入未验证；评论提及修复工作但未构成本项目的复测。当前 RstrBot 尚无自动记忆派生；RSTRBOT-30 要求每个非连续片段对应全部来源，无法核对就不发布或提升，不能猜测修复旧聚合数据。该条为上游证据转化的未来验收，不是本项目已实现或竞品实测。

## OC-16844 · 常驻读取器与索引更新的代次

2026-10-29 读取 [历史问题](https://github.com/openclaw/openclaw/issues/16844) 及评论：报告针对 OpenClaw 2026.2.13、macOS 和 QMD，称 WAL 写入后缓存的只读连接导致新结果定位失败并被丢弃；问题因不活跃关闭，关联 [PR #16917](https://github.com/openclaw/openclaw/pull/16917) 已关闭且未合并。未复测当前 OpenClaw，也不将作者的比例、根因或补丁当作已证实。RSTRBOT-31 提炼可观察的一致性和故障提示；RstrBot 当前同步词法路径与 QMD 不同，外部索引尚未实现。

## 会话恢复中的终态与遗留占用（2026-10-29）

[OpenClaw #181770](https://github.com/openclaw/openclaw/issues/181770) 报告 2026.10.6 的一个既有会话持续拒绝新消息；状态 closed/completed，相关 [#181801](https://github.com/openclaw/openclaw/pull/181801) 已合并，处理终态遗留。报告者在[后续验证](https://github.com/openclaw/openclaw/issues/181770#issuecomment-6375256933)指出自己的未终态栅栏未被该修复覆盖；[后续 #181952](https://github.com/openclaw/openclaw/issues/181952) closed/completed，维护者称 [#181992](https://github.com/openclaw/openclaw/pull/181992) 在无活动运行和待投递时处理该形态，并描述隔离 Gateway 测试。上述属于报告者/维护者及上游测试证据；本项目未在当前 OpenClaw 或 RstrBot 复现同一状态，也未核实哪一个发行版包含修复。RSTRBOT-36 提炼按证据释放孤儿占用且保留仍有效的工作和副作用；本地验收 not_executed，不据单纯 killed 状态清除。

## OC-120385 · 工具目录在不同执行入口不完整

2026-10-29 查阅 [OpenClaw #120385](https://github.com/openclaw/openclaw/issues/120385) 正文和评论，问题仍 open。作者已更正：原定时任务案例的 MCP 缺失由任务有限 toolsAllow 配置导致，而不是最初猜测的身份绑定；部分媒体续接修复已合并，其他目录缩减与错误诊断仍需上游复测。RstrBot 不支持相同 Code Mode/MCP 通用目录；本地另实测100个停用插件遮蔽旧启用插件，交互和计划都漏工具。这是不同根因的本地证据，转为 RSTRBOT-38，不能视为对当前 OpenClaw 的实测。

## OC-115076 · 混合文本与图片的来源及可读取性

2026-10-29 读取 [OpenClaw #115076](https://github.com/openclaw/openclaw/issues/115076) 正文及评论，问题仍 open；报告者提出文字与图片一起发送时来源分类及附件可读取性问题。后续评论指出部分附件处理已有修复，剩余分类语义仍待决，不能把原报告所有现象视作当前已复现。RSTRBOT-39 先实现真实文本附件且保留用户正文，不宣称支持图片；未来完整媒体应分别保留用户正文、文件来源与能力，并验证实际下游文件，见CMP-09。


1.32 按 RSTRBOT-41 增加受限 PNG/JPEG 与正文及文本附件同时进入真实模型接收端的合同；本项目测试需核对原图摘要、重试和权限。此实现不证明 OpenClaw 当前仍有原报告问题，也不代表完整媒体 CMP-09 已通过。
