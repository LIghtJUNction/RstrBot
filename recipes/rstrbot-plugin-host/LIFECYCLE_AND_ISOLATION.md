# 插件安装、生命周期、数据和隔离合同

范围：RB-005/006/014/015/017/018/019/029/034/045/047/051。新增能力为 `specified_not_implemented`，新增运行验收均 `not_executed`。现有可信本地程序、Unix单调用进程、OpenAI/diagnostic/Webhook worker继续按 [BUILD_PROMPT.md](BUILD_PROMPT.md) 重建。此文件定义未来所需行为，不宣称当前已能装任意仓库或隔离恶意本机程序。

<a id="host-install"></a>
## H-01 安装来源和原子激活

安装输入为 `source_kind:local_manifest|archive|git_commit|index_entry`、canonical_source、精确commit/版本、expected_digest、runtime_kind、license_record和权限提案。local_manifest仍限实例管理员；工作区管理员只配置已批准包的实例。未知来源不可因索引排名或与旧包同名获得信任。`package_identity = canonical_source + package_id`；不同来源相同id显示冲突并分开展示，不覆盖现有包。

Git与索引是获取候选插件源码/包的外部来源，**不是Recipkg配方依赖**。安装前锁定commit，构建与依赖准备在独立准备区完成；禁止把Git HEAD悄悄当固定版本。离线可使用已验证且摘要完全匹配的缓存；没有匹配缓存准确报告offline_missing。索引失效不影响已有安装，用户可输入直接固定来源。兼容索引可读取`astrbot_plugin_`命名，但该前缀不是GitHub topic；向topic搜索映射时使用服务实际允许的语法并保留原包名，不为兼容重命名仓库。

状态顺序为 `discovered → downloaded → verified → prepared → awaiting_permissions → staged → active`。任何阶段可 `failed`，显式取消为 `cancelled`；取消只删除本次未激活准备区，不卸载旧包。所有阶段有operation_id、package_digest、阶段错误和重开后可读的进度。读取来源/许可证/权限是只读动作；下载、依赖安装、构建钩子和运行代码分别记录实际发生的行为，未批准安装不得执行仓库安装脚本。

新版本先校验可执行摘要、配置schema、协议、权限差异和可逆迁移，再切换活动代。扩权时旧授权不能自动延续；新版本拒绝权限或握手失败，保留旧活动代。切换采用“先停止新派发→撤销旧代租约→有界回收旧进程→提交新代→重新开放”的受控窗口；窗口显示updating，不承诺无停机。旧代迟到结果留诊断，不能覆盖新代配置或制造第二次业务终态。

重启时 `staged` 未提交的版本不激活；已提交代按数据库事实恢复。旧版本回滚需要可读旧schema或已验证恢复点；不可逆数据迁移必须先显示不可原地回滚并要求独立备份确认，不能先升级后宣称“一键撤回”。升级本身没有权限补发历史pending/unknown操作。

<a id="host-runtime"></a>
## H-02 运行时、监管与背压

每个包声明 `runtime:rust-native|python|node|dotnet|other-approved`，精确启动命令来自已批准manifest，不接收模型任意命令串。缺运行时只将相应实例标runtime_missing，核心及其他worker继续可用。安装运行时是管理员显式动作；不因打开插件列表自动联网安装。Python冲突依赖按实例独立环境准备，其他托管运行时同样锁定包依赖，不能共享可写的跨租户环境。

既有全局8槽、10秒排队、5秒连接/握手、默认120秒且最多300秒调用边界保留。新调度按工作区排队轮转，同工作区不能持续抢占全部空槽；槽位、排队深度、拒绝数、取消延迟各自可观察。每个请求记录配置deadline，队列等待计入总deadline；收到撤权/取消时，即使数据队列满，也有独立取消通路。回调不保持数据库事务等待进程或网络。

允许可配置的重连退避只用于无业务副作用的连接建立。`checksum_mismatch / unsupported_protocol / invalid_manifest / permission_denied / runtime_missing`停止自动重启并显示修复动作；临时建立失败可按有限次数、上限间隔和deadline重试。重试配置必须有实际有限值，缺配置即零次自动重试。任何业务调用重试由核心根据意图和下游回执决定，宿主不得自己重发。

进程终态含退出码/信号/原因、清理结果和有界stderr摘要。取消、deadline、host future丢弃、应用退出均收回所属进程和临时目录。Unix进程组回收只适用于可信worker；Windows需要私有named pipe的SID访问控制及Job Object真实子树回收，未实现时维持unsupported_platform。Linux/macOS/Windows逐平台验证，不从Unix一次通过推导三平台达标。

<a id="host-data"></a>
## H-03 配置、持久数据、卸载与秘密

包、实例、配置组、运行代、调用分别标识。实例的配置为 `(workspace_id,instance_id,config_group_id,version)`；机器人连接再绑定connector_id，不用包名推断共享。界面显示schema version、有效值、继承来源和秘密“已配置”标记；空输入表示保留、明确clear表示清除、disabled表示停用，三者不混淆。CAS冲突返回当前版本和安全字段差异，不自动覆盖旧草稿。

长期插件数据通过协议`plugin_data.v1`服务保存在受限namespace中；无数据库句柄、任意路径或全局KV。实例共享数据须显式标为共享；个人数据按用户再次分区，工作区管理员不隐式取得个人值。迁移声明输入schema、输出schema、是否可逆、资源清单和恢复方法；备份必须纳入必需插件数据与外部状态清单，缺一项报告incomplete，不悄悄宣称完整灾备。

