# rstrbot-plugin-host 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-PLUGIN-HOST-01 | 校验正确与错误摘要的本地 manifest，模拟缺失可执行文件 | 错误分类清楚，校验失败不启动进程 |
| RSTRBOT-PLUGIN-HOST-02 | 9 个等待调用占据 8 槽并让第9个超时；队列期间撤权 | 槽位有界；超时/撤权调用不派发业务请求 |
| RSTRBOT-PLUGIN-HOST-03 | 握手成功后撤权、禁用实例或更改 generation | 旧调用拒绝继续；新调用检查当前权限 |
| RSTRBOT-PLUGIN-HOST-04 | 取消等待进程及其子进程，并丢弃宿主调用 future | 进程组回收、直接子进程被 reap、临时目录清理 |
| RSTRBOT-PLUGIN-HOST-05 | 模拟慢通知消费者、超大结果和流末尾缺失 | 明确 slow_consumer/协议错误，内存有界，不静默成功 |
| RSTRBOT-PLUGIN-HOST-06 | 本地 OpenAI 测试服务返回稀疏工具片段与缺 usage | ID/片段顺序和 opaque state 保留，usage 与成本未知 |
| RSTRBOT-PLUGIN-HOST-07 | 发送到公开HTTPS、重定向、私网和DNS重新绑定情形 | 默认策略拒绝越界；私网须双重授权，TLS失败不降级 |
| RSTRBOT-PLUGIN-HOST-08 | MCP 固定目标发送一次，再用同ID重复、改正文、重启后重复 | 一次真实接收；同参数读回执，改参数冲突，不重放未知意图 |
| RSTRBOT-PLUGIN-HOST-09 | Webhook目标返回2xx、4xx、断线，并注入回执持久化失败 | accepted/rejected/unknown及persistence分离；无自动重试 |

## 1.13宿主和适配器验收

合同：[LIFECYCLE_AND_ISOLATION.md](LIFECYCLE_AND_ISOLATION.md)、[ADAPTERS_AND_COMPATIBILITY.md](ADAPTERS_AND_COMPATIBILITY.md)；线格式从声明依赖`sdk → protocol`取得。以下软件场景当前全部`not_executed`。实施者为每项记录精确配方/实现/SDK/运行时版本、OS/数据库、实际步骤/命令、下游记录和passed/failed/blocked/not_executed。下游服务只用独立合成账号与受控接收器；缺真实账号的live部分单列blocked，不能用fixture计作真实平台通过。

共同夹具：工作区A/B，各有用户owner/member及同名不同来源插件；下游HTTPS/模型/平台/MCP夹具可按阶段拒绝、延迟、接受后断连，记录请求ID/参数摘要/接收次数；磁盘/数据库可在intent提交、派发、回执提交点故障；可杀宿主/worker后用原数据重启。每个写操作先执行正常路径，再故障/撤权，并只读核对真实下游与持久意图。0请求和1次接收均须从接收器记录证明。

