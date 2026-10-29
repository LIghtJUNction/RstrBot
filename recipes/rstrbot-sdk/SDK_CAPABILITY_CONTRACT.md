# 四语言SDK的完整能力合同

范围：RB-007/016/019/023/030/033/037/047。此增补状态为 `specified_not_implemented`，新增验收 `not_executed`。原四SDK基础协议的历史证据保留在来源材料，不继承为本轮或扩展已通过。唯一线格式来源是声明依赖 `protocol` 的 `PROTOCOL_REFERENCE.md`、`CONFORMANCE_VECTORS.md` 与 `EXTENSION_PROTOCOL.md`；本包不另造一套RPC或复制核心OpenAPI。

<a id="sdk-surface"></a>
## SDK-01 公共类型与等价入口

四种语言可以用自然的异步方式实现，但必须暴露下表的同义能力。`Plugin`的handler按精确method注册；未知方法返回标准错误，不能交给catch-all隐式执行。上下文由初始化生成只读对象，业务参数不能覆盖。默认只选1.0；1.1-draft必须明确启用已实现features，没有某feature不安装其handler。

| 公共语义 | Rust | Python | TypeScript | C# |
| --- | --- | --- | --- | --- |
| 启动一个受控调用 | async serve(Plugin) | await serve(Plugin) | await serve(Plugin) | PluginServer.ServeAsync(Plugin) |
| 不可变调用上下文 | InvocationContext | frozen InvocationContext | Readonly InvocationContext | immutable InvocationContext |
| 取消 | cancellation token | cancellation event/CancelledError | AbortSignal | CancellationToken |
| 有界进度 | async emit_progress | await emit_progress | await emitProgress | EmitProgressAsync |
| 结构化结果和错误 | Result/PluginError | typed result/PluginError | typed result/PluginError | typed result/PluginException |
| 受控host资源/data/凭据/任务 | async HostServices | async HostServices | HostServices promises | HostServices async methods |
| 结构化媒体 | AssetRef + granted read | AssetRef + granted read | AssetRef + granted read | AssetRef + granted read |

这些类型是新重建的公共API设计，不声称现有SDK已有同名类。所有HostServices方法严格对应协议白名单；不存在`get_database`、`get_admin_token`、`run_shell`或通用无限制HTTP能力。未协商的服务在本地即返回capability_unavailable，核心仍须独立检查，SDK预检不是安全边界。

每个新handler返回 `{value,usage,effects,cost_scope}`，或带安全category/domain/effects的错误。具体DTO字段只从protocol生成/校对。工具、模型、hook、命令、入站poll、MCP、沙箱和子任务使用同一取消/事件/错误基元；不会因Python例外或C#任务取消自动转成成功空字符串。

<a id="sdk-values"></a>
## SDK-02 编码、字节和状态

UTF-8字节长度用于帧和配额，不能以Python字符数/JavaScript UTF-16长度代替。ID是字符串；generation及其他wire整数只接受协议safe整数范围。未知usage为null，不由语言默认值填0。缺字段与显式null分别保留；金额为十进制字符串。错误不能包含密钥、完整私有URL、远端原始正文或环境。

provider opaque_state作为受限JSON树原样往返，稀疏tool-call索引、数组顺序、签名串及非ASCII内容不丢失。不把相同tool name当相同调用ID。核心发起工具调用时，模型原始tool-call ID作为不透明`call_id`传给worker并须与历史`tool_call_id`一致；它不同于JSON-RPC `id`与通知`request_id`，不表示授权或重放。媒体只传asset_ref及有范围的read_handle，读取到的字节和sha256可由共同fixture检查；SDK不能自行打开远端提供的宿主路径。

进度只能用协商通知和当前root ID，且有界阻塞/背压；消费者慢时显式slow_consumer并关闭根调用，不能静默丢字段或攒无限数组。SDK将`tools.call.params.call_id`作为不透明业务数据原样交给handler；它与JSON-RPC `id`和通知`params.request_id`分开，不表示授权或幂等。每根调用恰好一个终态，通过原子终态门锁拒绝第二次完成；但已知副作用回执必须留在最终结果/错误中。用户取消、截止、连接断开和runtime异常分别归因，cancel不进入普通重试分支。

初始deadline转换为剩余时长后用单调时钟倒计时；通知/重连不能延长原根deadline。SDK把取消传给handler与HostServices，在本地截止后不再提交新子请求；宿主仍负责强制终止不合作进程。重启后无活的InvocationContext，缓存的handle全部不可复用，重建持久工作只能由核心调度一个新根调用。

