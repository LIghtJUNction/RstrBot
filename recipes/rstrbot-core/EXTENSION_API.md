# 扩展 HTTP 合同 · rstrbot-http/2-draft

状态：**specified / not_implemented / not_executed**。这是下一轮生成的唯一扩展接口来源，不是当前服务端路由清单。当前 `/api/v1`、`OPENAPI.md` 的 66 个业务操作和 2 个健康操作不增删、不重解释；旧客户端不被强迫升级。此处的 HTTP 契约版本与配方版本、数据库 schema、插件 wire 协议分别管理；插件扩展选择 `1.1-draft`，不会因为 HTTP v2 就接受未知插件版本。

本合同通过有限 operation registry 与严格 DTO 给出可生成的接口。实现必须从本文件产生一份扩展 OpenAPI 和四 SDK 的同版类型，校验路由/DTO 对照后才能对外声明 available；不能在每包复制一份不同的“真实 API”。引用的业务语义在 [DOMAIN_CONTRACTS.md](DOMAIN_CONTRACTS.md)，持久语义在 `storage/DURABILITY_CONTRACT.md`。下文代码块为声明式类型，不是必须依赖的实现源码。

<a id="E01"></a>
## E01 发现、版本、错误与授权

新增 `GET /api/v2/capabilities?workspace_id=ID`。要求当前身份；未加入给定工作区返回 404。响应 `contract_version` 必须精确为 `rstrbot-http/2-draft`，并提供 `supported_contracts`、`server_instance_id`、`implementation_revision`、`capabilities[]` 和有限 `limits`。没有 v2 的旧服务返回旧服务器真实 404；客户端显示扩展不可用，不能猜测端点后发写操作。

每个能力项为 `{id, supported:boolean, authorized:boolean, available:boolean, reason_code:string|null, operation_names:string[], limits:object}`。含义分别是服务实现了、调用者获准、当前前提可用；available 只能在前两项为真且依赖就绪时为真。声明设计稿不等于 supported。无权字段不能泄露私有资源/配置；能力信息不是操作授权缓存，每次调用仍复查。

当前会话 Cookie/Bearer/Origin/Fetch-Site/浏览器 mutation 头保护继续应用。新增个人/服务凭据采用显式 `RstrBotCredential <opaque>` scheme，不把当前 Bearer token 改成独立 API key。凭据类型由服务端哈希记录确定，不能从 token 内自述权限。v2 写入携带 `X-RstrBot-Contract: rstrbot-http/2-draft`；不匹配返回 409 `contract_mismatch`。版本协商失败不得静默降级到会改变副作用的旧动作。

所有 JSON 对象默认拒绝未知字段；ID 为不透明 UTF-8 1..128 bytes，不能含 NUL、控制字符或 `/`，名称不充当 ID。时间用 UTC RFC3339，duration 使用整数毫秒，金额使用整数微美元；既有 v1 的 Unix 秒格式保持不变。配置文本按字段字节数校验，不能用字符数代替 UTF-8 字节数。JSON 请求上限 256 KiB（文件分块例外）；错误响应上限 8 KiB；列表 `limit` 为 1..100，默认 50。部署者可降低这些新合同上限，能力响应必须给实际值。

统一 v2 错误：`{error:{code,message_key,request_id,retryable,details?}}`；details 仅字段错误或可公开限制，不放凭据、私聊内容、原始远端错误/URL 查询。稳定代码/状态：400 validation_failed/unknown_operation；401 unauthenticated/credential_revoked；403 permission_denied/approval_required；404 not_found；409 version_conflict/idempotency_conflict/contract_mismatch/generation_mismatch/reconciliation_required；410 cursor_expired/asset_expired；413 limit_exceeded；422 unsupported_media/unsupported_mapping；429 capacity_exceeded/budget_exceeded；503 capability_unavailable/storage_unavailable。`retryable` 仅指重新取得状态或允许的新准入，不授权重放写操作。

不存在、越权资源、私有对象 ID 猜测采用同一 404；操作当前权限不足且无需暴露对象时可用 403。跨入口都返回同一领域 code，显示文案可本地化。取消不是一般 provider error；本地 ACL、持久化、幂等和参数错误不触发供应商回退。

<a id="E02"></a>
## E02 动作与查询端点

| 方法与路径 | 输入 | 输出及性质 |
| --- | --- | --- |
| `GET /api/v2/openapi.json` | 合同版本 | 本文件生成的扩展 schema；不复制 v1 对象 |
| `GET /api/v2/capabilities` | 见 E01 | 能力三态/限值；只读 |
| `POST /api/v2/workspaces/{wid}/operations` | `Command`，见 E03/E05 | 异步 durable accepted 返回 202 `CommandResponse`；一次秘密签发同步完成返回201；同键同语义返回200原Operation且ephemeral_output=null；验证失败返回错误 |
| `GET /api/v2/workspaces/{wid}/operations/{oid}` | oid | 当前可读 `Operation`；只读、绝不派发 |
| `POST /api/v2/workspaces/{wid}/operations/{oid}/cancel` | `{expected_revision, idempotency_key}` | 202/200 Operation；设置取消意图，保留实际外部结果 |
| `POST /api/v2/admin/operations` | `Command`，仅`backup.prepare`和E08的package管理动作 | 实例管理员+最近认证+相应维护/安装边界；202 CommandResponse；实例范围，不伪装工作区授权 |
| `GET /api/v2/admin/operations/{oid}` | oid | 当前实例管理员只读运维Operation；不含私聊正文/凭据 |
| `POST /api/v2/admin/operations/{oid}/cancel` | `{expected_revision,idempotency_key}` | 实例管理员取消自己合法创建且仍可取消的备份/包下载/准备作业；保留旧active及已提交激活事实，不能取消任意工作区私有run |
| `GET /api/v2/admin/backup-jobs/{id}` | job_id | 实例管理员读取BackupJob范围/缺项/摘要；实际备份下载遵循独立运维入口 |
| `GET /api/v2/admin/objects/{kind}` | E08 admin kind，limit/cursor | 当前实例管理员可读列表；只读，不触发下载/安装 |
| `GET /api/v2/admin/objects/{kind}/{id}` | E08 admin kind/id | 当前实例管理员读取安装进度/来源/摘要/许可/权限差异；只读，无原秘密 |
| `GET /api/v2/workspaces/{wid}/objects/{kind}` | `limit,cursor?` | `{items:Record[],next_cursor,snapshot_revision}`；先 ACL 后分页/计数 |
| `GET /api/v2/workspaces/{wid}/objects/{kind}/{id}` | ID | `Record`；只读，服务端拥有状态字段 |
| `GET /api/v2/workspaces/{wid}/events` | `after?,limit?` | `{items:DomainEvent[],cursor}`；当前 ACL；过期 410，客户端完整重同步 |
| `POST /api/v2/workspaces/{wid}/configuration/resolve` | `ResolveInput` | `ResolvedConfig`；纯只读，任何付费/发送计数为零 |
| `POST /api/v2/workspaces/{wid}/schedules/preview` | `{schedule:Ref\|null,draft:ScheduleDraft}` | `SchedulePreview`；已有计划读当前paused，新草稿paused=true；只读，与实际调度同引擎，无保存/认领/费用 |
| `GET /api/v2/workspaces/{wid}/schedules/{id}/occurrences` | limit/cursor | `{items:[{occurrence_id,planned_instant,state,run_ref,delivery_refs}],next_cursor}`；当前计划读权/私有权，每项明确manual或scheduled来源 |
| `POST /api/v2/workspaces/{wid}/retrieval/query` | `RetrievalInput` | `RetrievalResult`；可能使用有费用的重排/嵌入，须显式模式/预算，不能当无副作用普通 GET |
| `GET /api/v2/workspaces/{wid}/uploads/{id}` | upload_id | `UploadState` 含已收块索引/摘要，无裸存储路径 |
| `PUT /api/v2/workspaces/{wid}/uploads/{id}/chunks/{index}` | bytes；`Content-SHA256`、`Content-Length` | 200 `{index,sha256,size_bytes}`；同块同摘要重试幂等，异摘要 409；只有创建者当前授权可写 |
| `GET /api/v2/workspaces/{wid}/assets/{id}/content` | version；可选单一有效 Range | 200/206 已授权 bytes，内容类型/长度/摘要；禁止跳转公共永久 URL |
| `GET /api/v2/workspaces/{wid}/assets/{id}/derivatives/{derivative_id}` | version | 同上；派生物独立检查父引用 ACL，禁止通过缩略图越权 |
| `POST /api/v2/workspaces/{wid}/credential-bindings/{id}/oauth/start` | `{expected_revision,idempotency_key}` | 当前本人/服务管理者+最近认证；201 `{intent_id,authorization_url,expires_at}`，no-store；固定供应商登记的回调，无任意redirect输入 |
| `GET /api/v2/provider-oauth/callback` | 供应商返回`code,state`或固定错误字段 | 一次state/浏览器绑定/原会话和配置版本验证，受控交换；303到固定同源状态页，不在URL带token；不要求跨站发送普通Strict会话Cookie |
| `GET /api/v2/shares/{share_id}/resources` | 当前会话或合法`RstrBotCredential` + `X-RstrBot-Share-Token`，limit/cursor | 仅Share资源白名单的只读ReadRef列表；每次核对命名audience/成员/来源ACL/到期/撤销 |
| `GET /api/v2/shares/{share_id}/resources/{kind}/{id}` | 同上 | 只读授权投影，正文/消息须在明确共享范围内；不能借kind走管理接口 |
| `GET /api/v2/shares/{share_id}/assets/{id}/content` | 同上 + asset version/可选Range | 只有Share显式包含的asset bytes；父会话分享不隐含附件；不重定向公共URL |

