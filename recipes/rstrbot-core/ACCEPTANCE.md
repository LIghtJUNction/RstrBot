# rstrbot-core 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-CORE-01 | 临时SQLite启动，使用终端bootstrap凭据建立账户，再重复/并发setup | 仅有效证明一次成功；产生登录会话，15分钟过期或已初始化后拒绝 |
| RSTRBOT-CORE-02 | 不匹配Origin、跨站Fetch-Site、缺浏览器mutation头、有效/无效Bearer | 按OpenAPI拒绝越界；Bearer不绕过Origin校验 |
| RSTRBOT-CORE-03 | 遍历OPENAPI.md全部操作，以合法与非法DTO调用 | 注册路径/方法、200/204及错误封套、必填和UTF-8边界一致 |
| RSTRBOT-CORE-04 | 配置diagnostic模型并启用工具实例，发送/diagnostic/tool hello | 真实worker发现echo、调用后续轮、消息/事件/终态持久化 |
| RSTRBOT-CORE-05 | 真实本地模型夹具流出片段后断开/重启核心 | 已落库片段可恢复，run interrupted，重启不再调用上游 |
| RSTRBOT-CORE-06 | 重复chat幂等键同参数/变正文；共享会话尝试私有代理 | 重复不多发，变参数冲突；私有代理原子拒绝无run/键消费 |
| RSTRBOT-CORE-07 | 调用中注销、改密、到期或移出工作区 | 租约失效后不启动下一模型/工具；已知使用仍保留 |
| RSTRBOT-CORE-08 | 模型完整URL路径或provider变更，不给新密钥/clear | 400拒绝；明确清除后保存，原秘密不转发新目标 |
| RSTRBOT-CORE-09 | 普通成员启用私网，或实例政策禁用后调用原已获准私网模型 | 授权拒绝；阻塞调用不预留新费用 |
| RSTRBOT-CORE-10 | 模型测试在本地上游记录请求 | 实际调用保存配置，最多8输出token，usage缺失为未知，无自动重试 |
| RSTRBOT-CORE-11 | 知识含Unicode与HTML文本；私有/共享检索、删除、归档记忆 | 引用准确、显示安全、ACL先过滤；归档与删除不混淆 |
| RSTRBOT-CORE-12 | 一次/间隔/时区cron任务preview与执行，执行中所有者降权 | preview无派发；正确认领与skip，重新核权 |
| RSTRBOT-CORE-13 | 路由保存后直接发送，批准启用后发送；UTF-8恰好/超过32768字节 | 未启用拒绝；启用后最多一次派发；超限验证失败 |
| RSTRBOT-CORE-14 | 第9个Webhook等待槽位时注销发起会话，另一个同用户会话仍有效 | 旧凭据不被新凭据续命，派发前拒绝并保存真实状态 |
| RSTRBOT-CORE-15 | Webhook在目标接收后断线，再重复POST同key和查询receipt | unknown不伪造成未发送；不创建第二投递；GET只读 |
| RSTRBOT-CORE-16 | 数据库连接只读探测、无关schema、Valkey中断 | 探测不迁移、不覆盖；可选缓存degraded不导致授权放宽 |
| RSTRBOT-CORE-17 | 密码/邮箱/Google/Passkey一次性挑战和双凭据换绑 | 不重复消费；新凭据验证成功前保留旧凭据；稳定sub显式绑定 |
| RSTRBOT-CORE-18 | 数据库备份恢复及recovery/complete；非空目标和旧版本回滚尝试 | 按恢复边界拒绝危险覆盖；暂停外部执行，不重放未知操作 |
| RSTRBOT-CORE-19 | 管理员从实际OpenAI兼容模型端点发现列表；覆盖缺少密钥、已保存密钥复用、切换provider/地址、私网关闭、撤权与超时 | 只调用`model.list`并限制/净化返回ID；不生成、不预留预算、不持久化临时密钥；不能把已保存密钥转给不同地址；撤权时停止后续调用；列表不可用时API明确失败而表单仍允许手动ID |

## 1.13 新增软件验收（全部 not_executed）

