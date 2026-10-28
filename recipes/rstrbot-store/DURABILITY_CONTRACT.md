# 持久状态、迁移与恢复完成合同 · 1.13

状态：本文是新的重建合同，新增 schema 3、外部状态完整备份、多节点和新增对象仍为 `specified / not_implemented / not_executed`。当前 schema 2、`rstrbot-db-v2`、v1 导入及启动恢复合同保持；现有 12 项验收保留。本文件只定义可观察数据保证，不要求照抄某套表名或 SQL。

数据层本质上保存“谁拥有、哪个版本、是否执行、实际结果”。缓存、插件内存和浏览器都不能成为这些事实的唯一来源。

<a id="S01"></a>
## S01 逻辑数据与归属（RB-002/003/004/005/007/018）

所有业务对象含工作区、稳定主体、可见范围、对象 revision、创建/更新时间；实例全局对象与工作区对象采用不同类型/访问入口。用户/服务主体、交互会话、个人 API 凭据、持久委托是不同记录，不能用一个可变 user_id 字符串互相替代。

| 事实 | 必需关联与不变量 | 失效行为 |
| --- | --- | --- |
| 账户、成员、角色 | 账户稳定ID；成员唯一 `(workspace,user)`；角色封闭五值；最后owner保护 | 删除/降权按当前成员检查；未知角色拒绝，缓存不扩大权限 |
| 服务主体/个人凭据 | 主体类型、workspace、grant_revision、到期、token哈希、负责人、disabled/revoked | 个人权限与本人当前权限交集；服务按独立grant撤销；不保留明文token供事后读取 |
| 资源/分享/附件 | owner主体、visibility、ACL revision、父资源与资产版本；默认private | 父级撤权/删除立即阻断子对象访问；公开父对象不自动公开未授权子对象 |
| 外部绑定/秘密 | 加密密文、key_id、provider、完整目标、外部稳定主体及revision | 目标变化禁保留旧secret；轮换原子换指针，失败保留旧值；日志/备份清单不含明文 |
| 插件实例/数据 | package来源摘要、instance/workspace/机器人scope、generation、data schema | 同包不同scope不同数据；旧generation写入失败；卸载数据保留/删除策略有显式记录 |
| 运行/attempt | initiator/authority_ref、conversation/generation、配置/模型/人格快照、限额、attempt ID | 撤权禁止新派发；终态/已知费用不可被取消抹掉 |
| 计划/occurrence | owner、schedule revision、timezone/tzdb、planned instant、claim | 唯一occurrence防重；所有者意图不能被别的admin覆盖 |
| 投递/工具意图 | idempotency scope、规范化digest、目标/参数/批准/凭据版本、状态 | unknown不复活为prepared；外部状态核对追加事实，不能重新发 |
| 原文/索引/摘要 | source/version、chunk、index identity/generation、派生来源范围 | 来源失效立即排除派生；清理可延后但不能继续检索 |

同一业务操作的 HTTP/CLI/MCP/SDK 调用经过同一事务函数。验证资源属于工作区、验证当前成员/ACL、CAS 与写入在可串行化的同一逻辑边界内；不能先在外面做授权再让过期角色通过写入。耗时密码哈希和网络在事务外，提交时重新验证其依据版本。查询先授权后分页、排序、计数；越权不通过总数或缓存键泄露。

验收：`R13-STORE-001/002/003`。

<a id="S02"></a>
## S02 准入、幂等、事件与预算（RB-008/013/025/028/029/031/032）

以下伪代码定义一个事务边界；不是可执行钩子。

```text
admit(command, trusted_context):
  begin short transaction
  require current_authority(trusted_context, referenced_objects)
  require all referenced revisions and run generation match
  old = intent(scope = workspace + actor + operation_name + key)
  if old exists:
    require canonical_input_digest == old.input_digest
    return current authorized view of old
  require finite queue/attempt/step/deadline limits
  reserve budget atomically, or reject
  persist intent(prepared), reservation, operation, acceptance event
  commit
  dispatch only after committed intent, with fresh authority checks
```