工作区对象 kind 的封闭集合为 `service_principals, personal_credentials, service_credentials, credential_bindings, execution_profiles, approvals, delegated_runs, shared_tasks, channel_connections, uploads, assets, knowledge_libraries, knowledge_sources, knowledge_jobs, index_generations, memory_consents, derived_memories, workflow_definitions, workflow_runs, shares, reconciliation_cases, runs, schedules, tool_intents, delivery_intents, conversations, models, agents, routes, plugin_instances, plugin_data_exports, mcp_connections, mcp_catalogs`。`backup_jobs`和E08列出的package对象只在独立admin命名空间。未实现 kind 返回 capability_unavailable，不返回空成功。这里提供只读投影，不能把 runs/deliveries 等服务端事实开放为通用可写 JSON。

`conversations/models/agents/routes` 是既有v1对象的v2只读适配投影，消息读取还用 `GET /api/v2/workspaces/{wid}/conversations/{id}/messages?limit&cursor`；读取入口接受当前会话或明确授权的v2凭据，不能把v2凭据偷偷送进只认会话的v1。底层对象身份不复制，v1.version精确映为Ref.revision、Unix秒映为RFC3339；字段/枚举/secret脱敏沿用OPENAPI.md，不在此重定义。conversation列表不带完整历史，消息页按当前ACL和稳定cursor读取。`runs/schedules` 对当前v1部分采用同一只读桥接，扩展字段按E04附加，并明确capability/contract_version；旧对象缺扩展字段时为null/不支持，不能捏造default即已支持。人类会话仍可用v1创建当前配置；服务凭据只操作显式grant的对象，不自动获得v1全管理能力。

列表 cursor 为服务端签名/存储的不透明 `(workspace, principal, query, sort, upper_bound, position, expiry)` 引用；变工作区或查询复用拒绝。排序固定 `(created_at,id)`；并发变化用 snapshot_revision 告知，已经被删/撤权对象即使属于旧快照也不返回。建议只读缓存不能作为授权来源。

<a id="E03"></a>
## E03 共同 DTO 与状态

```typescript
type Ref = { id: string; revision: number }; // revision 为1..9007199254740991整数
type ReadRef = Ref & { kind: string; scope: "workspace" | "instance" };
type Subject = { type: "user" | "service"; id: string };
type Visibility = "private" | "workspace";
type Effect = "read_only" | "idempotent_write" | "non_replayable_write";
type Bound = { max_steps: number; max_attempts: number; deadline_at: string;
  max_cost_microusd: number | null; cost_mode: "strict" | "estimated";
  max_children: number; max_depth: number; max_parallel: number };
type AssetRef = { asset_id: string; version: string; sha256: string;
  media_type: string; size_bytes: number };
type Content = { type: "text"; text: string } |
  { type: "asset"; asset_ref: AssetRef } |
  { type: "quote"; message_id: string; component_id: string };
type Command = { operation_name: string; idempotency_key: string;
  expected_versions: Ref[]; params: OperationParams };
type Record = { id: string; kind: string; workspace_id: string | null; owner: Subject;
  visibility: Visibility; revision: number; created_at: string;
  updated_at: string; state: string; data: TypedRecordData };
type Operation = { id: string; operation_name: string; workspace_id: string | null;
  actor: Subject; revision: number; input_digest: string; target_refs: Ref[];
  status: "queued" | "running" | "completed" | "failed" | "cancelled" |
    "interrupted" | "unknown";
  phase: "accepted" | "preparing" | "waiting_approval" | "dispatching" |
    "waiting_result" | "persisting" | "reconciling" | "finished";
  created_at: string; updated_at: string;
  output_refs: ReadRef[]; error_code: string | null;
  execution: "not_started" | "running" | "succeeded" | "failed" | "cancelled" | "unknown";
  delivery: "not_applicable" | "not_sent" | "accepted" | "rejected" | "unknown";
  persistence: "committed" | "failed" | "reconciliation_required";
  budget: { reserved_microusd: number; actual_microusd: number | null;
    coverage: "known" | "partial" | "unknown" };
  reconciliation_required: boolean; event_cursor: string };
type EphemeralOutput =
  { type: "credential_token"; credential_ref: Ref; token: string } |
  { type: "share_token"; share_ref: Ref; token: string };
type CommandResponse = { operation: Operation; ephemeral_output: EphemeralOutput | null };
type DomainEvent = { id: string; cursor: string; workspace_id: string;
  resource_ref: Ref; run_id: string | null; generation: number | null;
  operation_id: string | null; attempt_id: string | null; kind: string;
  occurred_at: string; recorded_at: string; data: BoundedEventData };
```

`Record.data` 必须与 kind 的下列 DTO 匹配，不能任意 JSON 混用。`OperationParams` 必须与 E05 中 operation_name 的行匹配。`AssetRef.version` 是opaque内容版本字符串（1..128 UTF-8 bytes），与记录CAS `Ref.revision` 不同；HTTP/worker/四SDK原样传递和比较，禁止强转数字。sha256为64位小写十六进制，size_bytes为0..9007199254740991整数；其他公开整数同样有safe范围，溢出拒绝。正整数限值有服务端上限；`Bound.max_cost_microusd=null` 只允许明确批准的 estimated 模式且不能表示免费。集合长度、总字节和深度必须在能力 limits 内；建议初始生成值为 `max_steps=16,max_attempts=1,max_children=4,max_depth=4,max_parallel=4`，它们是可降低的设计默认值，不是性能达标数据。deadline 必须是未来、且不超过管理员策略；没有值时不得执行。