<a id="sdk-lifecycle"></a>
## SDK-03 注册、资源释放与长期任务

handler注册表按 `(method,feature)` 固定，重复注册拒绝；配置schema校验与代码导入分开。Python `--list`只读entry-point元数据，不import插件；`--entry-point`必须唯一；四语言都不得在只读发现时偷偷联网下载运行时或依赖。运行依赖按实例准备并锁定，清单可以公开安装前提，但SDK不因调用失败自行修改宿主环境。

资源生命周期为 `created → initialized → handling → terminating → closed`。初始化失败不得进入handler；取消/错误也执行有界cleanup，cleanup不得发起新计费/出站操作。异步任务、流迭代器、文件句柄和子进程由作用域关闭；脱离作用域的后台task不会成为可靠任务。持久后台协作必须经`host.task.submit`取得child_run_id后由核心托管；权限、媒体、deadline和预算不得比父调用更宽。

公开hook API只接收获准数据快照和允许修改字段，不能把可变核心内部对象交给插件。旧生态内部对象需要加载器显式代理，SDK没有“兼容模式=管理员”开关。工具效果声明是提案：read_only、idempotent_write、non_replayable_write经宿主/核心政策判定，不能用自行标read_only绕过审批。

<a id="sdk-conformance"></a>
## SDK-04 同一夹具、四个真实进程

重建后提供一个共同harness；每种SDK由独立真实进程运行，记录运行时/OS/架构、SDK和配方版本、命令、退出码、帧摘要、下游计数与结果。基础向量与扩展向量分别计数；缺语言运行时只记not_executed。Linux一次通过不代替Windows/macOS，虚拟化transport也不代替named pipe原生证据。

每语言除echo外，必须生成以下最小样例：保留opaque_state的流式模型、可取消工具、带版本CAS的数据插件、主动/被动hook、媒体读取、平台poll/send、MCP适配、带权限递减的子任务代理。普通样例默认只连接本地受控服务，无真实账号/付费/外发。每个样例有成功、错误、取消、撤权、重启和下游核对；不以“能够转发JSON”代替功能等价。

| 共同向量组 | 必须比对的真实结果 | 验收ID |
| --- | --- | --- |
| 当前1.0 + 1.1-draft协商 | 基础零回归；未知feature不业务执行 | R13-SDK-001 |
| Unicode/整数/null/opaque_state | 四语言语义相同；帧字节限额相同 | R13-SDK-002 |
| progress/slow consumer/双终态 | 有界结束；每root一终态；正确ID | R13-SDK-003 |
| 取消/截止/断开/异步cleanup | 无后续新副作用；进程和句柄被收回 | R13-SDK-004 |
| host资源/data CAS/凭据broker | 真实字节一致；并发冲突；秘密零泄露 | R13-SDK-005 |
| hooks/commands及隔离 | 顺序/父过滤/撤权一致；跨实例调用0 | R13-SDK-006 |
| model/媒体/platform/MCP | 参数与引用/回执不丢失；unknown不重发 | R13-SDK-007 |
| task.submit及重启 | 相同child_run_id可查；不生成孤儿常驻任务 | R13-SDK-008 |
| 打包/依赖发现/干净安装 | metadata发现无导入；无无声依赖下载 | R13-SDK-009 |
| 文档与机器索引 | 每公开能力有schema、样例、错误及运行状态 | R13-SDK-010 |

<a id="sdk-docs"></a>
## SDK-05 文档、版本和外部前提

生成公共API参考、共同JSON schema和向量数据是实现产物，放在实现工作区；配方只保留Markdown合同和声明式样例。机器索引每项含 `sdk_version / protocol_version / feature / method / schema_ref / example_ref / effect / platform_status / acceptance_ids`，人类文档从同一表生成，避免两份不一致。新增特性、错误或弃用要同步四语言映射；暂缺实现的语言返回明确unsupported并保留缺口，不能从总表删除。

每语言教程至少包括：空环境安装必要运行时→安装锁定SDK→创建manifest/实例→只读发现→获得最小授权→调用→核对下游→取消/恢复→卸载和数据选择。教程的环境版本沿用来源快照并明确是固定基线，未来升级另核实，不声称“最新”。应用版本、配方版本、wire协议版本、SDK发行版本和外部MCP日期版本分别记录。

本合同不要求服务器启动Python、Node或.NET来提供Rust核心服务。调用选择了相应语言插件才需要相应运行时；是否可原生运行、打包、签名和发行为独立验收。原仓库自有开发技能保持普通文件，本包不创建技能仓库，不将普通Git源码当配方依赖。
