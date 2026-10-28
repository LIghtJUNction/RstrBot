# 外部适配器、MCP与兼容加载合同

此处定义 RB-020/021/022/023/024/027/030/033/037/038/039/049 的扩展重建目标，全部新增功能 `specified_not_implemented`，运行验收 `not_executed`。有限Webhook及独立MCP已有合同继续适用；本文件不是现有软件支持更多平台的证据。通过依赖 `sdk → protocol` 取得 `EXTENSION_PROTOCOL.md`，不复制HTTP/OpenAPI或原215/287追踪。

<a id="adapter-model"></a>
## A-01 供应商、工具状态与外部工作流

供应商配置由固定source/package/provider、完整规范化endpoint、模型ID、凭据版本、代理/私网政策和模型参数形成不可变运行快照。保存A后新增B只增加B；显式替换需要B的目标身份和CAS，不以同名模型键覆盖A。发现结果为有来源的ID/显示名/支持的能力，空列表、unsupported、authentication_failed和timeout分别返回；自由模型ID保留，列表成功不等于模型生成成功。

`model.list`不调用生成；`model.test`真实调用且有显示的费用/输出限制；`model.generate`承接实际运行。供应商必须逐项声明 `text / tools / image_input / audio_input / audio_output / video_input / embeddings / rerank / signed_continuation`，没有该能力就不能接受相应任务。当前OpenAI兼容实现及diagnostic保持原范围，diagnostic不作为AI质量、商业账号或费用实测。

续轮保留工具调用ID、顺序、稀疏分片和供应商opaque_state；未完整的参数JSON不执行。签名或多模态载荷不能跨不兼容供应商伪造；不支持转换返回明确缺口，用户可新建运行而非篡改原证据。达到工具上限时工具列表设为禁用且保留合法收尾；纯文字工具名称不触发工具。流结束标记缺失为失败/中断，不能以已有文本当完整成功。

重试预算属于核心：记录每个真实HTTP尝试、费用和工具意图。只有确认未派发/无输出且核心策略明确允许的调用能重试；已经有输出需新分支或显式继续语义；工具accepted/unknown不能因模型换路重做。错误包含 `domain:provider|authentication|network|storage|authorization|protocol|budget`；存储冲突、权限拒绝和取消不触发换供应商。费用缺失保持unknown，失败和快速决策调用同样记账。

外部工作流/搜索/专用媒体服务作为独立插件，通过实际能力映射：查询输入、授权资源、远端操作ID、流事件、引用、收费单位和可查询回执。结果引用含来源URI、检索时间、内容版本/摘要及访问范围；URL/结果正文是不可信内容，不授予抓取或执行权。没有真实endpoint验收只能标planned或configured，不以“Dify/Coze/搜索”预设名称表示兼容。外部job启动接受后断线保留remote_job_id或unknown；只读查询可继续，不能再次submit猜测补偿。

<a id="adapter-mcp"></a>
## A-02 通用MCP客户端与恢复

初始重建验收profile固定为 **MCP 2025-11-25** 的stdio及Streamable HTTP。这是明确的兼容目标，不称“最新标准”：核对日2026-10-28的latest指向2026-07-28，尚未做版本迁移与验收，因此该较新profile保持unsupported。应用插件RPC的1.1-draft与MCP的日期版互不替代。profile变化必须更新版本化映射/测试；对方回复不支持的版本则断开并给原因，不按近似字段继续。

连接输入包含连接ID、workspace/所有者、transport、精确profile、固定端点或批准的本地命令、凭据引用、公开能力提案和有限超时/重连策略。状态为 `disabled → connecting → initializing → ready → degraded/reconnecting → ready`；配置永久错误为`failed`，撤销为`revoked`。初始化握手完成前不发布工具；修正配置后可用原配置ID显式重试。一个generation内的并发断线共用一次重连过程；配置/Credential版本改变产生新generation，旧事件不能污染新连接。

