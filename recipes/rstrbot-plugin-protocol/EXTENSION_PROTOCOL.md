# 扩展插件协议合同：1.1-draft

本文件定义下一轮重建的线协议，状态为 `specified_not_implemented`；验收均为 `not_executed`。包版本 `0.2.0-recipe.2` 是材料版本，不是协议实现版本。当前可观察行为仍由 [PROTOCOL_REFERENCE.md](PROTOCOL_REFERENCE.md) 的 **1.0** 和 [CONFORMANCE_VECTORS.md](CONFORMANCE_VECTORS.md) 定义，二者保持原文。这里没有将草案加入当前程序的能力发现。

本质是“在一条已认证连接上，传有类型的请求和回执”。身份、权限、预算、永久事件和操作意图由调用方核心保管；插件回答能力和结果，不能自己扩大权限。此文件是插件线格式、扩展能力名和共同向量的唯一规范；HTTP 不在这里重复定义。

<a id="px-01"></a>
## PX-01 版本、初始化和能力交集

1.0 的精确版本握手、单根调用、已有方法和拒绝反向管理 RPC 完全保留。旧配置继续选择 `1.0`。只有安装清单、用户启用配置和核心实现均显式允许时，才启动 `1.1-draft`；未就绪返回 `capability_unavailable`，不得为“兼容”把一次可能已发出的调用改版重发。

新进程初始化仍为 `initialize`，发送 `protocol_version`、`nonce`、`context`；1.1-draft 增加 `features` 字符串数组和 `limits`。响应必须回显版本及完整上下文，并返回自身 `features`。不匹配、重复初始化、初始化前业务帧及未知必需特性均拒绝。有效特性计算如下；不能把未支持与未授权混成“没有插件”。

```text
effective_features = installed_manifest ∩ worker_handshake ∩ core_implemented ∩ enabled_features
authorized_operation = effective_feature && enabled_instance && current_authority
                       && resource_scope && budget_or_effect_policy
```

| 精确能力 ID | 必需方法/用途 | 默认授权 |
| --- | --- | --- |
| `hooks.v1` | `hooks.invoke`，有序生命周期钩子 | 无；逐事件和读取范围授权 |
| `commands.v1` | `commands.list`、`commands.execute` | 无；发现和执行分别核权 |
| `plugin_data.v1` | 通过受控 host 服务读写实例数据 | 无；实例和用户范围分别批准 |
| `platform.receive.v1` | `platform.poll`，有界入站批次 | 无；连接账号及观察权限单独启用 |
| `media.refs.v1` | 结构化 `asset_ref`，不传宿主路径 | 无；每次消费重验 ACL |
| `credential_proxy.v1` | `host.credential.request` | 无；只允许固定受众和操作 |
| `mcp.client.v1` | `mcp.invoke`，经 MCP 适配器调用 | 无；远端能力声明不是授权 |
| `sandbox.exec.v1` | `sandbox.execute` | 无；隔离后端实测可用才允许 |
| `tasks.delegate.v1` | `host.task.submit`，核心建立子任务 | 无；权限与预算取父运行子集 |
| `delivery.prepare.v1` | `host.delivery.prepare`，只保存绑定路由的待批准意图 | 无；不授予批准权或实际发送权 |
| `host.services.v1` | 下节限定的 worker→host 子请求 | 无；需另外具备每个服务的具体能力 |

新特性不扩充 1.0 的能力域枚举；它们是 1.1-draft 的版本化特性。新实现必须同时能启动 1.0 worker，且不得向其发送草案字段。取消/网络失败不是版本协商失败。若未来将草案改为稳定版，必须另存稳定规范、升级说明和同版向量，不直接把字符串改成“1.1 已支持”。

<a id="px-02"></a>
## PX-02 类型、上下文和资源归属