`credential.issue` 与 `resource.share` 的秘密由核心在准入事务提交前生成，只持久hash/指纹；这两项必须同步提交completed Operation后，以201的`ephemeral_output`一次返回，响应`Cache-Control:no-store`，不能排队后要求用户GET取secret。任何GET、重复幂等键、事件、审计、导出都不带ephemeral_output。若首次响应丢失，只读核对确有对象后撤销并以新键重新签发；不重新泄露旧token、不因重试产生第二活跃凭据。其它动作ephemeral_output恒null。admin操作workspace_id=null，不能由工作区endpoint派发；其幂等scope以instance取代workspace。

`ReadRef.kind` 必须是E02/E08封闭集合，不能自由命名。workspace引用固定读 `objects/{kind}/{id}`，instance引用为`backup_jobs`时读admin/backup-jobs/{id}，为E08包对象时读admin/objects/{kind}/{id}；调用者不能提供任意read URL。这样审批提案、schedule、逐目标delivery和run均可刷新后读回，不依赖已经丢失的POST响应。`target_refs`的kind由operation输入字段定义，输出一律补明确kind/scope。

OAuth start保存短期意图：原actor/session或服务管理会话、binding revision、固定provider/endpoint、PKCE S256 verifier、state、nonce（OIDC时）、固定回调及到期；浏览器另有短期HttpOnly Lax绑定Cookie。callback仅走这个窄例外，不能关掉其它Origin/CSRF保护。核心按已登记提供方元数据验证TLS、code/PKCE与签名/issuer/audience/nonce（若提供ID token），确认原凭据仍有效和binding revision未变，原子消费intent后才把pending→verified→active；失败/取消/过期/重启丢失绑定不替换旧active。凭据只加密存储，不在URL/日志/事件出现。直接secret replace也先完成目标/格式验证，任何可收费测试须另作明确操作，不用静默收费检测来激活。此处是供应商绑定，不能覆写现有Google用户登录/邮箱合并规则。

金额桥接：worker `Usage.provider_charge` 的原金额十进制string与currency原样保存，来源由调用身份记录；price_revision来自核心准入时冻结价目，不向strict worker DTO臆加同名字段。仅核心做十进制精确运算，四SDK不使用浮点估价。USD非负金额按`ceil(amount × 1000000)`入整数微美元预算账本，同时保留raw金额与rounding_delta；这是不低估预算的最小单位取整，不声称原币账单精确等于取整值。非法/负数金额另记诊断（退款需独立经验证调整事实），溢出safe整数时保留raw且actual_microusd=null/coverage=unknown，不截断为0。非USD没有冻结可审计汇率/价目版时也保留原币raw+unknown；若明确启用换算，保存汇率decimal、来源、采集时间、rate_revision和同一取整算法，不能用实时波动率重写历史费用。unknown不得因为取消、转换失败或撤权而释放成0费用。

Share token只能经上述专用header提交，不能放URL、查询参数、错误或普通日志；前端链接只含share_id，token通过明确的独立粘贴/安全传递渠道输入并仅驻内存，页面刷新需再次提供或重新授权，不写匿名全局草稿。token本身不等于用户身份：必须有当前会话或合法RstrBotCredential且主体在Share.audience；user须仍是有效workspace成员，service须仍active且其当前独立grant包含该只读资源。资源所有者/来源ACL与share revision同样有效；token不扩大任一主体的grant。没有token、非受众、已撤销/过期均404，不反映资源存在。匿名公开/无认证iframe共享当前为unsupported；嵌入需受控同源上下文且沿用上述只读请求。分享不包含模型密钥/管理配置；允许kind仅`conversations,assets,knowledge_sources,derived_memories`，每个条目精确列明版本和消息范围，owner更新范围必须新share，不用token变更意图。

phase使用上列封闭枚举。DomainEvent.kind仅为`operation.accepted/operation.phase/operation.terminal/resource.changed/grant.revoked/source.invalidated/receipt.observed/run.delta/tool.progress/platform.progress/task.progress`；data严格对应如下：operation事件为`{operation_ref:ReadRef,status,phase,error_code}`，resource/grant/source事件为`{resource_ref:ReadRef,reason_code}`，receipt事件为`{intent_ref:ReadRef,delivery,persistence,reconciliation_required}`，run.delta为`{run_ref:ReadRef,sequence:安全整数,text:string}`，progress为`{operation_ref:ReadRef,sequence:安全整数,completed:number|null,total:number|null,message_key:string}`。所有内容限已授权对象/有限字节，不能把自由字符串当新能力。核对结果追加事实/修订而非修改已有事件；一个run/generation只提交一个执行终态，delivery/persistence后续可以增加可验证的独立事实。

幂等键为调用者提供的 1..128 ASCII 字符；作用域 `(workspace, actor, operation_name, key)`。服务端持久记录规范化输入摘要，包含所有语义参数、对象版本、目标/配置与内容摘要；规范化使用明确的字段排序和 UTF-8，不能把日志脱敏正文用于计算业务摘要。校验和权限通过后，准入意图、幂等记录和预算预留在一个事务中写入。相同键不同语义 409；相同键同语义只读原操作且重新授权。操作未完成/unknown 也不能重复派发；幂等记录在外部后果仍可能存在时不得自动过期为“新请求”。

<a id="E04"></a>
## E04 扩展对象的严格数据形状

以下列出**可写字段**和服务端输出字段；没有列出的字段不可由客户端写入。表中的 Ref/Subject/AssetRef 使用 E03；所有账户/授权句柄都是同实例 opaque ID。

