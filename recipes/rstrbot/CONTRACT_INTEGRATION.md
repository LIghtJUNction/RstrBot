# 1.13 依赖、接口、版本与外部前提

本文件只做跨组件定位与约束；字段/权限/状态的规范仍在负责组件。本文所说“定义完成”指可审阅的重建材料，不指接口已经在运行程序中存在。

## I01 八包版本与无环闭包

| 包 | r2输入版本 | 本次材料版本 | 直接依赖别名→包 | 主要新增合同 |
| --- | --- | --- | --- | --- |
| rstrbot | 0.2.0-recipe.6 | 0.2.0-recipe.7 | desktop→rstrbot-desktop | 52覆盖、派生追踪、集成/验收约定、竞品执行册与issue回归 |
| rstrbot-desktop | 0.2.0-recipe.1 | 0.2.0-recipe.2 | client→rstrbot-web | NATIVE_DELIVERY.md |
| rstrbot-web | 0.2.0-recipe.5 | 0.2.0-recipe.6 | service→rstrbot-core | CAPABILITY_WORKFLOWS.md |
| rstrbot-core | 0.2.0-recipe.2 | 0.2.0-recipe.3 | storage→rstrbot-store；plugins→rstrbot-plugin-host | DOMAIN_CONTRACTS.md、EXTENSION_API.md |
| rstrbot-store | 0.2.0-recipe.1 | 0.2.0-recipe.2 | 无 | DURABILITY_CONTRACT.md |
| rstrbot-plugin-host | 0.2.0-recipe.1 | 0.2.0-recipe.2 | sdk→rstrbot-sdk | LIFECYCLE_AND_ISOLATION.md、ADAPTERS_AND_COMPATIBILITY.md、COMPATIBILITY_FIXTURES.md |
| rstrbot-sdk | 0.2.0-recipe.1 | 0.2.0-recipe.2 | protocol→rstrbot-plugin-protocol | SDK_CAPABILITY_CONTRACT.md |
| rstrbot-plugin-protocol | 0.2.0-recipe.1 | 0.2.0-recipe.2 | 无 | EXTENSION_PROTOCOL.md |

输入优先级：00/20及根1.12明确用户约定 > 对应包合同 > 历史/生成器附录。`12-complete-rebuild.md` 在最后一个归档文件后附有通用“仓库约定”，其中要求生成源码独立仓库/子模块，与本任务已确认的generated临时产物规则冲突；该段不是13归档内文件，不能覆盖用户约定，也不复制进项目技能。保留输入12原文用于追溯，本次不创建任何源码/技能仓库。

作者树仍为 `recipes/<package>/`，RECIPE的source只指向真实相邻Markdown配方。递归打包时source可改写为该包 `.deps/<alias>` 并重算LOCK，所有别名只在其所在包有效。独立重建叶子包不读取父包文件；host接收core提供的broker实现，不反向声明core依赖。根集成合同/覆盖索引是整体验收入口，不是叶子编译所需的隐藏文件。

必须按 protocol→SDK→host、store→core→Web→desktop→根的拓扑检查。八包均受本轮合同影响而升版；包版本不是HTTP/插件/MCP/schema版本。服务器构建可只消费core及Web产物，不安装桌面/Node运行时；浏览器无需Rust。不得把generated、普通Git源码库、数据库、账号、工具链或外部服务加入配方依赖冒充闭包。

## I02 单一真相与当前/未来分离