采用选定profile的initialize/initialized、能力协商、分页、超时和关闭语义。只提供已实现的 `tools/list`、`tools/call`、`resources/list`、`resources/read`、`prompts/list`、`prompts/get`；每种能力独立supported/authorized/available。每页≤100项，单项/总响应受插件帧上限和明确的核心资源配额约束；大内容经获准asset发布，不无限拼接。list_changed通知只使目录失效并触发有界刷新，不能把新增工具自动加入既有授权。

工具身份为连接来源+server identity+tool name+schema digest；工具声明的readOnly/idempotent等annotations只作提案，不能代替本地策略和真实行为验收。catalog改变时旧批准参数绑定失效；待批准调用必须重新展示变更。调用前/排队后/重新连接后均核对原主体、generation、工具schema、参数摘要、授权及预算。

stdio仅启动已批准的命令和最小环境；stdout只读协议，stderr有界脱敏。网络端点沿用默认公开HTTPS、目标授权、DNS固定解析、TLS校验和Origin边界；不继承宿主代理/凭据。若部署私网必须实例政策和该连接的绑定批准同时成立。OAuth以用户明确绑定的连接为单位，state/PKCE/回调/受众和scope验证，token按目标及版本保存；刷新失败为reauthorization_required，不能转用其他用户或连接token。公共客户端无秘密也不能省略这些绑定。

连接自动恢复只恢复目录/订阅/会话连接，**不重放业务工具调用**。已发写调用丢回执为unknown，保留intent与server operation handle；有可靠只读receipt查询时核对，未提供则交给用户决定。read_only也不是无成本；查询可能收费/泄露数据，仍受授权/预算约束。重连的退避次数/上限/总deadline必须记录，永久错误不无限循环。

远端sampling、elicitation及UI资源默认不启用。若单独实现：sampling须绑定核心运行和已批预算；elicitation只能呈现实际用户决定，不能自动填秘密；UI在隔离origin/sandbox中渲染、无自动网络/导航/宿主代码权限，所有桥接操作走同一业务授权。远端返回的“请忽略权限”文本不能修改这些策略。该分支未实现时明确unsupported，而非返回空成功。