| DTO / kind | 可写字段 | 服务端事实/状态与不变量 |
| --- | --- | --- |
| `Grant` | `operation_names:string[], resource_refs:Ref[], expires_at, max_cost_microusd?, max_uses?` | 不接受通配符 `*`；同工作区；resource_refs 可为空仅限不读资源的创建动作；必须是授予者可授范围 |
| `ServicePrincipal` | `name, custodian_user_id, grants:Grant[], expires_at` | subject.type=service；active/disabled/revoked；credential_revision；负责人不等于被冒充的执行身份 |
| `CredentialIssueInput` | `target:Subject, name, grants:Grant[], expires_at` | 仅签发输入，target=user只允许本人；target=service需管理该服务且grant不能超其当前范围 |
| `PersonalCredential` | 由CredentialIssueInput(target=user)派生，不能通用写 | owner固定当前user，target类型user；只存hash，读取仅name/范围/到期/前缀指纹；active/revoked/expired |
| `ServiceCredential` | 由CredentialIssueInput(target=service)派生，不能通用写 | owner/target固定service；issued_by记录实际管理员，不冒充user；其他状态/秘密边界同PersonalCredential |
| `CredentialBinding` | `provider_id, endpoint, external_subject?, scope:string[], credential_source:{type:replace,value:string}\|{type:clear}\|{type:retain}` | pending/active/auth_required/revoked；secret 只写；retain 仅目标完全一致；外部主体来自已验证回调/签名，不接受自报为已验证 |
| `ExecutionProfile` | `layers:{level:instance\|workspace\|agent\|conversation,ref:Ref}[], overrides:FieldOverride[], model_candidates:Ref[], bounds:Bound` | resolved_digest、每字段 effective_value/source、capabilities；权限/限额使用交集/最小值 |
| `FieldOverride` | `field:system_prompt\|examples\|model\|tools\|knowledge\|memory\|quiet_hours, mode:inherit\|value\|clear\|disabled, value?` | value 只在 mode=value 存在且匹配字段类型；examples 为有来源文本数组，不是真实消息 |
| `Approval` | 建立由工具意图派生；decision 仅 `approve\|reject` | proposed/approved/rejected/expired/revoked/consumed；绑定intent_digest/tool/目标/版本/主体/费用/有效期；不能改参数 |
| `RunRead` / runs | 只读 | `conversation_ref,generation,operation_ref,profile_ref,status,stage,attempt_refs,child_run_refs,event_cursor,execution,delivery,persistence,budget`；附加字段仅在真实扩展run存在，正文/消息按ACL分页 |
| `ScheduleRead` / schedules | 只读；由schedule.save/set_enabled修改 | `draft:ScheduleDraft,enabled,next_occurrence,revision,owner,occurrences:[{occurrence_id,planned_instant,state,run_ref,delivery_refs}],needs_reconciliation`；occurrences另分页，不隐式返回全部历史 |
| `ToolIntentRead` / tool_intents | 只读；由tool.propose/execute驱动 | `run_ref,tool_instance,tool_name,arguments_redacted,resource_refs,effect,intent_digest,approval_ref,bounds,state:proposed\|prepared\|sending\|accepted\|rejected\|not_sent\|unknown,receipt_ref`；审批者有权的实际参数可读，秘密仅存在标记 |
| `DeliveryIntentRead` / delivery_intents | 只读；由delivery.prepare/dispatch驱动 | `target_route_ref,target_conversation_ref\|null,content_refs,content_digest,operation_ref,approval_ref,planned_instant\|null,execution,delivery,persistence,remote_receipt_ref\|null,reconciliation_required`；每目标一个Record，unknown不隐藏 |
| `DelegatedRun` | `parent_run_id, task, context_message_ids:string[], assets:AssetRef[], requested_grant:Grant, bounds:Bound, mode:foreground\|background, failure_policy:fail_fast\|collect, result_target:{type:parent\|shared_task,id:string}` | child_id/parent_id/effective_grant/预算子预留；默认背景不解除原租约，持久后台需显式 grant |
| `SharedTask` | `title, participants:Subject[], resource_refs:Ref[], purpose, expires_at` | active/revoked/completed；每参与者pending/accepted/revoked、consent_revision和获准引用集合；邀请不等于同意，不自动导入私人上下文 |
| `ChannelConnection` | `adapter_instance:Ref, platform_account_id, credential_binding:Ref, settings:object, gate:GatePolicy` | generation/cursor/connected\|connecting\|degraded\|auth_required\|disabled；settings 必须匹配已装包公开schema，不能带任意executable |
| `GatePolicy` | `trigger:mention\|reply_to_bot\|explicit_command\|always\|disabled, member_filter:all\|allowlist, member_external_ids:string[], quiet_hours:{timezone,start_local,end_local}[], max_events_per_window:number, window_ms:number` | 与host完全同名枚举；allowlist必须非空，all时列表为空；disabled只静默消费，不等于禁用连接；always仍受成员/安静时段/速率/预算限制；私聊另有policy |
| `Upload` | `name, media_type, expected_size_bytes, expected_sha256, target_ref:Ref, visibility` | created/uploading/verifying/published/cancelled/failed/expired；chunk_size/max_total/已收块；范围不宽于target |
| `Asset` | 由 upload.complete 发布；不接受通用 create | asset_ref、owner/ACL、source/derivative refs、published/deleted/quarantined；只读带授权bytes |
| `KnowledgeLibrary` / knowledge_libraries | `name,visibility`，可见性改变仍走显式sharing动作 | owner/revision、active/deleted、active_index_generation_ref\|null；v1知识不自动混入共享库，导入须选明确归属 |
| `KnowledgeSourceRead` / knowledge_sources | 只读；由导入/replace/delete驱动 | `library_ref,source_version:string,source_kind:asset\|url,source_label,original_asset_ref:AssetRef\|null,normalized_text_asset_ref:AssetRef\|null,parser_id,parser_revision,source_sha256,chunk_refs,index_generation_ref\|null,state:staged\|parsing\|indexing\|ready\|failed\|cancelled\|deleted,previous_version_ref\|null,error_code`；staged/抓取失败可无asset，解析失败可无normalized；ready必须两个非null引用及摘要齐全；URL导入也保存版本化原文字节asset，不只留下可变URL |
| `KnowledgeJob` | `library_ref:Ref, sources:({asset_ref:AssetRef}\|{url:string})[], parser_id, index_profile:Ref` | 每项 staged/parsing/indexing/ready/failed/cancelled/deleted；source/version/chunks/index generation、逐项错误 |
| `IndexGeneration` | `library_ref:Ref, embedding_profile:Ref\|null, chunker_revision, sparse_backend_id\|null` | building/ready/active/failed/retired；model/revision/dimensions/source版本/检查点/旧active指针 |
| `MemoryDeriveInput` | `source_message_refs:Ref[], kind:summary\|memory, enabled, archived, pinned, consent_ref:Ref\|null` | 输入不接受已生成content；自动memory需consent且当前访问许可 |
| `MemoryConsent` / memory_consents | `source_refs:Ref[],allowed_kinds:(summary\|memory)[],purpose:string,automatic:boolean,expires_at` | owner固定当前本人或明确受托服务；active/revoked/expired、revision；仅列来源/用途可派生，默认automatic=false |
| `DerivedMemory` | 派生创建后只由memory.update/delete改变明确字段 | content为生成输出，保存模型/模板/generation/source_digest及valid/invalidated/deleted；人工编辑保留原来源并记录edited_by/revision，不伪装未修改模型输出 |
| `WorkflowDefinition` | `name, nodes:Node[], edges:Edge[], failure_policy:fail_fast\|collect, bounds:Bound` | draft/validated/active/retired；E09严格DAG/字段映射/条件/join；配置版本不可变快照 |
| `Node` | `id,type:read\|model\|approved_tool\|branch\|join\|subagent\|deliver,input:NodeInput,resource_refs:Ref[]` | type/input判别联合、Edge和DataRef均由E09独占定义；不得使用任意eval/JS/shell表达式 |
| `WorkflowRun` | `definition:Ref, mode:execute\|validate\|simulate\|controlled_read, input:object, bounds:Bound` | 每节点 operation_ref、状态/错误/输出ref；生产副作用只限execute且逐节点当前审批 |
| `Share` | `resource_refs:ReadRef[], message_ranges:{conversation_id,from_message_id,to_message_id}[], audience:Subject[], expires_at, mode:read` | active/revoked/expired；种类/版本与消息范围固定；未列附件不隐式分享；无模型/写权限；token一次返回并仅header消费 |
| `BackupJob` | `mode:database\|complete, destination_label, external_state_policy:require_complete\|allow_incomplete` | prepared/snapshotting/verifying/ready/failed/cancelled；清单/摘要/范围/缺项；destination_label不是任意宿主路径 |
| `ReconciliationCase` | 仅核心自动创建；手工提交 `evidence_refs:string[], observed:accepted\|rejected\|not_sent\|unknown, note` | evidence来自已授权只读观察；append-only；不能以手写 accepted 代替可靠远端证明；status unresolved/resolved |

object 修改用 Ref 版本 CAS；没有通用任意 kind 的 create/update/delete 路由。配置/文本长度取能力 limits，未声明上限视能力不可用；文档/音视频解码限制独立于传输 bytes。`settings`、`input` 和 `mapping` 必须先通过固定版本 schema/类型检查；即使是 JSON object 也不允许未知能力或任意代码。