| 对象 | 唯一规范位置 | 当前与扩展关系 |
| --- | --- | --- |
| 当前HTTP/DTO/错误/数值范围 | core OPENAPI.md（完整声明）与HTTP_CONTRACT.md（语义） | 仍为API v1，66 API＋2健康；本轮不改其字节，旧程序不会因新文档增加路由 |
| 未来HTTP动作/DTO/发现/只读核对 | core EXTENSION_API.md | `rstrbot-http/2-draft`、`/api/v2`为未实现目标；封闭registry、三态能力、严格版本与明确效果 |
| 领域状态/授权/预算/来源 | core DOMAIN_CONTRACTS.md | 现有v1权限继续有效；未来对象和策略只能在协商成功后开放 |
| 数据耐久、唯一性、备份迁移与fence | store DURABILITY_CONTRACT.md | 已有schema2/备份v2继续读取；新schema迁移按该合同独立验收，不假装当前格式已含附件/插件外部状态 |
| 当前插件wire | protocol PROTOCOL_REFERENCE.md、CONFORMANCE_VECTORS.md | 1.0保持原文及原向量，不把反向broker等新能力写成现有 |
| 未来插件wire、具体feature与共同向量 | protocol EXTENSION_PROTOCOL.md | 1.1-draft显式选择/交集，旧worker仍按1.0；功能失败不能降版重发 |
| 四语言表面与映射 | SDK SDK_CAPABILITY_CONTRACT.md | 四SDK遵循同一wire、类型和限制，不自行添加第五种语义 |
| 进程/插件/沙箱/兼容/MCP平台映射 | host两个新增合同 | 外部MCP profile另行冻结，不等于HTTP/插件版本；未实现的标准/profile/平台显示缺口 |
| UI与原生投影 | web CAPABILITY_WORKFLOWS.md、desktop NATIVE_DELIVERY.md | 页面只投影实际能力；按钮或教程存在不证明服务支持 |
| 原需求与历史 | 根REQUIREMENTS、TRACEABILITY、BASELINE-v1.4、history、SOURCE_HASHES | 保留原字节与ID，新覆盖/索引引用而不重写历史 |

保留历史原文时不重写其相对链接：history/BUILD_PROMPT-v1.12.md中的链接按原根包语境解读，部分链接直接在history路径打开会指向错误相对位置；阅读当前入口请用根BUILD_PROMPT。这是逐字历史保存的已知显示限制，不影响现行合同链接或递归闭包。

未来扩展绝不静默覆盖v1事实：若v1与扩展在动作名/数据形状不同，必须在core或protocol权威文件给明确桥接，不允许SDK/前端各自猜测。尤其资源revision与内容asset version、群聊触发规则、一次性凭据、子任务参与同意、审批后的精确参数必须有可编码的单一类型。只读查询不得消费token、发起新投递或预留模型费用。

## I03 跨组件边界与协商

入站平台是数据提供者，不能决定本地owner/workspace。core验证外部账号绑定和观察授权后建立稳定会话、来源、当前入口及批准目标。插件声明只产生候选能力：清单∩握手∩核心实现∩启用配置后，还要当前身份授权、资源范围、预算和后端可用性。新feature不加入1.0的原能力枚举。

core把身份/权限版本/父run/配置快照/截止/预算派生的最小上下文交给host。host的GrantHandle按root invocation、实例和generation绑定；worker的服务子请求只能走protocol白名单与每方法具体feature，不能任意调用HTTP管理端点。core在真正派发和返回用户结果时重新核权，host声称persistence=saved不等于数据库已提交。

所有相关组件共同使用同一operation_id/意图关联：Web重连只读core记录；core关联模型/工具/子任务；host回传实际效果；store保证唯一性与账本；desktop退出不篡改这些业务终态。取消、超时、权限撤销与网络错误均不能自动换一个会话、路由、供应商、协议版本或子代理来重放可能发生的效果。

当前model.list是发现而非生成；当前只读工具循环不会因草案存在而允许写工具。未来审批必须绑定最终参数/目标/版本/费用范围并在派发前再次核对；工具hook修改参数后原审批失效。一次性token只在明确的一次成功响应出现，日志、事件、持久查询和幂等重复响应不再次显示。

## I04 数据迁移与兼容

升级前记录配方、程序、schema、备份格式、插件profile和外部对象清单版本。先校验有足够空间和恢复材料，再在排他维护/明确多节点fence下迁移；事务或可恢复检查点确保旧/新完整状态，不允许半迁移后ready。`includes_external_state=false`的旧备份不能被宣传为完整附件/插件灾备。

旧v1/1.0客户端必须保留原有当前能力。新客户端发现v2不存在时回到已存在v1功能并显示扩展不可用，不发送虚构v1动作。迁移产生的新对象不能由旧客户端的宽泛JSON编辑绕过权限；旧插件profile与新协议分别协商。旧数据导入必须显式选owner/workspace，无法证明归属的匿名资料留隔离，不能归给第一个登录者。