卸载输入必须选择 `keep_data|export_then_remove|remove_data`，默认keep_data。先禁用、撤销租约、取消任务、移除工具/命令/路由/钩子注册，再处理数据。export_then_remove只有在导出已持久化、摘要确认且用户确认交付后才删；导出下载断线不算交付。保留数据以包来源、schema和所有者标识，后续同名不同来源包不得认领。已完成/unknown操作回执和必要审计不随普通卸载清掉，避免旧幂等键变成新发送。

凭据代理使用核心签发的opaque handle和固定request_template_id。模板冻结scheme/host/port/path、方法、允许参数、受众、签名算法和响应字段；插件不能覆盖认证头或把handle送到新主机。注入认证与可信签名/刷新组件是两种需要分别验收的实现；刷新按凭据版本合并，撤销/更换目标使旧版本失效，失败不切到另一个计费账号。普通工具及其他实例永远得不到原始密钥。

<a id="host-hooks"></a>
## H-04 钩子、命令与事件观察

协议扩展中的hook_phase和DTO是唯一线格式。宿主以本次运行快照的稳定顺序调用已启用钩子，记录输入版本、事件ID、耗时和返回补丁。策略钩子失败默认停止相应动作；只读装饰钩子可显式配置skip-with-diagnostic。取消、权限失效和unknown不能降格为普通异常触发备用发送。模型流与非流、主动计划与入站消息都经过对应阶段，清理钩子无论成功失败均有有界机会，但不能靠清理钩子发送新消息。

`inbound.observed`是观察私有内容的独立能力，默认关闭；管理员启用还需账号/会话所有者的有效内容授权，界面显示disabled_reason。仅配置“允许插件执行工具”不允许旁听所有消息。平台收到消息→核心鉴权和归属映射→入站事件持久化→授权观察广播→触发门控；未授权订阅者计数为0，静默门控仍可推进消费游标。撤销观察权限后不继续发送排队内容，已获得的数据无法声称被远端撤回。

命令使用`<source>/<package>/<command>`的稳定身份；display_name和alias不当身份。父命令过滤、子命令权限、参数schema、平台支持都先检查；alias冲突拒绝激活，不以“最后注册者获胜”静默遮蔽。平台命令目录从同一获准注册表导出，卸载/禁用后移除。选择某个Runner不自动赋予其所有监听器/命令/工具权限。调试只读查看与生产副作用执行分别授权。

<a id="host-sandbox"></a>
## H-05 技能资源与真实沙箱

技能文本、图片/PDF等资源读取和执行代码是两项能力。合法读取输入是版本化asset_ref/read_handle；返回实际字节或经批准转换的内容并记录消费摘要。只读取资源无需启动执行沙箱。技能文本是任务数据，不能因为含“忽略权限”或自称系统指令就取得额外能力。

执行输入须有program_ref、明确隔离profile、只读挂载资源、可写输出区、网络策略、CPU/内存/进程/输出配额和deadline。`isolation_profile`限定为 `trusted_native|restricted_process|isolated_runtime`；trusted_native明确不防恶意同账号代码，不能用于要求不可信执行的任务。后两者只有实际后端实现并通过探针才显示ready；无法建立隔离返回sandbox_unavailable，不退回宿主shell。

ready探针逐项证明：其他租户目录不可读；宿主凭据/环境不可读；无授予网络请求失败；路径穿越/符号及硬链接/压缩炸弹被拦；子进程逃逸不能存活；限额触发后核心仍可取消。仅“容器启动成功”不等于所有边界通过。能力表分别记录OS/架构/内核或运行时/后端版本、实际支持项和blocked原因，最低设备/性能值未测保持未知。

产物先写隔离临时区，完成后校验类型、大小、摘要、恶意路径和ACL，再以原子步骤发布为核心asset_ref。部分失败/取消的产物标partial并等待用户保留/删除决策；未确认已交付的文件不得自动清空。重启只恢复已发布元数据和可核对的临时记录，不自动重跑程序。读回产物需重新鉴权，不把沙箱内部路径发给浏览器冒充可下载文件。

<a id="host-license"></a>
## H-06 许可证与外部前提

每个第三方包/样本记录来源URL、精确版本/commit、内容摘要、许可证路径/摘要、允许的使用方式和分发义务。未识别许可证标license_unknown，不随RstrBot的MIT声明重新授权。兼容行为说明可独立撰写；若实际分发第三方代码、模板、图标或运行时，需满足其各自许可证和商标边界。未知分发许可的默认取舍是只提供获取/验证说明，暂不捆绑。

网络仓库、语言运行时、系统隔离设施、真实供应商账号、平台机器人权限、OAuth应用、签名/公证和商业配额均属外部前提，不加入八包配方依赖。安装私有包所需凭据只用于明确来源，诊断/任务归档不得包含token、Cookie或真实消息。本轮只交付Markdown合同，不执行安装/升级/发布。

## 验收关联

H-01对应R13-HOST-001/002/003；H-02对应004/005；H-03对应006/007/008；H-04对应009/010；H-05对应011/012；H-06对应013。每个场景操作和真实下游核对位于 [ACCEPTANCE.md](ACCEPTANCE.md)，缺平台/运行时为blocked或not_executed，不能从材料齐备改成passed。