<a id="E05"></a>
## E05 封闭 operation registry

动作封闭集合为本节与E08表的并集，经 E02 的 POST operations；`backup.prepare`和E08显式admin动作仅走admin命名空间，其余仅走workspace命名空间，错用scope拒绝。查询不走此入口。每行 effect 是核心动作的外部后果分类，生成实现还须按具体工具进一步收紧。输出中token按E03一次秘密响应，永不持久化为可读明文；网络丢失后只能撤销并重新签发，不能再次读出秘密。所有敏感写操作要求当前权限、CAS、适用的最近认证和不可绕过的网络/预算政策。

| operation_name / capability | `params` 严格形状 | 权限、效果、输出与恢复 |
| --- | --- | --- |
| `principal.create` / `identity.service.v1` | `{principal:ServicePrincipal}` | admin+明确可授grant；内部幂等写；输出主体Ref；失败不产生半个主体 |
| `principal.update` / 同上 | `{principal:Ref,data:ServicePrincipal}` | admin+当前可授grant；CAS改变负责人/权限/到期，grant revision提升并取消越界派发；输出Ref；不能借更新冒充个人 |
| `principal.revoke` / 同上 | `{principal:Ref}` | admin；撤销主体/全部凭据/未派发授权；输出Ref；在途unknown仍保存 |
| `credential.issue` / `identity.tokens.v1` | `{credential:CredentialIssueInput}` | 本人最近认证，或admin为服务签发；不能给他人个人token；按类型保存个人/服务凭据；201输出Ref+一次token |
| `credential.revoke` / 同上 | `{credential:Ref}` | 本人或有相应服务管理权；输出Ref；不以同user其他token续命 |
| `binding.upsert` / `credential_proxy.v1` | `{binding:Ref\|null,data:CredentialBinding}` | 拥有者/获准admin且最近认证；代理初始化可能外部认证，返回pending/active；目标变化禁retain |
| `profile.save` / `configuration.layers.v1` | `{profile:Ref\|null,data:ExecutionProfile}` | 本人private或获准共享配置管理；内部CAS；输出配置Ref和digest，不改在途快照 |
| `resource.share` / `sharing.scoped.v1` | `{share:Share}` | 资源所有者且可授全部引用；输出Ref+一次只读token；不能暗含附件 |
| `resource.unshare` / 同上 | `{share:Ref}` | 原所有者或授权撤销者；输出revoked Ref；后续读取失效 |
| `run.create` / `runs.extended.v1` | `{conversation:Ref,generation:number,content:Content[],profile:Ref,bounds:Bound}` | 当前会话写权，所有内容同ACL；外部模型可收费；输出run Ref，原历史/版本不被覆盖 |
| `tool.propose` / `tools.approval.v1` | `{run:Ref,tool_instance:Ref,tool_name:string,arguments:object,resource_refs:Ref[],effect:Effect,bounds:Bound}` | 可调用者；arguments校验公开schema，核心复核effect；只持久提案，不执行；输出intent/approval Ref |
| `approval.decide` / 同上 | `{approval:Ref,decision:approve\|reject,intent_digest:string}` | 对绑定目标有批准权者，最近认证按效果政策；内部写；输出approval Ref，不自动派发 |
| `tool.execute` / 同上 | `{intent:Ref,approval:Ref\|null}` | 当前原执行主体、原租约、有效批准；按effect；输出操作/结果Ref；unknown不可再次执行 |
| `task.share` / `tasks.delegate.v1` | `{task:SharedTask}` | 对引用可授范围成员；内部写；输出共享任务Ref；参与者需显式接受才读取私有来源 |
| `task.consent.accept` / 同上 | `{task:Ref,participant:Subject,resource_refs:Ref[],purpose_digest:string,expires_at:string}` | participant必须是当前本人，或当前有明确委托的服务主体；CAS同意目的/引用/到期，不能替别人同意；输出task Ref与consent revision |
| `task.consent.revoke` / 同上 | `{task:Ref,participant:Subject}` | 同意者本人或明确服务管理者；撤销该参与者引用授权及后续子任务；输出task Ref，已知结果/费用仍保留 |
| `task.delegate` / 同上 | `{delegation:DelegatedRun}` | 仅父run当前可授主体；外部模型/工具预算子划分；输出child Ref；重复只读同child |
| `connection.save` / `platform.receive.v1` | `{connection:Ref\|null,data:ChannelConnection}` | 工作区admin且实例已安装可信adapter；只保存disabled；输出Ref；不即刻连接外部平台 |
| `connection.set_enabled` / 同上 | `{connection:Ref,enabled:boolean}` | 已批准平台身份/目标和当前operator授权；开启可能调用真实平台；输出Ref/generation；旧代停止不覆盖新代 |
| `upload.create` / `media.refs.v1` | `{upload:Upload}` | 对target有写权且资源配额足够；内部预留；输出upload Ref/块大小/到期 |
| `upload.complete` / 同上 | `{upload:Ref}` | 创建者且同target当前写权；验证/解码无生产外发；输出asset Ref或验证失败，不发布半文件 |
| `upload.abort` / 同上 | `{upload:Ref}` | 创建者；释放未发布块/配额；输出cancelled Ref；不删除已发布对象 |
| `asset.delete` / 同上 | `{asset:Ref}` | 所有者或明确管理授权；先tombstone后GC；输出Ref，派生读取同时拒绝 |
| `library.save` / `knowledge.jobs.v1` | `{library:Ref\|null,data:KnowledgeLibrary}` | 当前库所有者/新库创建者；CAS，默认private；输出Ref；不能通过更新visibility绕显式分享 |
| `library.delete` / 同上 | `{library:Ref}` | 所有者；tombstone库与派生可见性，取消后台任务；输出Ref，保留必要未知费用记录 |
| `knowledge.import` / `knowledge.jobs.v1` | `{job:KnowledgeJob}` | 获准库写权/来源读权，URL网络授权；解析/嵌入可能收费须配置Bound通过引用profile；输出job Ref/逐项结果 |
| `source.replace` / 同上 | `{source:Ref,input:{asset_ref:AssetRef}\|{url:string},parser_id:string,index_profile:Ref,bounds:Bound}` | 文档所有者/明确编辑授权、旧revision CAS；新source_version异步准备，未完成前旧ready继续或显式不可用；输出source/job ReadRefs；来源变更使旧派生失效，新内容不混旧索引 |
| `source.delete` / 同上 | `{source:Ref}` | 当前来源所有者；tombstone源/引用/派生读权限，取消后台任务；输出deleted ReadRef，不等价于删整库或任意asset |
| `index.rebuild` / `knowledge.index.v1` | `{generation:IndexGeneration,bounds:Bound}` | 库管理者；新代构建/预算；输出Ref，旧active不变 |
| `index.activate` / 同上 | `{generation:Ref,expected_active:Ref\|null}` | 库管理者；仅ready且当前来源/ACL完整；原子切换，输出active Ref |
| `memory.derive` / `memory.derived.v1` | `{memory:MemoryDeriveInput,profile:Ref,bounds:Bound}` | 可读所有来源且写目标；生成可能收费，自动memory需同意；输出Ref/content/来源范围，过期来源不发布 |
| `memory.consent.grant` / 同上 | `{consent:MemoryConsent}` | 来源所有者/当前本人或明确服务管理授权；目的/来源白名单/到期显式同意；输出Ref；不得替其他人私聊授权 |
| `memory.consent.revoke` / 同上 | `{consent:Ref}` | 原同意者或明确服务管理者；CAS撤销并停止后续自动派生/在途后续调用；输出Ref；删除既有memory走明确delete，不假称远端撤回 |
| `memory.update` / 同上 | `{memory:Ref,content?:string,enabled?:boolean,archived?:boolean,pinned?:boolean}` | 所有者且当前可见，至少一个字段；CAS保留来源/编辑者，归档不等删除；输出新Ref，不无提示调用模型 |
| `memory.delete` / 同上 | `{memory:Ref}` | 所有者；tombstone/派生失效先提交，物理GC后做；输出deleted Ref，不自动重建 |
| `schedule.save` / `schedules.extended.v1` | `{schedule:Ref\|null,draft:ScheduleDraft}` | 仅计划所有者；保存disabled，当前owner未来执行权限；输出Ref和preview，不认领或外发 |
| `schedule.set_enabled` / 同上 | `{schedule:Ref,enabled:boolean,acknowledged_case_ids:string[]}` | 仅所有者；存在未决case时只能暂停或明确核对；启用只排未来，不重放旧unknown |
| `schedule.run_once` / 同上 | `{schedule:Ref,bounds:Bound}` | 仅计划所有者且当前执行权限；独立manual occurrence/新意图，不消费或覆盖自动planned instant；输出run及逐目标intent ReadRefs，仍逐目标审批/dispatch |
| `schedule.delete` / 同上 | `{schedule:Ref,cancel_active:boolean}` | private仅所有者，共享计划允许已有明确管理删除权；先停未来认领/tombstone；cancel_active只取消在途后续动作，已接受/unknown回执保留；输出deleted ReadRef |
| `delivery.prepare` / `delivery.multitarget.v1` | `{run:Ref\|null,targets:Ref[],content:Content[],bounds:Bound}` | 每目标明确路由权；只持久逐目标意图；输出delivery intent Refs；不发送 |
| `delivery.dispatch` / 同上 | `{intent:Ref,approval:Ref\|null}` | 原主体/委托、原目标版本启用且批准；non_replayable_write除非远端验证幂等；输出真实逐目标回执 |
| `workflow.save` / `workflows.v1` | `{definition:Ref\|null,data:WorkflowDefinition}` | 本人/共享配置权；纯验证+CAS，输出Ref和逐节点错误；无生产调用 |
| `workflow.set_state` / 同上 | `{definition:Ref,state:active\|retired}` | 定义所有者/授权配置者；仅validated可active，retired不再准入新run；旧在途按冻结版本与当前权限继续或显式取消 |
| `workflow.run` / 同上 | `{run:WorkflowRun}` | execute需真实操作授权；其他模式受D08限制；输出workflow run Ref/节点Refs |
| `backup.prepare` / `operations.backup.v1` | `{backup:BackupJob}` | 实例管理员且维护边界；共享跨工作区导出不能借普通成员token；输出job Ref/缺项；不是公共任意文件读 |
| `reconciliation.record` / `operations.reconcile.v1` | `{case:Ref,evidence_refs:string[],observed:accepted\|rejected\|not_sent\|unknown,note:string}` | 原主体/运维核对者并逐资源授权；只追加观察，决不派发；输出case Ref；手写结果标operator_assertion不充当远端证明 |