| 编号 | 输入、操作及故障注入 | 下游、恢复与隔离断言 |
| --- | --- | --- |
| R13-HOST-001 | 直接固定git/archive/本地manifest/离线缓存安装；错误摘要、同名不同来源、无缓存和取消 | 错误不执行代码；来源分离；旧包可用；取消仅删准备区；无中心索引仍能固定来源安装 |
| R13-HOST-002 | 活动vA准备vB，新增权限被拒绝/握手失败；随后批准合法vB | 拒绝时vA有效；切换期间updating；只激活一个代；旧代迟到无新业务写入 |
| R13-HOST-003 | staged前/后和代次提交前/后杀宿主；尝试不可逆schema回滚 | 重启按已提交事实恢复；未提交代不激活；不可逆回滚明确拒绝，保留可验证备份和旧数据 |
| R13-HOST-004 | 仅Rust启动；缺各托管运行时；实例A/B冲突依赖；超过8并发和连续A请求 | 核心不依赖全部运行时；A/B环境隔离；槽位有界且B不永久饥饿；队列deadline及拒绝可查 |
| R13-HOST-005 | Unix可信进程树取消/超时/父future丢弃；Windows私有pipe跨SID连接和Job回收；macOS本地退出 | 各平台分别给真实证据；无后续请求/孤儿进程；缺实现为unsupported/blocked，不能沿用Linux结果 |
| R13-HOST-006 | 同包两workspace/用户/配置组写数据，双CAS，改密/停用后读旧柄，再重启 | 归属独立；冲突不覆盖；旧柄失效；已提交数据准确恢复，0越界内容 |
| R13-HOST-007 | 依次keep_data/export_then_remove/remove_data卸载；导出途中断线并重启 | 注册/路由/任务/租约先撤；未交付导出不删；保留的unknown回执不变成可重放；不同来源包不认领旧数据 |
| R13-HOST-008 | 用固定模板代理注入认证、另用可信签名/刷新组件；换目标/轮换/撤销/并发刷新 | 两类实际下游分别证明；秘密不泄露；受众/版本错误0调用；失败不转别人的计费身份 |
| R13-HOST-009 | 流/非流、主动/被动任务跑全部hook；顺序patch冲突、慢hook取消、策略hook失败 | 确定顺序和版本；越权patch拒绝；取消无fallback发送；cleanup有界且不产生新副作用 |
| R13-HOST-010 | inbound观察默认关→只开账号A→撤销并制造迟到；父子命令/别名冲突/禁用 | A/B默认收到0内容；开启仅A；撤销后0新增；命令目录同步、父过滤有效；权限原因可见 |
| R13-HOST-011 | 不启动沙箱读取获准文本/图片/PDF，核实际字节/内容；另请求隔离执行但后端准备失败 | 阅读可独立成功；不存在“仅路径”假消费；执行返回sandbox_unavailable且宿主执行计数0 |
| R13-HOST-012 | 真隔离后端探测跨租户文件、环境密钥、私网/外网、软硬链接、解压炸弹、子进程逃逸；输出发布时取消/磁盘满 | 全部按profile拦截/限额；核心能取消；未交付产物不清空；重启不自动重跑，平台结果分开 |
| R13-HOST-013 | 安装记录缺许可证/来源/精确版本；依赖包含额外分发义务 | 标license_unknown/fixture_unfrozen并阻止兼容认证或未授权捆绑；MIT不覆盖第三方；已安装无关包不受误删 |
| R13-HOST-014 | 先保存供应商A再加B，B校验失败/取消/CAS冲突；发现为空/不支持；切目标不换密钥 | A配置/模型选择保留；B失败保留草稿；自定义ID仍可用；旧秘密未发新目标；list请求无生成 |
| R13-HOST-015 | 稀疏工具流/签名/多模态续轮；工具上限收尾；供应商失败与storage conflict分别注入；外部workflow接受后断连 | ID/状态/引用保持；无虚构转换；storage不触发provider回退；所有实际尝试记账；unknown job不重复submit |
| R13-HOST-016 | stdio与Streamable HTTP分别用冻结2025-11-25profile连接；首次初始化超时后修正重试；对方仅较新版本 | 支持者完成初始化再发业务；同配置能修复；版本不支持明确断开，0猜测调用；目录不提前ready |
| R13-HOST-017 | 同代10个MCP调用遇断线，注入连接永久坏配置；list_changed变工具schema | 合并一次重连；退避有上限；永久错误停止；新schema需重新批准，旧目录不覆盖新代 |
| R13-HOST-018 | MCP写工具接受后断线；OAuth刷新中撤权/换目标；远端sampling/elicitation/UI请求 | unknown不重放；token受众/主体不串；未授予能力拒绝；UI不自动网络/执行，0秘密自动填充 |
| R13-HOST-019 | 逐测五个trigger枚举与member_filter/名单校验；同事件重复、持久化前杀进程、重连、self-echo和引用旧bot消息；群静默与私聊并行 | 事件/游标不丢已确认记录；重复不规划；静默推进消费但not_replied；正常引用不误丢；私聊不被群安静策略阻断 |
| R13-HOST-020 | 真实文件/音视频夹具传输、断点中远端变更、大小/解码/ACL失败、撤权和取消 | 字节/摘要/所有者可核对；错误清理只影响本次；不拼旧新内容；B不能读A附件；模型实际消费有来源 |
| R13-HOST-021 | 多段/多目标投递，前段accepted尾段断线；Web查看原外部会话；错误时提供无关私聊fallback | 每段真实回执；前段不重发；unknown不换目标；其他会话0污染；origin/entry/destination保持区分 |
| R13-HOST-022 | 独立MCP服务固定账号目标，send同ID/异参数、重启、目标accepted但本地保存失败 | 同意图接收次数1；异参数冲突；真实delivery与persistence分开；恢复默认停发；不可覆盖目标/凭据 |
| R13-HOST-023 | 按COMPATIBILITY_FIXTURES的FX-AST-01先测未修改真实command/文本链/生命周期，再用明确剩余样本集覆盖旧LLM hook、多轮工具、媒体/对象代理/后台推送；逐项禁用卸载 | 来源已冻结与五阶段运行结果分开；实际下游正确；越权拒绝；后台需独立授权；残留工具/路由/任务为0；首个样本不代替未覆盖集 |
| R13-HOST-024 | 按FX-OCODE-01/FX-PI-01固定profile原样加载并调用，再按各自未覆盖集补tool/hook/summary/data/UI、取消与重启 | 每入口有映射与可核对结果；未知内部/原生UI能力明确unsupported；不注入管理员/宿主shell；不把OpenCode1.18.4样本扩大为2.2.0支持 |
| R13-HOST-025 | 旧HTTP路由账本逐方法运行；旧单用户数据指定owner导入中取消/冲突；可选旧UI真实浏览器全流程 | 支持项遵守新ACL/错误/流；未支持不返回假200；源不损坏；无匿名全局数据认领；旧UI不能越权，未交付保持缺口 |

通过门槛：成功和故障恢复断言都成立且下游一致；若只跑了其中一个子场景，该行标partial/未全覆盖并列出子结果，不得整体passed。新能力不因旧OpenClaw issue已closed而通过；此表不是竞品实测结果。

## 1.14 验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-HOST-20 | 本地OpenAI兼容端点在发现和生成分别返回401/403/429及携带秘密哨兵的响应体 | 使用协议规定的独立错误码，不读取或透传响应体；其他状态保持通用错误，旧worker仍可运行 |

## 1.15 工具调用关联透传

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-PLUGIN-HOST-09 | 核心传入指定`call_id`，worker记录它；并发请求具有各自JSON-RPC ID和通知`request_id` | worker收到的`call_id`原样一致且不串调用；宿主不把它解释为授权或幂等键 |