规范化摘要包括所有语义输入、资源版本和目标；字段排序固定，UTF-8 字节按业务正文计算。日志脱敏、显示修剪或消息去重不得改变规范化输入。重复键同参数返回原操作，异参数冲突；未通过授权/DTO校验不消费幂等键和预算。不同有效意图不能因为内容相同被全局去重。幂等记录保留周期不能比外部后果可存在的周期短；归档需保留拒绝重放用的最小 tombstone。

出站前先持久 prepared→sending，再实际派发。崩溃点可能落在“已经sending但尚未实际写出”，恢复仍 conservatively unknown；此误差宁可要求核对，也不自动重发。远端accepted→本地回执提交之间无法由数据库事务原子覆盖，返回独立 delivery/persistence，持久原意图作为后续核对锚点。不得宣称对无远端幂等/事务的系统 exactly-once。

事件具有唯一event_id、所属workspace/资源/run/generation/attempt以及单调cursor。一个运行只能CAS写入一次执行终态；投递/持久化后续核对是独立事件。保存消息/流片段/终态及相关事件的事务尽量同边界，不能发出已持久化的假回执。事件读取重新授权；过期cursor明确重同步，不能偷偷遗漏。长期历史、投影generation、branch与删除tombstone分别记录。

金额用有溢出检查的整数微美元，预留/结算唯一 `(attempt_id,budget_component)`；settle重复不会双计。并发 reservation 不得超过可保证的硬限额，已知超出估值的实际费用仍入账；未知usage/price不能释放成零。parent reservation 子划分给child，不能同时让父子花同一剩余额度。取消/撤权释放的仅是确认未发生部分；unknown保留冻结/核对余额及覆盖度。

验收：`R13-STORE-004/005/006`，核心 `R13-CORE-009/013/014/015/016/022/039`。

<a id="S03"></a>
## S03 双数据库、缓存与过载（RB-009/010/011/045）

SQLite 保留单核心、WAL、完整耐久、短事务、有限锁等待与有界写队列。写队列准入、锁等待、磁盘满、磁盘I/O错误分别诊断；不能统一成 timeout 然后自动执行同一外部动作。已返回 durable accepted 的记录在重启后必须存在，无法保证时必须失败且不派发。长读导致 WAL/checkpoint 滞后需暴露指标与处理建议；取消/暂停不被无限工作队列饿死。

PostgreSQL 必须使用同一业务向量，验证排序/分页、CAS、隔离、唯一约束、预算/认领、恢复和错误分类。迁移账户与普通运行账户分离；普通运行账户无schema修改或跨tenant维护权。若采用 RLS，连接池每事务设置并复位workspace/主体，测试异常/取消/rollback后归还连接；若不采用 RLS，显式记录应用ACL边界并运行同样越权测试，不能宣称已有RLS。后台维护是独立明确权限，不能偷用通用业务连接绕过隔离。

Valkey 仅保存可重建presence/缓存；断开、清空、过期、重启不能丢意图、授权、硬预算、claim或事件。缓存键包含workspace和对象/ACL revision，读写失败可degraded；无法确认权限和预算时拒绝相应动作，不用旧缓存继续执行。cache stampede用有限并发合并，不能把回退变成无限数据库负载。

性能结论保留固定硬件、数据规模、连接/worker数、负载分布、失败和重复数；没有测试不填写延迟/RSS达标。数据库时间和系统墙钟只用于定义的用途；lease使用数据库可比较时间/单调计数，不能让客户端随意时间戳取得认领。

验收：`R13-STORE-007/008/009`。

<a id="S04"></a>
## S04 资产、索引与插件外部状态（RB-006/012/018/026/035/036/037）