接口虽以 registry 传输，权限不由字符串决定：实现必须 exhaustive 匹配 operation_name→DTO→core handler→capability→effect。未知操作一律拒绝；插件不能注册同名 handler 覆盖核心动作。CLI/MCP/SDK 使用同一 registry，不另开可绕过审批的快捷路径。最终公开 OpenAPI 应把 operation_name/params 编成 oneOf/discriminator。

<a id="E06"></a>
## E06 日历、配置与检索查询 DTO

```typescript
type ResolveInput = { layers: Ref[]; overrides: FieldOverride[]; for_operation: string };
type ResolvedConfig = { fields: {field:string,effective_value:unknown,source:Ref|null}[];
  digest:string; effective_grant:Grant; effective_bounds:Bound; warnings:string[] };
type ScheduleDraft = { title:string; conversation:Ref; profile:Ref;
  content:Content[]; kind:"once"|"interval"|"cron"; at?:string;
  interval_ms?:number; cron?:string; timezone:string; viewer_timezone:string;
  missed_policy:"skip"; targets:Ref[]; bounds:Bound };
type SchedulePreview = { schedule_digest:string; tzdb_version:string;
  paused:boolean; occurrences:{utc:string,schedule_local:string,viewer_local:string,
  occurrence_key:string}[]; warnings:string[] };
type RetrievalInput = { library_refs:Ref[]; query:string; top_k:number;
  mode:"lexical"|"hybrid"|"rerank"; profile:Ref|null; bounds:Bound|null };
type RetrievalResult = { items:{source:ReadRef,source_version:string,
  original_asset_ref:AssetRef,normalized_text_asset_ref:AssetRef,
  parser_id:string,parser_revision:string,chunk_id:string,title:string,excerpt:string,
  location:{start_char:number,end_char:number,page:number|null},
  index_generation:Ref,score:number,score_kind:string}[];
  mode_used:string; degraded:boolean; reasons:string[]; operation_id:string|null;
  usage:{cost_microusd:number|null,coverage:"known"|"partial"|"unknown"} };
```

日历一次仅有 at；间隔仅有 interval_ms 且锚点由服务器保存；cron 仅有六字段 cron（秒、分、时、日、月、星期），IANA zone 必须来自本机 tzdb，任意自由字符串通过验证才接收。预览至多返回未来 10 次/实际配置限制，越界返回明确错误；暂停计划亦可预览但不能制造运行。未来 v2 缺失当地时刻 skip，重复当地时刻采用 earliest instant，预览输出该策略提示；不得静默改变当前 v1 的实际时间语义，升级前用两版 fixtures 比对并明示差异。

检索 lexical 无模型/重排费用且不需 Bound；hybrid/rerank 只有 profile 真实可用且 Bound 明确才执行。未知 mode 不回退为成功；明确可降级的重排失败只返回已授权候选并标 degraded。费用请求保留 operation_id 便于取消/核对，不得由无幂等只读重试制造多次收费：带费用的检索还须 `Idempotency-Key`，其作用域/输入摘要/操作查阅遵守 E03。

RetrievalResult只返回ready且引用完整的来源，结果中的两个AssetRef保持非null；每个RetrievalResult.source.kind固定`knowledge_sources`，scope=workspace。字符位置相对normalized_text_asset_ref对应的Unicode scalar序列，页码仅解析器有真实页边界时提供；原始文件通过original_asset_ref读取。来源replace后旧citation显示其原版本与失效/过时状态，读取仍按当前ACL；source删除/撤权同时使原文、规范化文本和所有派生下载拒绝，不能通过保留asset绕过删除。KnowledgeJob逐项结果必须返回source ReadRef，即使URL导入也可刷新后读其真实版本、解析器和错误。

<a id="E07"></a>
## E07 与 worker broker、迁移和验收的连接

插件 `1.1-draft` 的可选 `host.services.v1` 方法由 protocol 包独占定义；核心在当前 root invocation 内将其转到上述同一领域服务。root request ID/opaque handle 由核心绑定，worker 不得指定本地 owner 或扩展 grant。`host.task.submit` 返回持久 child 引用而不是假完成；`host.delivery.prepare` 仅准备待批准意图，不能发送。root 结束、generation 改变、撤权或超时使 handles 失效；原协议 1.0 仍拒绝反向请求。