恢复后暂停计划、平台、外部工具及模型派发，核对密钥/对象/插件状态与未决回执。只读核对和已授权修复不等自动恢复所有后台工作。schema不可逆时回滚用兼容备份，不能把旧二进制直接放回新库。外部accepted但本地保存失败保留该事实，unknown不因恢复成功变为not_sent。

## I05 供应商、平台、工具链与许可证边界

| 类别 | 需要部署者提供并冻结的前提 | 缺失时的行为 |
| --- | --- | --- |
| 模型/嵌入/搜索/工作流/媒体服务 | 实际端点、版本/profile、模型ID、认证、上下文/媒体限制、价格与使用授权 | 只显示支持目标与缺口；诊断夹具不算真实供应商证明，不默选另一个计费身份 |
| 聊天平台/邮件/Google/Passkey | 获准测试账号、平台权限/事件与路由限制、HTTPS/OAuth配置、真实邮箱/认证器 | 合成协议测试与实机分列；不开真实外发、不宣称同步或投递到达 |
| SQLite/PG/Valkey/共享对象 | 实际库版本、磁盘/连接权限、备份与对象共享、各环境恢复能力 | 权限/预算事实不可确认则拒绝危险操作；缓存退化不能放宽授权 |
| Rust/四SDK/Web/桌面系统依赖 | ENVIRONMENT来源快照及生成时锁文件/工具链/OS/arch，Tauri平台原生前提 | 未解析或缺设备记blocked；不声称所有“最新”或所有平台兼容 |
| 沙箱、签名与自动更新 | 真实隔离后端、平台权限、固定签名身份/信任根、分发服务 | 默认不开放不可信代码/自动更新，失败不退回无隔离或忽略签名 |

八包现有LICENSE.md为RstrBot材料许可，原文不改；它不替第三方依赖重新授权。生成实现对库、运行时、外部SDK、兼容插件、素材/字体与示例分别记录固定版本、来源、许可证/SPDX（能确认时）、NOTICE要求及再分发方式；无法确认标unknown并阻断含该内容的再分发。仅在普通进程中运行、用独立插件或引用仓库链接，不自动解决许可证/平台条款问题。需要实质法律判断时交人工核查，不能靠猜测授予权限。

本轮不复制竞品代码、界面素材或issue业务日志；只提炼公共行为与有来源的回归条件。兼容样本保存为外部测试前提并冻结摘要/许可，不把普通Git软件仓库塞进Recipkg闭包。纯技术文档引用和未运行候选版本不是已安装能力。

## I06 一致性验收

| ID | 实际检查与通过条件 |
| --- | --- |
| R13-INT-001 | 解析8个RECIPE TOML，作者source皆存在且为Markdown配方，拓扑无环/全可达；版本符合I01，无generated或普通源码仓库依赖 |
| R13-INT-002 | 解析原OPENAPI得到66＋2操作并比对基线hash；原协议参考/向量及所有LICENSE保持字节不变，新DTO/能力只出现在正确版本合同 |
| R13-INT-003 | 每个52条目有位置/责任/六边界/新验收；215族/287原验收ID与原文hash不变，派生索引无失联条目；1.12历史等于输入提示词 |
| R13-INT-004 | 各包BUILD_PROMPT能找到新合同，ACCEPTANCE引用有效场景；跨包名、能力ID、AssetRef、触发枚举、凭据响应与参与同意操作一致 |
| R13-INT-005 | 从归档读取完整递归材料且无作者绝对目录/generated/源码依赖；官方check/pack和本地材料检查各自记录实际运行情况，未运行工具不冒称通过 |
| R13-INT-006 | 用旧v1/1.0和新草案客户端分别执行兼容/缺能力/错版本/不合法字段向量；不能因网络或业务失败自动降版重放；此项需真实运行，材料检查不足 |
| R13-INT-007 | 迁移/恢复跨SQLite/PG、插件/对象代次和多节点fence；读回原归属/账本/unknown，外部发送不重复；此项需真实运行 |
| R13-INT-008 | 发行前核外部来源/许可/账号/目标版本清单，实际产物与声明一致；未知项阻断对应声明，不能把本轮材料交付记成发布 |

R13-INT-001至005的材料部分可在本次本地校验；006至008的运行/发行仍not_executed。完整结果以任务回传的验证报告为准，不预填passed。