profile来源：[生命周期](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)、[传输](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)；[2026-07-28规范](https://modelcontextprotocol.io/specification/2026-07-28)仅作升级候选。核对日期2026-10-28；本合同不复制外部规范全文，也不根据文档可访问就宣称实现通过。

<a id="adapter-platform"></a>
## A-03 平台入站、群聊、媒体与可靠出站

每个连接明确平台账号、bot身份、支持方向和格式、平台侧权限、接收游标及真实状态。`platform.capabilities`至少报告send/receive、文本/文件/音视频、回复引用、撤回、长度/文件限制、真实回执等级和远端去重能力；不支持的选项禁用并显示原因。认证成功不等于群组内有发送权，配置保存不等于连接在线。当前Webhook继续只声明出站文本/HTTP accepted，不增加接收或人类已读能力。

入站用协议`platform.poll`，核心按外部稳定键映射用户、会话和消息，昵称不能合并身份。先保存事件/内容引用与新游标，提交成功才确认消费；同一event重复不再运行规划。消息同时保留稳定会话、origin、当前entry和authorized destination；Web查看外部会话不改变这些绑定。`self_echo`需要与bot身份和已知发送记录核对；引用机器人旧消息是普通用户输入，不能一律丢弃。

群/私聊分别使用与核心HTTP扩展完全相同的GatePolicy：`trigger:mention|reply_to_bot|explicit_command|always|disabled`、`member_filter:all|allowlist`、`member_external_ids:string[]`、带IANA时区的`quiet_hours`、`max_events_per_window`和`window_ms`；不另保留同义mode字段。allowlist只匹配本连接内已映射外部成员且保存时名单必须非空；空名单保存返回validation_failed，运行时遇损坏空名单则拒绝所有成员。all要求member_external_ids为空且只省去成员过滤，不授予任何新资源/模型/发送权限。always仍受成员、安静时段、速率、预算与权限门槛；disabled表示仅静默消费，不调用模型/发送，连接自身是否enabled是另一状态。reply_to_bot仅匹配真实回复关系，不能凭正文伪造。并发使用核心执行配置的Bound.max_parallel及宿主H-02队列上限；不另造GatePolicy并发字段。本版GatePolicy不做消息合并，window_ms只表示限流窗口，不能顺手当合并窗口；适配器未来若提供合并，须先发布自己的版本化settings schema和独立验收，未声明时逐事件消费。轻量门控在昂贵规划前；如果门控本身调用模型也必须记账。不回复仍记录consumed且推进游标，reply_state保持not_replied；私聊不继承无关群的安静开关。不同用户私有记忆只在该用户获准上下文使用，汇总/子代理协作仅写显式共享任务空间。

媒体入站先验证平台元数据，再下载到隔离区；校验大小、MIME/解码类型、摘要、路径/链接和资源限额，成功后发布带ACL的asset_ref。取消下载/解码只删除本次未发布临时对象，不影响同工作区并发附件。断点续传绑定来源和内容版本，远端变更失效而非拼接；原文/转换版各有摘要和所有者。模型确实读取/解码了指定内容且可回查才算媒体任务成功，插件只返回“附件路径”不算通过。

出站intent冻结route、目的地、会话、owner/service授权、参数、配置代次和operation_id；生成状态、队列状态、平台accepted、落库状态分别保存。平台拒绝/unknown时不得借不相关私聊、临时会话或另一个同用户登录继续发送。runner配置`delivery=none`只禁止该runner自动推送；独立显式工具发送仍需它自身授权/审批，不暗中继承或取消。多目标/文本分段每一段有派生segment_id和独立回执，已accepted段不能因尾段失败全部重发；缺少远端幂等能力时清楚显示保证边界。

授权撤销/群权限变化/连接禁用阻止后续新派发，已远端accepted不能伪称撤回。重启只读取持久游标、路由和回执；pending/unknown进核对队列。用户即时回复需能关联已发送内容，即使整体运行的保存阶段失败，也不能把历史当从未存在。

<a id="adapter-standalone"></a>
## A-04 独立平台MCP

保留现有`rstrbot-webhook --mcp-stdio`固定目标/秘密/私有状态目录边界。扩展到其他平台时，每个独立服务拥有其operator配置、账号scope、独立回执库和撤销入口，不借核心浏览器会话。工具只接收text/asset_ref和operation_id，不接受覆盖目标或认证字段。文本/媒体能力分别发现；跨平台身份映射由管理员明确配置，不能从用户输入推断。

持久化intent→最多一次本地派发→保存真实回执；回执保存失败同时返回真实delivery和persistence=failed。相同ID同参数只读原回执，异参数冲突；恢复时不重放unknown。备份须含自己的状态和账号配置的秘密引用，恢复默认停发并重新核对授权；认证刷新/目标改变使旧运行租约失效。独立服务通过不代表通用MCP客户端或真实平台账号已通过。

<a id="adapter-legacy"></a>
## A-05 兼容加载器：范围、固定样本与行为映射

AstrBot旧插件、旧HTTP/数据/UI、OpenCode和Pi为分别可安装的兼容组件。当前全部未实现；不将它们变成核心内置结构，不把import成功或hello world一条命令当全兼容。兼容记录有 `detected / imported / behavior_verified / recovery_verified / cleanup_verified` 五项结果，各为passed/failed/not_executed/blocked，另列unsupported_features。

每轮先建立冻结清单：ecosystem、canonical_repository、commit、release、SDK/package lock、样本相对路径/摘要、许可证、所需系统能力、已批准scope和测试日期。首批AstrBot/OpenCode/Pi三个官方未修改候选已在 [COMPATIBILITY_FIXTURES.md](COMPATIBILITY_FIXTURES.md) 固定commit/路径/字节摘要、SDK profile和许可证；来源校验已做，运行仍未执行，各样本明确列出尚不覆盖的行为。发布声明只覆盖该清单的真实样本与版本。后续尚未确定精确commit的新增样本标fixture_unfrozen，阻断该新增样本的兼容认证，不能用它模糊替代已固定样本或所有配方的完成状态。不得使用滚动main做复现标识，普通公开样本选择可由实施者按规则完成。样本代码不复制进Markdown配方，未来执行者在授权环境按固定来源获取，原样运行并核对摘要。

下面是**RstrBot拟定映射**，不是宣称上游当前行为已经在本项目运行：

| 生态与公开入口 | 映射到本项目 | 必须实测的可见结果/不支持边界 |
| --- | --- | --- |
| AstrBot metadata.yaml、Star、command过滤、MessageEventResult | 安装metadata→包身份；Context受控代理；命令→commands；回复→绑定intent | 未修改helloworld/多参数命令实际入站→回复；只允许清单公开字段，不给核心对象 |
| AstrBot LLM hooks、llm_generate/tool_loop_agent、旧工具注册 | hooks/model调用及核心run；旧对象调用白名单逐项映射 | 多轮工具ID/状态保留；真实模型夹具计数；不支持的内部成员返回明确兼容错误 |
| AstrBot消息链/媒体/后台发送 | asset_ref转换；指定外部origin→已授权route | 真实字节/目标回执；后台发送需要独立持久授权，不保留过期交互登录 |
| OpenCode Plugin工厂、event、tool.execute.before/after、自定义tool | 工厂注入受控context；event/hook→对应phase；tool→结构工具 | 输入补丁在批准前验证；client仅显式代理方法；任意Bun shell/文件访问需沙箱权限 |
| OpenCode experimental.session.compacting | summary.before和带来源的摘要提案 | 来源/归属/版本保留；实验hook版本不匹配明确失败，不回写永久历史 |
| Pi registerTool/registerCommand、on生命周期、appendEntry | tools/commands/hooks与有归属的插件数据/事件 | 四项分别验证；事件不能越权读会话；持久条目重启恢复而不自动重发 |
| Pi ctx.ui与自定义TUI | 经声明的Web/终端呈现适配能力 | 无可用UI时unsupported_ui；不能空返回假装用户同意；原生TUI不假装Web已兼容 |

公开参考：AstrBot [最小实例](https://docs.astrbot.app/dev/star/guides/simple.html)与[AI接口](https://docs.astrbot.app/dev/star/guides/ai.html)；[OpenCode插件文档](https://opencode.ai/docs/plugins/)；[Pi扩展说明](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/extensions.md)。核对日2026-10-28，旧badlogic/pi-mono URL已重定向到earendil-works/pi；此发现只用于来源识别，未替用户承诺兼容新旧所有包名或任何版本。

真实未修改样本至少覆盖：命令、LLM前/后hook、多轮工具、媒体、兼容对象代理、跨会话后台推送、禁用/卸载清理、重启恢复和拒绝越权。一个样本可覆盖多项，但每项有独立下游证据；自写合成样本只验证桥接合同，不能取代真实样本的兼容证明。缺实体平台账号时受控适配器测试须标fixture，live部分blocked。

旧HTTP适配器必须有路由账本：旧method/path、版本、认证映射、参数/分页/错误/流映射、新业务操作、实际支持状态。未列出的旧路径返回明确unsupported；绝不200+空数据伪装成功。旧令牌不成为实例管理员；旧单用户导入先选workspace/owner、预览资源清单并证明来源访问权，再事务导入；取消/失败保留源数据，新ID映射可核对，未知副作用回执导入为quarantined。

若交付旧WebUI，单独固定来源版本、许可、资产摘要和路由清单，在真实浏览器跑登录、配置、聊天、媒体、断线恢复和注销。依赖缺失/路由未映射明确blocked；可选旧界面不能成为绕过新授权和草稿隔离的入口。没有旧UI交付时仍保留需求与未实现状态。

<a id="adapter-acceptance"></a>
## 验收与结果区分

A-01对应R13-HOST-014/015；A-02对应016/017/018；A-03对应019/020/021；A-04对应022；A-05对应023/024/025。每项实际操作、故障点、下游计数和恢复要求见 [ACCEPTANCE.md](ACCEPTANCE.md)。上游closed issue是历史回归线索，当前open也不等于已在所选版本实机复现；所有兼容/竞品/真实商业平台状态均保持未执行，直到有匹配版本与环境的证据。