未来 v2 数据仅在 storage 的显式 schema 3 迁移成功后启用，迁移不能改变 v1 原对象/状态值；个人凭据与会话分别验证。没有扩展插件时仍能运行 v1 已有能力；任何扩展缺前提只将对应 capability unavailable，不关闭整个核心。v2/插件版本选择、权限和可用状态都进脱敏验收记录。

接口完成门槛为 `R13-CORE-003/004/007/009/015/022/027/029/032/034/036/039/041/042` 的业务下游证据，加 `R13-CORE-040` 的生成 schema/路由/DTO exhaustive 检查。全部九节都参加检查。这些场景本次均 not_executed；文件存在、类型能生成或 schema 可解析均不等于端到端已支持。

<a id="E08"></a>
## E08 插件安装和 MCP 管理的完整入口

本节承接`plugins`依赖的`LIFECYCLE_AND_ISOLATION.md` H-01/H-03及`ADAPTERS_AND_COMPATIBILITY.md` A-02，HTTP/DTO仅在本文件定义。来源获取和依赖准备不是配方依赖；不能把普通Git源码库塞进八包依赖图。安装/构建/权限批准/激活分开，打开列表或读取元数据不自动执行代码。

Admin只读kind为`package_install_jobs, plugin_packages`；两者Record.workspace_id=null、owner为实际实例操作者。工作区只读kind`plugin_instances, plugin_data_exports, mcp_connections, mcp_catalogs`遵守正常workspace/owner/ACL。以下字段严格，不允许未知属性；source/URI都经实例批准网络政策，URL不能携带明文凭据，来源认证仅用绑定目标的credential reference。

| DTO | 输入字段 | 输出/状态 |
| --- | --- | --- |
| `PackageSource` | `source_kind:local_manifest\|archive\|git_commit\|index_entry, canonical_source:string, pinned_revision:string, expected_sha256:string, runtime_kind:rust-native\|python\|node\|dotnet\|other-approved, credential_ref:Ref\|null` | local_manifest仅能引用管理员预登记的本机导入ID，不给HTTP任意路径；git_commit为完整冻结commit；index_entry解析后仍固定原来源/版本/摘要 |
| `PackageInstallJob` | `source:PackageSource`；阶段动作另列 | `package_id, source_digest, license_record:{source,version,license_path,sha256,status:known\|unknown,distribution_constraints:string[]}, permission_proposal, permission_digest, schema_revision, protocol_versions, previous_package_ref\|null, state:discovered\|downloaded\|verified\|prepared\|awaiting_permissions\|staged\|active\|failed\|cancelled, phase_error, progress, recovery_point_ref\|null` |
| `PluginPackageRead` | 只读 | `identity:{canonical_source,package_id},active_version,sha256,runtime_kind,license_record,approved_features,permission_revision,protocol_versions,instance_refs_redacted,rollback:reversible\|restore_required\|unavailable`；不含各workspace私有config |
| `PluginInstanceInput` | `package_ref:Ref, config_group_id:string, config_schema_revision:string, config:object, visibility, requested_features:string[]` | config严格匹配该包公开schema且secret仅write-only引用；保存默认disabled |
| `PluginInstanceRead` | 只读 | `package_ref,config_group_id,config_schema_revision,config_redacted,permission_proposal,permission_digest,approved_features,permission_revision,generation,status,enabled,data_schema,data_strategy`；提案/摘要由核心按当前包/实例/配置计算，供显式决定；v1已存实例同一身份只读适配，不复制数据 |
| `PluginDataExportRead` / plugin_data_exports | 只读；由export_data/confirm_delivery驱动 | `instance_ref,generation,data_schema,scope:instance_shared\|principal_private,principal:Subject\|null,state:preparing\|ready\|confirmed\|failed\|cancelled,asset_ref:AssetRef\|null,manifest_sha256,size_bytes,confirmed_by:Subject\|null,confirmed_at:string\|null`；仅调用者当前获准数据，秘密以加密引用/排除清单处理，不导出原key/会话token |
| `MCPConnectionInput` | `name, adapter_instance:Ref, transport:stdio\|streamable_http, profile:"2025-11-25", endpoint:string\|null, approved_command_id:string\|null, credential_binding:Ref\|null, requested_capabilities:(tools\|resources\|prompts)[], limits:{connect_timeout_ms,initialize_timeout_ms,request_timeout_ms,max_reconnect_attempts,max_backoff_ms,total_deadline_ms,max_result_bytes}, allow_private_network:boolean` | stdio仅approved_command_id且endpoint=null；网络仅endpoint且command=null；保存disabled；较新/未知profile返回unsupported_mapping，不称latest |
| `MCPConnectionRead` | 只读 | `input_redacted,generation,state:disabled\|connecting\|initializing\|ready\|degraded\|reconnecting\|failed\|revoked,negotiated_profile\|null,supported/authorized/available,catalog_ref\|null,last_error,retry_after\|null` |
| `MCPCatalogRead` | 只读 | `connection_ref,generation,revision,items:[{kind:tool\|resource\|prompt,name,namespace,server_identity,schema_digest,schema,annotations,approved,available,reason}],next_cursor`；每页≤100项/实际更低限制；catalog变化不自动批准新工具 |

`Record.workspace_id`的类型允许null仅针对上列admin对象/BackupJob；workspace对象始终非null。`permission_proposal`是包声明的有限feature/scope/网络/文件能力列表和数据迁移信息，不是授权；`permission_digest`覆盖其规范化全文、包摘要和目标代次。

| operation_name / scope / capability | params | 真实动作、权限与输出 |
| --- | --- | --- |
| `package.discover` / admin / `plugins.install.v1` | `{source:PackageSource}` | 实例管理员只取得固定来源元数据/清单/许可证与声明；禁止安装脚本；输出package_install_jobs Ref(discovered) |
| `package.download` / admin / 同上 | `{job:Ref}` | 明确获取冻结包/验证摘要；离线仅用匹配缓存；输出verified或failed，旧active不改 |
| `package.prepare` / admin / 同上 | `{job:Ref,execute_prepare:true,dependency_lock_digest:string}` | 管理员明确批准准备阶段代码/固定依赖执行；仅verified且许可/网络边界已披露；独立准备区构建/握手/迁移预检，输出prepared/awaiting_permissions；失败不激活 |
| `package.permissions.decide` / admin / 同上 | `{job:Ref,permission_digest:string,decision:approve\|reject,approved_features:string[]}` | 实例管理员对这一摘要/版本批准权限上限；reject不影响旧active；不能授予workspace业务权限；成功到staged |
| `package.activate` / admin / 同上 | `{job:Ref,expected_active:Ref\|null,recovery_point_ref:Ref\|null}` | 仅staged；原子活动代切换/回收旧进程；不可逆数据迁移须已验证恢复点，否则拒绝；输出plugin_packages Ref |
| `package.uninstall` / admin / 同上 | `{package:Ref}` | 实例管理员；仍有实例/必须数据引用则409并列脱敏阻塞scope，先经各workspace合法清理；不擅自删除私有数据/历史回执；输出tombstone |
| `plugin_instance.save` / workspace / `plugins.lifecycle.v1` | `{instance:Ref\|null,data:PluginInstanceInput}` | workspace admin只选已批准包；CAS/schema验证，扩权先禁用；输出plugin_instances Ref(disabled/configured)，不下载代码 |
| `plugin_instance.permissions.decide` / workspace / 同上 | `{instance:Ref,permission_digest:string,approved_features:string[],decision:approve\|reject}` | workspace admin且资源所有者/观察私聊授权另验；权限是包上限∩实例决定∩当前业务grant；输出Ref/permission_revision |
| `plugin_instance.set_enabled` / workspace / 同上 | `{instance:Ref,enabled:boolean}` | 当前实例管理者，所有版本/权限有效；禁用撤销租约/注册；启用不自发业务调用；输出Ref/generation |
| `plugin_instance.export_data` / workspace / 同上 | `{instance:Ref,scope:instance_shared\|principal_private,principal:Subject\|null}` | 仅有数据导出权者；private主体必须是本人或明确授权的服务，不给admin隐式私聊权；生成受ACL保护asset和清单，输出plugin_data_exports Ref，创建/取消均不删原数据 |
| `plugin_export.confirm_delivery` / workspace / 同上 | `{export:Ref,sha256:string,received_bytes:number}` | 原导出主体在实际下载读回并校验摘要/字节后明确确认；CAS核对ready且代次/数据schema/范围未变，输出confirmed Ref；仅点击下载或请求启动不算交付 |
| `plugin_instance.uninstall` / workspace / 同上 | `{instance:Ref,data_strategy:keep_data\|export_then_remove\|remove_data,confirmed_export_ref:Ref\|null}` | 当前合法管理者且不擅自读别人private数据；先停用/撤销/注销，再按策略；export_then_remove须可核对已交付摘要确认，否则保留数据；known/unknown回执不删 |
| `mcp_connection.save` / workspace / `mcp.client.v1` | `{connection:Ref\|null,data:MCPConnectionInput}` | workspace admin且stdio命令由实例预先批准/网络目标合法；保存disabled/CAS，输出mcp_connections Ref |
| `mcp_connection.set_enabled` / workspace / 同上 | `{connection:Ref,enabled:boolean}` | 获准管理者；连接初始化/目录只读前提就绪，输出真实state；失败保留配置可修复；禁用撤销调用和handles |
| `mcp_connection.refresh_catalog` / workspace / 同上 | `{connection:Ref}` | 当前管理者和连接generation核对；只读列目录但仍应用请求费用政策/上限；输出mcp_catalogs Ref；不执行tools/call、不自动授予新能力 |

