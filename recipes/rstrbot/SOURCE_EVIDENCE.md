# 来源、迁移布局与证据范围

日期：2026-10-28，Asia/Taipei。项目与远端保持RstrBot，https://github.com/LIghtJUNction/RstrBot.git 。源码观察基线main提交 `ead5bf2cea735078e1a37655dcc136485f781b7f`，原工作区包含用户已确认纳入的未提交UI、SDK与文档改动。

## 2026-10-29 后续调整

用户要求按迁移技能在远程提供 generated 子模块；本文件下方“最终确定的布局”记录的是此前时点，现由独立源码仓库与 `generated/rstrbot-implementation/` gitlink 取代。源码仍不属于配方依赖或配方归档。独立源码仓库：[https://github.com/LIghtJUNction/rstrbot-implementation](https://github.com/LIghtJUNction/rstrbot-implementation)。迁移初始源码提交为 `4fe18fdb2dbd5ae1892c11e503811801bd463287`（基于原源码 `ead5bf2cea735078e1a37655dcc136485f781b7f`）；源码后续持续迭代，当前工具调用 ID 传递修复位于提交 `89355964d2c1f0561fed9188fbdc2b09b8e8c357`（父提交 `9fd0f6d304ab3f0993dbb104e3a7b4f6ad493930`）。完整迁移记录见项目 docs/MIGRATION.md。

## 当时确定的布局（历史）

RstrBot原仓库以recipes/为主要输入。既有实现、开发缓存及未提交改动完整移动到generated/rstrbot/，按用户指示作为可重新生成的本机产物由Git忽略。未建立观察源码子模块或新迁移仓库。早期额外创建的工作区已按用户要求移入系统回收站；本说明覆盖早期布局选择。

源码旧提交仍在原Git历史；移动前555个已跟踪/未忽略新增文件已与移动后逐文件核对SHA-256相同。generated/保存当前本机增量，但不进入配方提交或归档。没有把现有数据库、密钥、会话或用户数据根当作生成物移动。

## 来源身份

- 原工作区tracked diff的binary格式SHA-256：`5c15b92474b661aa898d0cd719def62c81ea25d9aac2ff621359a793b17efef8`。
- 原555文件集合指纹：`7d43724de7e22db0316e8a3d4c3562d1dd33cc6ff2771f6bc6702e1343a2d7e3`；逐项见SOURCE_HASHES.md，表内路径映射到generated/rstrbot/。
- Recipkg工具使用本机项目当前生成的Python参考CLI及当前recipes/模板；该工具源码不属于RstrBot配方依赖。初期工具源目录在并行迁移中调整为generated/（实际CLI源码为generated/src/），最终按新版指南与CLI校验。
- MIT许可保留LICENSE.md，外部依赖各自许可，未复制第三方视觉代码/资产。

## 观察与推断

| 类型 | 材料 | 支持的结论/限制 |
| --- | --- | --- |
| 直接Git/文件检查 | HEAD、origin/main、status、工具链/Cargo、LICENSE、移动前后文件摘要 | 来源及本地增量完整保留，不是运行验收 |
| 直接源码段查看 | main CLI、api router、store身份/错误类型、插件帧及超时定义；此前会话中UI组件/样式与表单 | 入口、公开类型、guard及UI增量；未逐函数全面审计 |
| 声明式接口 | OpenAPI、SDK共同向量 | 迁移快照包含65API＋2健康操作；2026-10-28本地迭代新增模型发现后为66API＋2健康操作。当前路由/DTO静态校验见docs/MIGRATION.md；示例不是运行回执 |
| 文档事实 | README、API/插件协议、认证/运维/桌面/UI/教程/技能 | 公开语义、信任边界与缺口，历史描述按CURRENT_SCOPE处理 |
| 原需求 | 基线1.4、52项requirements、215族/287验收映射 | 原目标完整保留；not_started等属于设计时点 |
| 测试定义/旧证据 | tests各HTTP/权限/恢复/浏览器入口、docs/evidence历史说明 | 提炼未来场景；本次迁移未重新跑原软件测试，不继承通过数量 |
| 作者选择 | 八包依赖分工、分阶段重建方案 | 组织合同，不是新增已实现能力 |
| 未知 | 性能硬指标、最低OS/架构矩阵、实体设备/真实服务/原生桌面 | 未验证，按ENVIRONMENT/PERFORMANCE及各包验收记录 |

用户语言选择保持Rust核心、TypeScript前端、四语言SDK，辅助工具可选；性能暂无数字预算；此前已确认中英文、双导航/移动体验、开发教程/插件技能与每次需求更新构建提示词继续有效。

## 迁移前工作区状态

以下为移动前状态，不是迁移后根仓库git status；本地增量在generated/rstrbot/保留。

```text
 M .github/workflows/ci.yml
 M README.md
 M docs/ACCEPTANCE.md
 M docs/PLUGIN_PROTOCOL.md
 M docs/blueprint/BUILD_PROMPT.md
 M docs/blueprint/SHA256SUMS
 M docs/blueprint/manifest.json
 M mise.toml
 M scripts/check.sh
 M scripts/package.py
 M scripts/verify_package.py
 M sdk/csharp/EchoPlugin/EchoPlugin.csproj
 M sdk/csharp/RstrBot.Plugin/RstrBot.Plugin.csproj
 M tests/plugin_conformance.py
 M web/src/App.tsx
 M web/src/Chat.tsx
 M web/src/Delivery.tsx
 M web/src/Resources.tsx
 M web/src/Sculpture.tsx
 M web/src/style.css
?? .agents/skills/rstrbot-development/SKILL.md
?? .agents/skills/rstrbot-plugin-development/SKILL.md
?? docs/DEVELOPER_TUTORIALS.md
?? docs/blueprint/history/BUILD_PROMPT-v1.3.md
?? web/src/BuildPromptEditor.tsx
?? web/src/DeveloperTutorials.tsx
```

## 本次验证边界

验证配方、递归依赖、归档和完整提示词；检查移动前后文件内容一致。为了保持用户先前打开的本地预览，使用新路径重启该预览并检查HTTP可访问；这不是独立重建或功能回归验收。未重跑原应用测试、未触发CI、未发送商业模型请求或外部消息。迁移验证结论见[校验记录](../../docs/MIGRATION.md)；本机临时输出已清理，最终软件是否实现仍须另行重建和验收。

## 迁移后本地迭代证据 · 2026-10-28

上述555文件及旧测试范围是迁移时点证据，不覆盖后续实现。本地叠加的配置向导已通过28项Web测试和7项临时实例Chromium场景，涵盖真实诊断插件发现、延迟HTTP夹具、取消/关闭/换目标、字段/键盘/精确价格、320px停用渠道和当前基线/浏览器草稿。独立源码布局另验证内嵌提示词快照回退。没有商业模型、外部发送、读屏或实体触摸验收。当前源码、构建产物摘要与命令记录在仓库docs/MIGRATION.md，不能沿用迁移SOURCE_HASHES推定增量源码身份。

## 随后的实际任务反馈 · 2026-10-28

独立临时SQLite与Chrome 156执行实际UI/API路径，包含初始化、诊断模型发现/聊天、资料导入原文/筛选、计划手动运行后目标会话读结果、普通成员/私聊和403、390px、离线恢复及核心重启回看。模拟角色不是访谈，诊断回复不是商业AI效果验收。账号切换后提示词草稿/历史泄露、时区选项/预览和中文网络恢复说明不足均有本地证据；相关上游issue只作独立线索，不能替代本地复现。

来源HEAD为25c36e26ab1a0abe12393bd0480a373cf801c714加本地生成增量，二进制和Web dist摘要与上节一致；具体操作、输入、截图/摘要与上游正文/评论检索范围见docs/FEEDBACK_2026-10-28.md及docs/MIGRATION.md。配方提示词1.10仅录入待修要求，未改变被测实现/内嵌1.9，不把旧通过数量、配方校验或反馈文档当作修复证明。

## 提示词1.11修复证据

在相同核心二进制上修改并重构建Web，实际浏览器覆盖账号A/B版本/导出与旧标签页、旧匿名键原样保留、存储拒绝、模拟OAuth返回、真实东京/台北预览和纽约DST序列、无任务/运行写入、迟到预览隔离、保存重开及实际计划结果、中英离线、服务端接受但确认丢失的幂等继续、晚确认保护新稿。源码/产物及脚本摘要、环境、最终34+11+7验证和限制记录在docs/MIGRATION.md及docs/REPAIR_2026-10-28.md。本地修复不代表发布/商用提供方、完整OAuth或原生设备验收；旧迁移摘要与1.10失败报告仍保留。

## 1.12目标与Pro交接

用户新增真实端到端竞品对比及OpenClaw issue反馈目标，已录入COMPETITIVE_ACCEPTANCE.md和OPENCLAW_FEEDBACK.md。首批5条上游正文/关键评论/状态已审阅，均为已关闭的历史回归线索，不是RstrBot复现或竞品当前失败证明。比较环境/版本尚未冻结，跨产品E2E尚未执行；生成实现仍为上一轮1.11，不能把配方1.12当新增功能已实现。Pro接收当前工作区快照与完整递归材料，任务状态记录于docs/PRO_HANDOFF_2026-10-28.md。

## 1.13 Pro完整配方回传与合并复核

2026-10-28由Pro基于私有Drive r2补全47个可落盘配方文件（修改30/新增17），新增202个软件验收ID，根1.13/根包.7、Web.6、core.3、其余五包.2。完整原文保留在原目录30-response.md，593620字节，SHA-256为ac80692cd8af91a107c3342a284d0d0cc27153460a7b4c0007a57291a342aab1；ZIP为266443字节，SHA-256为6cc7c616641a0696d530cd764420d83213ba32e642a0da7beba00ebc7cf823c6。原文与ZIP的50份文件逐字相等；本地核对72份输入与47项前后摘要后应用，无旧文件移除。

Pro自报材料和隔离副本应用检查，不声称运行官方Recipkg或软件；本地随后另行执行Recipkg验证，回执见项目docs/PRO_HANDOFF_2026-10-28.md。Codex复核追加现有开发/插件技能与随包材料的同步、既有授权持续性及集成ID简称；这些合并修正不冒作Pro原始字节。三份固定兼容样本的源码摘要已通过独立只读核对，仍没有兼容运行证据。新增OpenClaw两条正文/评论已复查，177956合并元数据及固定版本WhatsApp授权文档吻合；不把open问题、有修复PR或配置默认行为当作当前实机故障。

原52项、215族/287验收、当前OpenAPI/协议向量和旧历史保持原文。当前生成应用仍为1.11，1.13软件/跨产品验收全部not_executed；Pro冻结的候选release/commit只作未来执行起点，MainBot身份仍待确认。契约完备、文件可应用、软件通过、竞品领先和发布是不同结论。

## 2026-10-29 补充确认与运行观察

用户确认此前所说 MainBot 是 MaiBot（麦麦），此前1.13阶段身份待确认的描述保留为历史。AstrBot 冻结版已有局部首次对话与重启读回证据，完整共同任务和 MaiBot 实测仍未完成；具体版本、夹具、记录与限制见 COMPETITIVE_RUNBOOK。

1.16 随源码提示词快照与开发技能固定于源码子模块提交 `a626672dba59e57871cc6878c5e68bbfc009e8e3`，其父提交为先前的 `4fe18fdb2dbd5ae1892c11e503811801bd463287`；业务二进制未因本次配方更新而重建。

1.17 随包提示词与开发技能固定于源码子模块提交 `48452c4fff2db245a17203a3d214b6ea238b5dc5`，源代码行为未因本轮配方新增验收而改变。

1.18 会话模型恢复实现与随包资料固定于源码子模块提交 `1fc1d1d4a127b5b16c4f0ccd1266384e406269ba`；本机调试二进制 SHA-256 `908b8e36cef12dfd54a56e045afd6f49aae96473c01a9c524b9e1f31a06466b5`。此摘要只对应本次本机编译，不代表其他平台或运行中的旧实例已更新。

1.19 配方快照和浏览器测试入口固定于源码子模块 `9ec8022401f28967485f5c46f1e97d80526bb8f0`；没有改变产品运行逻辑。本机 Chromium 19项结果与范围见 docs/E2E_2026-10-29.md；其他RSTRBOT-25反例仍未验。

1.20 发送意图浏览器修复与随包资料位于源码子模块 `09ac632b472257603e9cca74ee46a744863ddb74`；核心调试二进制未改变。真实Chromium及本地诊断worker的失确认、刷新、手动继续、变参数、未发与存储拒绝验证见 docs/E2E_2026-10-29.md；跨设备、并发标签页及真实提供方未验。

1.21 多标签页待确认请求协调实现及随包资料位于源码子模块 `0bca19f1ea10daf3171945b78817ebbaefd0f513`；核心调试二进制未变。Chromium 双标签页和不支持锁的本机测试见 docs/E2E_2026-10-29.md；其他浏览器、跨设备及撤权竞态未完成。

## CMP-03 工具调用 ID 传递修复（2026-10-29）

源码提交 `89355964d2c1f0561fed9188fbdc2b09b8e8c357` 将模型原始工具调用 ID 传给插件 worker 的 `tools.call.params.call_id`，并由 Python、TypeScript、Rust、C# SDK 示例回显。四语言共同协议向量 24 项、真实 Python worker/历史消息/事件关联回归及 Rust 插件进程 10 项通过；完整记录、构建摘要与未验边界见 [本轮 E2E 记录](../../docs/E2E_2026-10-29.md#rstrbot-cmp-03)。