本表不是本轮执行回执。用独立临时数据根、纯合成账号U1/U2、工作区W1/W2、可计数的本地真实HTTP/插件/文件夹具执行；需要商业模型、真实平台、实体设备或原生OS的部分单列 blocked/not_executed。每个通过必须有实际下游字节/记录/计数与故障后读回；仅返回200、前端截图、mock状态或文件存在不够。所有用例保留调用者/配置/版本、操作ID、实际命令、故障时刻、重启后的事实及 passed/failed/not_executed/blocked。所有新接口在未实现时应返回不可用，不能为通过测试伪造业务成功。

| 编号 / 合同 | 输入、成功路径与故障/恢复操作 | 必须核对的下游事实 |
| --- | --- | --- |
| R13-CORE-001 / D01 | SQLite/PG各建立两用户两工作区、五角色和同名资源；各角色从HTTP/CLI/MCP/SDK调用同一读/改/删/运行；再降权、移除、换ID和重启 | 各入口领域code一致；私有内容、总数、事件/导出/缩略图零越权；instance admin非成员无业务写权；最后owner变更拒绝，历史事实保留 |
| R13-CORE-002 / D01 | 同一用户两会话；第一会话的调用停在队列/握手/派发前后，注销第一而第二仍在线；另测个人token缩权、服务grant撤销/创建者注销 | 第一租约不被第二续命；未派发计数0，已派发保留实际accepted或unknown；服务授权按独立grant且不冒充个人，已知费用不抹去 |
| R13-CORE-003 / D01/E01 | 读取三态能力；分别缺插件、无授权、坏外部前提、错误HTTP/worker协议；对缺能力动作强行POST；修复前提重读 | supported/authorized/available及原因真实；未知版本/操作不执行、不降级、不收费；恢复能力只影响新准入 |
| R13-CORE-004 / D01/E02 | 分享会话但不分享其私有附件/记忆；一次token用专用header+当前受众会话读回，重复键/GET取token，非受众/匿名尝试；撤销后继续旧分页cursor/事件/下载/嵌入入口，重复和重启 | 仅明确受众和资源/消息范围可读；token只首次201出现且不入URL/日志/持久输出；旧cursor不绕ACL；撤销后新读取拒绝，迟到结果不恢复缓存；已下载外部副本不能声称撤回 |
| R13-CORE-005 / D02 | 配置各层inherit/value/clear/disabled，运行中修改人格/模型；reset后让旧模型迟到；导入人格冲突/取消 | 实际上游请求与有效值/来源/digest一致；旧run使用旧快照，新run用新值；示例未写成真实消息，旧代不写新投影 |
| R13-CORE-006 / D02 | 保存供应商A并选其模型，再新增B；注入B保存失败、CAS冲突、取消和迟到发现；重启读回 | A/B独立ID与配置/选择均保留；不把同名模型当同资源；旧请求不覆盖新配置；目标变化不发送A秘密；对应R13-OC-69160 |
| R13-CORE-007 / D02/E05 | 实际受信代理夹具验证一次认证注入、一次受信签名/刷新；普通tool/另一workspace借用handle；改target/轮换/撤销/重启 | 仅批准目标接到正确认证，其他worker/日志/错误零秘密；旧handle失效；刷新失败auth_required且不换计费身份；一次token丢失后不能读回明文 |
| R13-CORE-008 / D02 | OAuth/绑定/轮换在verified之前取消、失败与重启；已有credential活跃时换目标retain；同昵称/邮箱不同稳定身份 | 旧active保留直到原子成功；旧挑战不复用、无自动账户合并；错误配置不关闭TLS或使用另一账户 |
| R13-CORE-009 / D03 | 同一run发送稀疏工具片段、Unicode/未知provider_state、工具name纯文本及畸形JSON；跨provider尝试复用签名 | 只有合法结构化调用一次执行；调用ID/结果配对、原始字段可追溯；普通文本零tool；不伪造签名，错误有唯一终态 |
| R13-CORE-010 / D03 | 真实本地上游先未写出即拒绝，再已流出片段断线；设置候选链/总max_attempts；记录底层HTTP计数 | 只有批准且满足未发送/幂等条件重试；已输出/未知不静默回退；SDK和模型回退总尝试不相乘；每attempt费用独立 |
| R13-CORE-011 / D03 | 模型重复无效工具/相同结果直到限额；工具总限额到达；在慢hook/审批/模型等待取消 | 无进展与纠错限额独立；最多一次合法tools_disabled收尾且受剩余预算/截止约束；取消不转普通error触发fallback；进程有界清理 |
| R13-CORE-012 / D03 | 创建长历史、branch、reset、stop、delete分别操作；迟到旧generation片段；事件断线/重复/乱序/过期cursor | 永久历史与投影区分；不混新会话；一次执行终态；有效cursor恢复完整，过期明确重同步，慢消费者不把已完成执行改超时 |
| R13-CORE-013 / D03 | 并发父/子/嵌入/重排/缓存计费调用触达预算；missing usage/price、实际超估值；四SDK返回USD亚微美元/大数溢出/非USD无汇率；取消/撤权后读账本 | 预留原子且父子不双花；raw charge无损，核心精确ceil到微美元并记rounding_delta；转换未知保持null而非0，不释放预留；strict拒绝无可约束上限，estimated显示保证范围 |
| R13-CORE-014 / D03 | 让模型成功完成，在终态/消息/事件保存交界注入存储失败；重启和只读核对 | execution成功与persistence失败独立，已有意图可找；无第二模型请求或第二终态；不能因保存失败触发另一付费供应商 |
| R13-CORE-015 / D04 | 写工具提案→审批→派发；在审批后改参数/target/package generation/费用上限；同键同参数/异参数与不同合法意图 | 改动使审批失效；未批准/未知effect无外部调用；同键读原事实、异参数409；不同合法意图不因文本相同被误去重 |
| R13-CORE-016 / D04 | 撤权分别卡在批准后、排队后、握手后、send后；远端接受但丢回执；取消/连续重启/再次同键POST/GET核对 | 派发前计数0；派发后accepted或unknown保留；GET零写请求；不自动重放、不借其他会话/身份/路由；实际目的端计数<=1 |
| R13-CORE-017 / D04/D07 | 文件工具/缩略图/URL下载/解压尝试跨workspace资产、../、软硬链接、重定向私网、TLS错误、解码炸弹；沙箱准备失败 | 无越权bytes/网络访问；TLS不降级；隔离失败无宿主执行；取消与失败保留已发布文件和可核对记录 |
| R13-CORE-018 / D05 | 新建库并并发导入asset/URL两来源，其中一份畸形/过长；source.replace CAS冲突，parsing/indexing/publish暂停、取消、source.delete并重启 | KnowledgeSourceRead、原文/规范化asset、parser/version和引用位置实际读回；坏文档独立；旧版/已删来源不发布，URL删除同样阻断派生；取消只清自己stage，索引不悬空 |
| R13-CORE-019 / D05 | 更换嵌入model/维度建新代，半途崩溃后按checkpoint继续；切active时CAS冲突；查询两库同名文档 | active始终完整同代，失败保留旧代；index identity和source版本一致；先ACL后召回，非法候选未发外部重排 |
| R13-CORE-020 / D05 | 固定相关性数据集词法/混合/重排；故障注入重排超时和权限撤销；实际打开引用原文位置 | 输出来源/version/位置真实，无法定位不编造；降级仅已授权候选、标degraded；计费可追溯；质量/时延报告保留未命中样本 |
| R13-CORE-021 / D05 | memory.consent.grant后生成有来源的摘要/记忆，重启截获模型输入；撤销consent/修改删除来源；memory.update分别设置archived/disabled/pinned再恢复，memory.delete；尝试替他人同意 | 自动派生默认关闭且同意有限用途/范围/到期；撤销阻止后续调用；派生失效从输入移除；pinned不自动删；archive可恢复不等delete；人工编辑保留来源/edited_by；私聊来源不共享 |
| R13-CORE-022 / D06 | 在合成时钟下完成一次schedule occurrence，连续重启三次并核对存储/模型/投递计数；手动run另建意图 | 已消费时刻不重执行，manual不冒充自动补跑；已执行/中断事实不消失；对应R13-OC-42640/59301 |
| R13-CORE-023 / D06 | 在认领提交前/后、模型派发前/后、receipt提交前/后逐点终止；反复重启、暂停/启用和owner降权 | 每个已接纳occurrence有持久记录；未派发not_sent/已派发unknown；needs_reconciliation禁用；skip不补旧未知，owner失权零新派发 |
| R13-CORE-024 / D06 | once/interval/六字段cron，Asia/Taipei/Asia/Tokyo和有DST区域；时钟前后跳，重复/不存在当地时刻；修改时区后的迟到preview | 核心preview与同版执行instant一致；显示两地/UTC/tzdb/paused；preview模型/投递/预留计数0；v1升级差异明确、不假装前端preview |
| R13-CORE-025 / D06 | 一个生成结果送两个独立批准目标；A accepted后DB失败、B rejected或unknown；立即在A回复、重启核对 | 每目标独立事实；A已知发送上下文可关联或明确待核对；B不污染A或其他私聊；无以整个任务失败触发重复发送；对应R13-OC-91420 |
| R13-CORE-026 / D06 | 独立MCP适配器在无完整代理/知识库时投递，参数尝试换workspace/目标；固定状态根重启及撤销凭据 | 实际sink只接已授权目标；同键/unknown不重放；与集成模式receipt/persistence语义一致；独立身份不借浏览器会话授权 |
| R13-CORE-027 / D07 | 一组平台入站序列：提及/非提及/安静/限流/自身回声/引用旧自身消息；群聊与私聊并行，重启恢复 | silent仍持久消费cursor且非answered；回声零新规划，引用旧消息可正常处理；私聊不被群安静策略堵住；每event仅一次合法处理 |
| R13-CORE-028 / D07 | 外部会话从Web查看/发送新授权意图；连接重连新generation后旧连接退出；混入另机器人event_id | stable会话/origin/entry/authorized destination分离；Web查看不改route；旧连接不清新代；ID字符串不授权，跨机器人不混；对应R13-OC-84544 |
| R13-CORE-029 / D07/E02 | 上传合成图片/音频/文件，掉线后只补缺块；块摘要冲突、取消、过期、target撤权；完成后读出实际bytes | 摘要与原件一致，未验证不published；同块同摘要幂等，异摘要409；撤权拒绝；取消不删已发布；file path字符串不算消费证据 |
| R13-CORE-030 / D07 | 图/音视频转换、本地PDF读取、HTTP错误伪装媒体、长文组件/引用/转发与文本兜底；关闭执行沙箱仍读取获准资源 | 媒体类型/顺序/来源正确；错误页不成图片；voice失败保留费用和失败，只经明确选择文本兜底；只读资源不强依赖执行沙箱 |
| R13-CORE-031 / D07 | 平台流式限速、网络断开、取消与重连；大积压触及queue上限；真实受控适配器最终读回 | 尾段/终态不丢且不逐token刷屏；backpressure明确，非无限排队；未知发送不补发，重连/消费去重不丢合法新event；真实平台另记blocked |
| R13-CORE-032 / D08 | task.share邀请后分别由本人accept/revoke与尝试代替他人同意；父run分派两个child，带已授权媒体/上下文/预算；尝试额外grant、private→共享task与非法result_target；父撤权/取消 | 未同意者内容零读取；consent CAS/目的/来源生效；child仅交集权限，父子预算无双花；媒体实际读取；后代停止后续派发，既有费用/结果保留 |
| R13-CORE-033 / D08 | child失败分别fail_fast/collect；foreground→background显式/缺失委托；父进程重启后再submit同意图 | 背景不偷换原租约；重复submit返回同child_run_id；无重复模型/投递；unknown子写不转其他会话继续 |
| R13-CORE-034 / D08/E09 | DAG含无效引用/循环/越权/映射冲突，branch两分支/缺字段/类型错，join不同到达顺序和skipped/unknown；validate/simulate/controlled_read查看生产sink计数，再activate+批准execute并取消/重启 | 纯条件无eval；schema拒绝错图；join只按固定策略结算，unknown阻止副作用后继；三种调试生产写0，模拟标合成；execute每节点真实事实/费用与冻结版本，恢复不重跑写节点 |
| R13-CORE-035 / D08 | 外部工作流/搜索/MCP返回超大或未知schema、坏引用、OAuth失效、非幂等断线；修正后重试新只读请求 | 能力/引用/费用/来源真实，unsupported不伪成功；UI资源无父权限；非幂等unknown不重放；相同代失败合并有界重连 |
| R13-CORE-036 / D09 | schema2快照→新实现显式升级→空目标恢复；缺key/asset/plugin状态、非空目标、旧二进制打开新schema、过程中取消 | 保留来源；缺项清楚；旧binary拒绝；quarantine无外部执行；解除恢复不重放unknown；软件未实现就保持not_executed |
| R13-CORE-037 / D09 | 不同cwd启动，同数据根并发核心；维护时移动数据根，注入校验/复制失败；配置无效升级 | 数据路径不随cwd改变；只一实例；失败不删来源/用户数据；原数据/权限/密钥指纹读回一致，不能自动覆盖唯一备份 |
| R13-CORE-038 / D09 | 未来PG双节点+共享资产，延迟旧owner使lease转移再恢复，网络分区/Valkey停机/旧worker迟到；压力中取消/暂停 | fencing拒绝旧提交/可控派发；外部未知不因接管重发；控制操作不饿死；PG存在不等于多节点通过；全部实际负载/失败保留 |
| R13-CORE-039 / D03/E01 | 制造消息幂等冲突/本地storage或ACL错误，再对照真正获准上游重试错误；脱敏日志前后计算业务digest | 本地错误模型HTTP/回退计数0且不消耗provider attempt；对照回退严格受总上限/预算；digest由原规范化业务输入得出；对应R13-OC-177909 |
| R13-CORE-040 / E01–E09 | 从本合同生成扩展OpenAPI/路由/四SDK DTO，遍历E05+E08所有operation_name、kind、ReadRef消费路由、未知字段/枚举/边界；运行全旧v1向量 | v1 66+2不改变；registry/DTO/handler/只读投影逐项闭合，未知动作拒绝；三态能力不从文档推true；HTTP/worker/schema版本可独立不匹配并清楚拒绝 |
| R13-CORE-041 / E08 | 实例管理员从固定archive/git/index/local候选discover→download→prepare明确执行批准→权限decision→activate；第二版本失败/取消/扩权拒绝；workspace admin尝试全局安装、建立合法实例后卸载三种数据策略，重启 | 实际摘要/许可/进程/注册表读回；未批准prepare执行计数0；旧active保留；跨scope拒绝；实例停用撤销租约；export未确认交付不删；未知/已完成回执不被卸载清空 |
| R13-CORE-042 / E08 | 用批准stdio命令与本地Streamable HTTP夹具分别保存MCP连接、启用、目录刷新/分页；初始化失败后修正同配置，版本错/撤权/新目录扩权、同代并发断线及unknown写调用，重启 | 目录在真实握手前不可用；profile准确；新工具不自动批准；一次合并重连有界；配置/目录可GET读回且secret不返回；非幂等unknown无重放，未批准命令不能执行 |