MCP业务工具调用仍通过`tool.propose/tool.execute`，资源读取经过同一当前grant，新增目录/annotations不授予写权。stdio命令预登记使用实例已有可信包manifest/受限CLI，approved_command_id解析为已安装包身份+冻结的manifest入口ID；HTTP不能传任意shell、args或环境变量取得执行权。OAuth使用E02绑定生命周期，同连接固定目标/受众/scope；远端sampling/elicitation/UI扩展当前unsupported，不伪造空成功。

插件数据导出通过既有授权asset content入口实际下载；网络中断/未确认、摘要或字节不符、instance generation/data schema/范围改变，均不能满足export_then_remove。confirmed_export_ref必须指向当前主体有权且涵盖本次删除范围的confirmed导出；范围不足返回409并保留数据，用户可选keep_data或先取得数据所有者明确授权完成新导出。导出/确认/卸载各有独立幂等意图与版本，不能靠同包名字认领另一来源的数据；必要的accepted/unknown回执和防重放tombstone不在普通数据删除范围内。

core的`delivery.prepare`/`delivery.multitarget.v1`与worker的`host.delivery.prepare`/`delivery.prepare.v1`是不同边界：broker还必须协商`host.services.v1`及`delivery.prepare.v1`且持本次单目标route handle；多目标API存在不能自动给所有插件开放broker。worker只能准备其获准目标意图，审批/dispatch仍由core操作完成。

新增验收 `R13-CORE-041/042` 联合 `R13-HOST-001/002/003/006/007/008/016/017/018`，检查完整管理动作的真实下游状态而非只看schema。

<a id="E09"></a>
## E09 编排节点、映射、条件与汇合

这是内建编排的完整有限表达能力；外部插件schema仅决定其已授权工具的arguments，不负责发明branch/join。节点上限64、边上限128、映射每边32、条件深度8/叶节点64、数据引用路径深度16，可由能力limits降低；超限在保存时拒绝。这里只规定生成默认上限，不报告性能已达标。

```typescript
type Scalar = null | boolean | number | string; // number为有限安全整数
type DataRef = { from: "workflow_input" | "node_output"; node_id?: string;
  path: string[] }; // 每项为字面对象key或十进制数组index，无通配符/eval
type ValueExpr = { type:"constant"; value:Scalar } | { type:"ref"; ref:DataRef };
type Condition = { op:"eq"|"ne"|"lt"|"lte"|"gt"|"gte"; left:ValueExpr; right:ValueExpr } |
  { op:"exists"; ref:DataRef } | { op:"all"|"any"; conditions:Condition[] } |
  { op:"not"; condition:Condition };
type FieldMapping = { source:DataRef; target_path:string[]; required:boolean;
  missing:"error"|"omit" }; // required=true必须missing=error
type Edge = { from:string; to:string; when:"always"|"true"|"false"; mapping:FieldMapping[] };
type NodeInput =
  { kind:"read"; source:ReadRef; fields:string[][] } |
  { kind:"model"; profile:Ref; content:Content[]; bounds:Bound } |
  { kind:"approved_tool"; intent:Ref; approval:Ref } |
  { kind:"branch"; condition:Condition } |
  { kind:"join"; policy:"all_success"|"collect"; required_upstreams:string[]; allow_skipped:boolean } |
  { kind:"subagent"; delegation:DelegatedRun } |
  { kind:"deliver"; intents:Ref[]; approvals:Ref[] };
```

Node.type必须与input.kind一致。DataRef只读当前工作流输入或当前图的已声明上游输出；不得通过路径寻址配置secret、其他run、原始宿主文件或未授权对象。字段映射先做目标类型/schema验证；两个来源写同一目标字段或父子路径冲突在保存时拒绝，不用“最后写入获胜”。缺required字段使该节点failed/input_missing，不发模型/工具；omit仅允许目标字段可选。节点输出保存为带ACL的有界JSON/asset引用，并带数据来源和revision。

branch只做纯本地比较，输出`{branch:true|false}`；eq/ne要求同类型严格比较，lt/lte/gt/gte只允许安全整数或同类型字符串的Unicode码点字典序，类型不匹配为failed/condition_type_error。缺引用仅exists得到false，其他比较为input_missing；all/any的条件数组必须非空，按声明顺序短路但不产生外部动作，not恰一个条件。不存在代码字符串、正则任意执行、网络函数或当前时间隐式读取。true/false边只能来自branch；未选边及仅由其可达的节点标skipped/branch_not_taken。

join的required_upstreams必须是实际入边集合，不得列其他图节点。等待这些上游全部终态（succeeded/failed/cancelled/skipped/unknown）再结算；不以第一个到达触发。all_success：任一unknown→blocked/reconciliation_required，任一failed/cancelled→failed/dependency_failed，存在skipped且allow_skipped=false→skipped/dependency_skipped，否则汇合成功。collect：输出每个上游的状态与可读output_ref，不将failed/unknown伪成成功；unknown仍阻止后继带外部副作用节点，只有显式新核对/新执行意图才能放行。join本身没有网络/模型/投递调用，配置collect不能绕过未知副作用门槛。

workflow执行只有active定义可准入，冻结definition revision/输入映射/授权与预算；validate可验证draft/validated，simulate和controlled_read不能执行生产write/deliver。每节点有独立operation_id/attempt与预算子引用；恢复遇unknown写节点不自动重跑或继续其依赖节点，父取消传播但保留已知结果。验收`R13-CORE-034/040`需覆盖两支分支、缺字段/类型错、重复目标映射、不同到达顺序的join、skipped/unknown与取消/重启。