JSON-RPC 2.0 外壳沿用 1.0。根请求 ID 为 `h:<唯一串>`；子请求 ID 为 `w:<唯一串>`，字符串逐字比较，不按数字转换。一个连接只容纳一个根业务调用；根 ID 不能被子请求或迟到通知复用。编码 UTF-8，一行一对象，帧≤1 MiB，参数≤512 KiB；这些为继承的安全上限，不是吞吐达标声明。

`context` 保留 `workspace_id:string`、`instance_id:string`、`generation:integer`；1.1-draft 另含 `invocation_id:string`、`operation_id:string`、`run_id:string|null`、`principal_kind:user|service|system`、`principal_id:string`、`authority_version:string`、`deadline_at:RFC3339 UTC`、`purpose:interactive|scheduled|delegated|administrative|test`。全部由核心注入和验证，业务参数中的同名字段不能覆盖。`generation` 为 0..9007199254740991 的整数，拒绝小数、NaN 和溢出；存储内部若更大必须在出线前拒绝或另行版本化，不能四 SDK 各自截断。

`GrantHandle` 是核心生成的不可猜测字符串；服务器绑定 workspace、principal、instance、generation、root invocation、resource version、允许动作、受众和到期时间。柄本身不是永久密钥，不能跨调用/重启/工作区复用。有效期还受原会话、当前成员、配置版本和父任务取消约束。系统或服务主体必须有独立显式授予，不能借一个仍在线的同名用户续命。

`asset_ref = {asset_id:string, version:string, sha256:64位小写十六进制, media_type:string, size_bytes:nonnegative safe integer}`。`version`是与HTTP扩展AssetRef完全相同的不透明内容版本字符串，逐字往返；它不同于资源记录CAS用的数值`revision`，禁止parseInt/Number或字符串化数值revision来冒充内容版本。引用仅标识内容；附加 `read_handle` 才表示本次可读的最小字节范围。模型输入的来源清单记录实际消费的 asset_id、版本、字节范围和转换版本。只返回本地文件名不算消费成功。删除/撤权后不得继续从缓存或旧柄读取。

`Usage = {input_tokens:integer|null, output_tokens:integer|null, other_units:object, provider_charge:string|null, currency:string|null, source:provider|estimated|unknown}`。金额使用十进制字符串，不以二进制浮点金额结算；未知不是零。实际账本结算及币种换算由核心唯一合同负责，SDK只原样传十进制金额/currency，不做浮点估价或自行折成美元；无法可靠换算时保持未知而非零。供应商 `opaque_state` 作为有大小限制的原始 JSON 值保留，不跨不兼容提供方冒充签名。密钥、请求头和私有响应正文不能放入进度、错误和诊断。

<a id="px-03"></a>
## PX-03 根调用、受控 host 服务与终态

没有 `host.services.v1` 时继续拒绝全部 worker 发起请求。启用后只接受下表白名单；这属于草案扩展，不回写成 1.0 已有能力。每个子请求带 `root_request_id` 与 `grant_handle`；每连接最多一个挂起子请求，不能在处理子请求期间递归调回同一 worker。子请求不得保持数据库事务等待网络。根退出、超时、断连或 generation 变更即撤销全部子请求与柄。

所有表中方法都同时要求 `host.services.v1` **和**“具体feature”列；每项必须在manifest、握手、核心实现与启用配置的交集中。缺任一feature在访问数据/写意图/外部请求之前返回`-32004`，即使grant_handle看似有效也不能调用。feature只代表已实现且可申请的表面，grant_handle还须逐动作/资源/参数/预算授权，不能用get授权执行cas。下表是完整白名单，没有前缀通配授权。