## 1.14 验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-CORE-20 | 对发现、模型测试与对话分别注入401/403/429、未知插件码和断网；重启后读run | 使用约定安全分类，HTTP v1保持502、持久失败终态和可选分类；旧记录/旧worker通用回退，不返回原始上游内容，不自动重放或猜测未知费用 |

## 1.18 会话模型绑定

根 RSTRBOT-28：验证新知识引用不持久复制原文、旧引用响应不暴露原文，并保留模型输入与版本定位；旧数据库物理副本及在途撤权仍须单独验证。

根 RSTRBOT-32：对已获准的 ready 文档注入损坏分块、缺失元数据与缺口；搜索必须明确降级并只列出有权访问的问题资料，对话在模型调用前失败并列出需要修复的资料；修复重存后恢复，无权和停用文档不得干扰。

执行根 RSTRBOT-24：会话元数据提供可选 last_model_id；仅在用户有权使用模型且消息已原子接纳后记录，旧数据缺值无需迁移猜测；共享会话不得使用私有模型。 未实际执行的平台和反例保持未验证。


## RSTRBOT-40 聊天字段完整性

普通和文本附件入口拒绝未知顶层及附件字段，消息、运行和模型请求均不增加；已声明请求仍成功，OpenAPI生成结果和运行行为一致。记录旧客户端兼容性变化。