资产上传清单含upload_id、creator、target/version、块号→长度/摘要、总长度/摘要、media_type和到期。published前必须校验完整性与目标当前ACL；同块不同摘要409；合法断点只补缺块。blob先写临时对象并fsync/确认后，数据库原子发布可读指针；数据库失败保留可收集孤儿，不发布悬空引用。取消/到期只清理本次未发布对象，不触碰其他上传或既有asset。

删除先事务tombstone及ACL版本变更，任何文件/缩略图/转码/索引读取立即过滤；物理GC按有界后台任务清理无引用内容。跨工作区去重即使内部共用blob也不能暴露另一租户文件存在或共享引用计数。软硬链接、路径逃逸、对象存储重定向与未经允许的公共URL不得成为授权绕过。

文档原文、chunk与index generation具有不可混淆身份。新索引在building期间保存检查点；ready需要全部被声明可用的结构同代；active指针CAS切换。取消/失败/重启不会把半代标active，也不会抹除并发已提交文档。source更新/删除或ACL改变使派生摘要/记忆/向量失效，恢复期间同样先授权再检索。

插件可持久数据需通过instance/workspace/scope绑定API或受保护专用状态根；目录是外部状态，不在当前数据库备份中。插件声明数据schema、可恢复性、版本迁移、是否有一致快照接口和不可逆边界。更新准备新数据代失败保留旧代或明确受控停机；卸载的保留/删除选择被记录，可逆数据策略不能假称覆盖不可逆迁移。缺少必须的插件状态使 complete backup不完整，而不是忽略后仍显示成功。

验收：`R13-STORE-010/011`。

<a id="S05"></a>
## S05 schema 迁移、导出与恢复（RB-002/005/012/013/051）

当前数据库 schema 2 与 `rstrbot-db-v2` 原格式不改。未来扩展新增 schema 3，只有新实现实际完成迁移/旧数据向量测试后才启用新HTTP/worker能力。迁移日志保存from/to版本、步骤摘要、开始/提交事实与备份引用；所有schema变更及原数据映射在受限迁移连接内事务执行，不能边监听边做半完成升级。

| 输入/动作 | 允许结果 | 失败/取消/回滚 |
| --- | --- | --- |
| 空数据库首次初始化schema3 | 同一次管理员初始化事务创建必要结构，私有默认 | 取消/失败不留下可被匿名接管的半管理员 |
| 现有schema2→3 | 保留所有ID、owner/ACL、消息/事件/预算、幂等键/回执、已消费计划时刻；新增对象表为空 | 验证失败回滚；已有未知意图不能迁移成prepared；旧值不自动映射为更宽权限 |
| v1/v2数据库备份导入新实现 | 空目标、校验来源格式→按原导入规则恢复到支持schema，再显式迁移3 | 非空目标拒绝；失败保留来源文件；未校验前不启平台/计划 |
| schema3数据库导出 | 新显式 `rstrbot-db-v3`，含schema3逻辑状态与算法/版本摘要 | 旧v2格式不能静默丢服务主体/新意图/资产引用；不伪装向后兼容 |
| 新完整备份 | `rstrbot-backup-v3` 清单关联数据库、asset blobs、插件状态、版本和摘要；key独立保管 | 必需状态缺失时status=incomplete；require_complete直接失败，allow_incomplete展示缺项 |
| 旧二进制读取schema3 | 拒绝不支持schema | 不自动降级；要回滚先停机，用升级前备份恢复到空目标 |

新完整清单最小字段：`format, created_at, application_revision, schema_version, consistent_boundary, database:{size,sha256,format}, assets:[{id,version,sha256,size,owner_ref}], plugin_states:[{instance_id,generation,schema,sha256,required}], key_ids:[fingerprint], exclusions:[{kind,reason}], completeness, unresolved_operation_ids`。清单可记录必要归属，不能公开分享。活跃会话、认证挑战、OAuth state、邮件token和明文秘密不进入备份；恢复不能复活个人会话或一次性挑战。加密密文与key分离，缺key不能假装凭据可用。