| 服务方法 | 具体feature | 输入（另加公共root/handle） | 效果与唯一允许的value类型 | 核心必须执行的限制 |
| --- | --- | --- | --- | --- |
| `host.resource.read` | `media.refs.v1` | asset_ref、offset、length | read_only；ResourceChunk | 单次返回解码字节≤64 KiB；读取前后ACL/版本一致；原路径不暴露 |
| `host.data.get` | `plugin_data.v1` | namespace、key | read_only；DataRead | namespace由柄绑定；不能传任意工作区/用户代读 |
| `host.data.cas` | `plugin_data.v1` | namespace、key、expected_version、value、mutation_id | idempotent_write；DataWrite | 仅expected_version=null表示创建；value可合法为JSON null；同mutation_id异内容冲突 |
| `host.data.delete` | `plugin_data.v1` | namespace、key、expected_version、mutation_id | idempotent_write；DataDelete | CAS、墓碑和审计；重启不复活或变新写 |
| `host.credential.request` | `credential_proxy.v1` | credential_handle、request_template_id、arguments、operation_id | 模板固定的效果类别；CredentialReply | 固定受众/模板/响应schema；模板若写入须相应审批，不接受插件自报read_only |
| `host.task.submit` | `tasks.delegate.v1` | child_spec、parent_run_id、operation_id | idempotent_write；ChildReceipt | 核心创建持久子任务；后续执行另受父授权/预算；不授予独立后台身份 |
| `host.delivery.prepare` | `delivery.prepare.v1` | authorized_route_handle、content、operation_id | idempotent_write；DeliveryIntentReceipt | 仅准备意图；该调用不得发送、批准或返回下游accepted |

所有子请求成功也使用下文统一`{value,usage,effects,cost_scope}`封套；value只能是对应类型，额外管理字段一律拒绝。以下为线类型，`AssetRef`即PX-02的asset_ref类型，`Json`仅指有大小限制的合法JSON值，`safe_integer`沿用PX-02边界：

```typescript
type ResourceChunk = { asset_ref: AssetRef; offset: safe_integer;
  bytes_base64: string; read_bytes: safe_integer; chunk_sha256: string; eof: boolean };
type DataRead = { found: boolean; value: Json | null; version: string | null };
type DataWrite = { key: string; version: string; replayed: boolean };
type DataDelete = { key: string; tombstone_version: string; replayed: boolean };
type CredentialReply = { response_kind: "json" | "text" | "asset" | "receipt";
  output: Json | string | AssetRef | null; receipt_ref: string | null };
type ChildReceipt = { child_run_id: string;
  state: "queued" | "running" | "completed" | "failed" | "cancelled" | "interrupted" | "unknown";
  replayed: boolean };
type DeliveryIntentReceipt = { intent_id: string;
  approval_state: "proposed" | "approved" | "rejected" | "expired" | "revoked" | "consumed";
  replayed: boolean };
```

ResourceChunk.chunk_sha256只校验本次解码块，asset_ref.sha256校验完整资产；偏移/长度不得溢出。DataRead.found=false必须value/version均null；found=true允许存储值本身为null，version仍必填。CredentialReply.response_kind和output必须匹配已批准模板响应schema，不能夹带认证头、原始秘密、任意管理对象；模板效果精确为read_only/idempotent_write/non_replayable_write之一，未分类即拒绝。新建子任务只确认queued持久事实，重复请求可返回该既有任务当前state；不得伪造已完成。新建投递意图的approval_state为proposed；重复prepare只读现有状态，不代签批准，也不以该封套暗示已投递。其effects必须dispatch=not_sent、delivery=not_applicable，已有投递结果通过独立获准receipt查询获取。