快照边界先暂停新外部执行与会改变资产/插件数据的发布，等待有界安全点；无法一致快照的插件标缺项，不能无限等。数据库快照、已发布blob清单、plugin快照全经摘要核验，未发布临时对象可以显式排除。取消快照保留原数据，清理本次未完成输出；不覆盖已有备份文件。来源路径/目标label必须经过受限运维边界，不给普通API任意文件读写。

SQLite↔PG转换先读一致导出，导入空目标，核对总数及逐类摘要、稳定ID、归属、秘密key_id、索引身份、附件清单、预算、幂等、计划已消费时刻、未知投递。验收成功前来源仍是唯一运行库；不双写或自动切流。失败保留来源/报告，重试清理或新建空目标；不能把重复导入当正常append。

恢复后进入quarantine：不启模型、平台、计划、webhook、后台child或重建中可能收费的步骤；恢复主体/权限先验证。只读检查报告缺key/缺外部数据/unknown费用/unknown回执及旧版本能力。显式 complete恢复动作只解除未来新执行，不重放旧unknown或错过时刻；必须逐项核对的条目保持单独needs_reconciliation。恢复时重建纯索引也要当前授权和有限资源，不能恢复旧任意执行权限。

验收：`R13-STORE-012`，核心 `R13-CORE-036/037`。

<a id="S06"></a>
## S06 启动与多节点恢复（RB-008/031/032/051）

当前恢复扫描的每页128条、JSON批次16MiB、至少一条保证推进、稳定(created_at,id)游标/首批上界保留；这是内部恢复机制，不是公开授权游标。依次恢复runs、deliveries和schedule claims；全部完成才监听和启调度。失败时不开放半就绪写入口；可在本机诊断输出步骤/错误，不输出正文或秘密。

未完成run→interrupted；prepared delivery→not_sent；sending delivery→unknown；在途schedule禁用并needs_reconciliation。子run/工作流节点继承同原则：新进程不能自动spawn重复child，未知写节点禁止下游继续。已完成occurrence和终态保留，不因第二次恢复“再次恢复”而丢执行历史。纯计算任务可按当前版本/ACL与检查点显式恢复，绝不能与外部发送混为一种重试。

未来多节点只在PostgreSQL、共享资产/插件状态边界以及实际故障证明具备时启用。每个claim包含 `scope, object_id, owner_node, epoch, lease_until, object_revision`；认领在数据库事务内提升epoch。所有落库和派发前检查当前epoch；过期owner晚到回执保存为核对事实但不覆盖新代。leader身份不能来自Valkey唯一记录；缓存损坏不产生第二有效claim。

外部系统若不能接受fencing或幂等，节点在send边界死亡只能unknown并人工核对；新owner不能以lease过期为重发许可。网络分区、暂停旧节点、时钟偏移、数据库故障、共享文件不可用、worker延迟返回都必须注入。没有这些证据时capability supported/available保持false或明确disabled，不能因“用了PG”宣传多节点。

验收：`R13-STORE-013`，核心 `R13-CORE-022/023/038`。

<a id="S07"></a>
## S07 新增验收的共同证据

以下全部为本次**未执行的软件验收**，具体步骤在 `ACCEPTANCE.md` 的 R13-STORE 表。每场景至少保存输入fixture摘要、实现revision/数据库schema、实际命令/时间、观察到的对象与事件ID、下游计数/摘要、故障点、恢复读取和断言结果。使用纯合成账号与内容；可以保留脱敏摘要，不把数据库/密钥/原消息作为对外配方附件。

同一向量先分别在真实本地SQLite与独立PostgreSQL运行；不能用mock内存库替代双数据库证据。数据库中断测试应观察HTTP/worker实际计数，确认“拒绝后未发外部请求”；单独查看返回码不够。迁移/备份测试只用临时空目标，不触碰现有服务。