namespace 为 `instance` 或 `principal`；实例域固定 `(workspace_id,instance_id)`，个人域再加 principal_id；键为 1..128 UTF-8 字节不含 `/`、`\`、NUL 的逻辑名，value≤64 KiB JSON。总配额由实例配置声明并在数据写入前核对，默认未配置写额度即不开放写能力。持久数据是核心数据根的一部分，不是进程临时目录。

根成功仍只有一个 JSON-RPC `result`；根失败只有一个 `error`。新方法的 result 是 `{value,usage,effects,cost_scope}`；`cost_scope` 为 `not_billable|estimated|provider_reported|unknown`。仅已知纯本地无计费操作可用 `usage:null,cost_scope:not_billable`；未知远端费用必须 `cost_scope:unknown`。`effects` 每项含 `operation_id`、`dispatch:not_sent|sent|unknown`、`delivery:accepted|rejected|unknown|not_applicable`、`persistence:saved|failed`、`receipt_ref:null|string`、`reconciliation_required:boolean`。这只是证据结构，核心不得把 worker 自报的 saved 当作已落库。

根错误的 `error.data` 必须保留已知 effects 和安全的 `category`、`phase`、`retryable_before_dispatch`。`-32800` 为取消；权限拒绝/撤销为 `-32003`，不支持的能力为 `-32004`，版本/CAS 冲突为 `-32005`，预算/限额为 `-32006`，需要人工核对的未知副作用为 `-32007`；原 `-32601/-32602/-32603/-32001` 保留。协议错误与存储、认证、供应商、限流、外发故障分别标 `domain`，不能把存储冲突包装为供应商故障触发换模型计费。

完成、取消和回执存在竞态：若取消前已知下游接受，保存该事实，同时停止后续动作；若不知道则 unknown。不能将“进程杀掉”写成“远端没发生”。重复相同 operation_id 先查持久意图；同 ID 不同参数冲突；unknown/pending 只读核对，不自动重发；用户显式建立新意图也必须看到重复风险和旧意图关联。

<a id="px-04"></a>
## PX-04 扩展业务方法

下列 DTO 为草案新增方法的共同形状。现有 `model.* / tools.* / platform.send` 在 1.0 中不改名也不改参数；新媒体/审批能力必须在 1.1-draft 下有明确实现后才暴露。

| 方法 | 输入 | 输出及副作用边界 |
| --- | --- | --- |
| `hooks.invoke` | event_id、hook_phase、subject_version、payload、allowed_patch_fields、deadline | decision=continue/stop/defer、patch、diagnostic_code；patch越界拒绝，不授予发送权 |
| `commands.list` | catalog_version、locale | namespace化命令、aliases、parent、参数schema、所需scope；每项有enabled/reason |
| `commands.execute` | command_id、typed_arguments、event_id、operation_id | 结构结果与effects；父命令过滤和每次授权先于调用 |
| `platform.poll` | connector_id、generation、cursor:null\|string、limit:1..100、wait_ms:0..30000 | events、next_cursor、has_more；不含核心本地owner/route；持久化前不可确认消费 |
| `mcp.invoke` | connection_id、connection_generation、profile、method、arguments、operation_id | 有界文本/结构结果/asset_ref与真实回执；不是任意URL访问器 |
| `sandbox.execute` | backend_id、isolation_profile、program_ref、arguments、read_handles、output_quota、network_policy_id、operation_id | exit_code、bounded stdout/stderr、published asset_refs、effects；隔离准备失败不执行 |

`hook_phase` 精确为 `inbound.observed / trigger.before / context.before / summary.before / model.before / model.after / tool.before / tool.after / outbound.before / delivery.after / run.cleanup`。授权后才发布 `inbound.observed`，默认不让插件观察所有私聊。持久事件交付可能重复，核心持有 `(event_id,instance_id,generation,hook_phase)` 的执行记录。只有声明纯函数且确认没有副作用的钩子可重算；已派发未知钩子须暂停核对。优先级顺序固定为 `(configured_order,instance_id)`；输入快照/输出补丁均记录版本；插件不能修改授权身份、原始来源、预算和 operation_id。若任何获准补丁改变工具参数、目标或效果，核心必须在最终参数确定后再审批；已有审批与旧摘要不匹配即失效，不能边执行边补批准。

`platform.poll.events[]` 项为 `{event_id,connector_id,generation,platform_account_id,external_conversation_id,source_entry,sender_external_id,occurred_at,received_at,content,reply_to_external_id?,self_echo}`。两个时间为 RFC3339，`source_entry` 为平台提供的来源标签，content 项严格选 `text` 或 `asset_ref`。所有外部 ID 是数据而不是权限证明。核心自己建立稳定会话 ID、外部来源、当前查看入口和批准的出站目的地四种独立字段；Web 打开外部会话不改其来源路由，字符串包含“discord”也不授予 Discord 发送权。去重键包括 connector 身份和外部 event_id；重连重发同事件不新增规划；静默消费只推进消费事实，不能标已回复。

长期适配器连接可以由宿主监督的连接管理器持有；业务调用仍经上述有界接口。连接管理器只有被批准的连接账号和订阅能力，没有核心数据库/全局管理入口；进程状态丢失以持久游标重新连接，不能让连接重建重放写工具。

<a id="px-05"></a>
## PX-05 声明式扩展向量与验收输出

每个向量对 Rust/Python/TypeScript/C# 使用相同 UTF-8 输入。匹配语义 JSON，不比较对象键顺序；opaque_state 内的数组顺序、字符串、null、缺字段必须逐项保留。必须补充原始帧摘要、worker 退出/清理证据和下游计数，不能仅比 SDK 返回值。

| 向量 | 输入变化/故障 | 确定结果 | 验收 ID |
| --- | --- | --- | --- |
| PX-V01 | 1.0 worker 收到1.0原向量；1.1-draft worker新增features | 两套分开通过，1.0无草案字段 | R13-PROTO-001 |
| PX-V02 | nonce/context/authority_version/generation逐一改动 | 握手拒绝；下游调用0 | R13-PROTO-002 |
| PX-V03 | 只有manifest声称`host.services.v1`或只有worker声称；再逐方法移除其具体feature并提交错误value类型 | 缺feature时0业务请求；回包类型越界为protocol_error；prepare不发送 | R13-PROTO-003 |
| PX-V04 | 两个并行w:请求、错root ID、根结束后晚来w: | protocol_error；根最多一个终态；柄失效 | R13-PROTO-004 |
| PX-V05 | 同asset_ref换workspace、删源后读、越界range | 拒绝/失效；零越界字节；未给宿主路径 | R13-PROTO-005 |
| PX-V06 | data.cas同version并发两次；重启后重复mutation_id | 一个提交一个冲突；重复读原结果 | R13-PROTO-006 |
| PX-V07 | platform.send目标接受后断连并cancel | effect=unknown或保留已知accepted；不重放 | R13-PROTO-007 |
| PX-V08 | safe整数边界/越界；未知usage；稀疏工具片段/opaque签名 | 边界明确、未知非零、原字段不丢失 | R13-PROTO-008 |
| PX-V09 | 撤权/父取消在子请求等待和结果返回处发生 | 不派发下一步；已知回执留存；无替代会话 | R13-PROTO-009 |
| PX-V10 | 上游poll重复event、旧generation、Web查看后再poll | 持久消费一次；稳定来源不改写 | R13-PROTO-010 |
| PX-V11 | inbound观察关闭→显式开启→撤销，回放同一event | 关闭时0内容广播；开启仅所授范围；撤销后停止 | R13-PROTO-011 |
| PX-V12 | error.domain=storage且category=conflict | 不触发provider重试/回退；错误归因保持 | R13-PROTO-012 |

全部新增向量当前 `not_executed`。关闭旧上游问题不使这些向量通过；1.0的历史20项四SDK证据不自动继承到扩展。

## 规范来源与选择说明

JSON-RPC 外壳参照 [JSON-RPC 2.0](https://www.jsonrpc.org/specification)（核对2026-10-28）；本项目对批次、帧长、ID及受控服务的限制是自己的合同。MCP 外部协议不等于本文件的插件RPC，双方由适配器显式转换；版本策略见宿主 `ADAPTERS_AND_COMPATIBILITY.md`。没有把文档可读或协议设计完成计作运行证据。
