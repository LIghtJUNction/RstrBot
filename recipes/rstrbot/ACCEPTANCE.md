# rstrbot 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-01 | 从归档生成完整提示词，在全新目录且无法访问generated/或历史源码时阅读材料 | 八个配方、完整API/协议/52需求/215族/287验收和技能都可离线获取 |
| RSTRBOT-02 | 按依赖顺序重建并运行所有组件验收 | 给出实际结果矩阵与产物摘要；任一未执行不冒作通过 |
| RSTRBOT-03 | 两用户、两工作区、同名资源、同工作区各自私聊及共享代理/知识 | 列表/读写/搜索/导出/事件均符合实时权限，管理者无隐式私聊权 |
| RSTRBOT-04 | SQLite/PG × Valkey关闭/开启，依次执行模型工具续轮、取消、撤权与恢复 | 持久语义一致，可选缓存失效不影响授权或硬预算 |
| RSTRBOT-05 | 从冷初始化到配置模型/插件、对话、知识引用、任务和显式Webhook发送 | 形成真实垂直链路，所有阶段状态与费用未知事实清楚 |
| RSTRBOT-06 | 运行排队撤权、迟到流、未知发送、进程崩溃、保存失败与重启 | 不创建替代副作用，不重复派发，保留已知回执和不确定状态 |
| RSTRBOT-07 | 四SDK真实进程、独立Webhook MCP及已安装provider/route适配器 | 同协议同错误，目标不可覆盖；非AI夹具不当商业端点证明 |
| RSTRBOT-08 | Web按320/390/768/1024/1440、双布局/主题/语言与焦点/触摸执行 | 无溢出，折叠/自动恢复正确，文案单语，未执行设备覆盖如实记录 |
| RSTRBOT-09 | 断网阅读随包教程、两个开发技能；新需求修改构建提示词 | 短教程与权威链接可用，基线/历史/差异可追溯，浏览器保存明确本地 |
| RSTRBOT-10 | Linux/macOS/Windows桌面分别安装/启动/退出/升级/卸载 | 逐平台证明，无原生证据的平台不得称完成 |
| RSTRBOT-11 | 按PERFORMANCE.md记录设备、数据量、并发、预热、重复次数、分位数和RSS | 未知预算保持未知；不能用包体积或diagnostic质量替代性能/模型质量 |
| RSTRBOT-12 | 对照REQUIREMENTS.md和TRACEABILITY.md逐项盘点 | 52/215/287保持完整，未完成项及阻塞条件仍可定位，无丢需求 |
| RSTRBOT-13 | 在全新克隆中运行`git submodule update --init --recursive`，核对.agents/skills/共享技能来源和固定提交，并检查两个RstrBot自有技能文件 | 所有Recipkg共享技能来自各自规范上游仓库且停在父仓库记录的提交；两个项目自有技能仍为普通文件，不要求额外技能仓库；配方归档不包含.git子模块元数据 |
| RSTRBOT-14 | 在模型向导配置OpenAI兼容端点，发现列表后选模型并保存；再用不支持发现的夹具手动输入 | 选项来自真实`model.list`响应，手动输入仍可用；发现不执行生成或预留RstrBot模型预算，临时密钥不保存，地址改变时不借用已存凭据；权限撤销时取消后续插件调用 |
| RSTRBOT-15 | 模型发现慢响应时改目标、取消或关闭重开；模型和渠道输入错误地址/凭据/数值，使用Enter前进 | 旧响应不进入新表单；取消明确只停止等待；错误在所属步骤定位并关联控件，核对前不保存，无重复请求 |
| RSTRBOT-16 | 同一浏览器A保存提示词/版本→退出→B登录→恢复/导出，再A重登；跨实例、旧全局键和旧标签页 | B看不到A正文或历史，A原版本仍在；身份变化及迟到回调不串号，旧键不自动认领或静默删毁；按TEST_FEEDBACK.md迁移取舍验收 |
| RSTRBOT-17 | 搜索Tokyo、选每日09:00并预览，换时区/频率后放行旧响应；保存重开并运行 | IANA/频率可选且支持自定义，真实只读预览标计划/设备时区和暂停；09:00东京对应08:00台北；无模型/派发副作用，旧预览不回填 |
| RSTRBOT-18 | 中文/英文离线发送→恢复连接→核对后手动继续；服务端已接受但客户端失联，确认晚到期间写新稿 | 当前语言说明准确且正文保留；不自动重放、不将未知说成未发；一次用户意图不重复，新稿不被旧确认清空；失败/未知不得冒作通过 |
| RSTRBOT-19 | 按COMPETITIVE_ACCEPTANCE冻结本项目/AstrBot/用户确认的MaiBot/OpenClaw的版本、环境和共同任务，逐项执行/读回真实结果 | 逐产品记录passed/failed/blocked/not_executed与证据；未实现能力计缺口，无法运行不算对方失败；没有同条件完整证据不得宣称全面超越 |
| RSTRBOT-20 | 审阅OpenClaw issue正文、状态及关键评论/修复，映射本项目配方与故障验收，重建后复测 | 来源/版本/日期/适用性/优先级/最小变化/验收/本地状态齐全；关闭不等于仍有缺陷，外部报告不等于本项目复现；保留反例与历史 |

## 1.13 追加的完整能力与一致性验收

[CAPABILITY_COVERAGE.md](CAPABILITY_COVERAGE.md) 为全部52项提供当前事实、责任位置和成功/失败/取消/重启/隔离撤权/预算副作用的完整断言；[ACCEPTANCE_HARNESS.md](ACCEPTANCE_HARNESS.md) 规定实际夹具、故障窗口与证据记录。下表是强制验收索引，不替代引用原文。初始全部not_executed；材料校验只对实际运行的材料项目更新，不能给运行场景打通过。原215族/287条原验收继续运行，见TRACEABILITY_INDEX和未改的TRACEABILITY。

| 新验收ID | 完整合同/场景 | 初始软件状态 |
| --- | --- | --- |
| R13-RB-001 | CAPABILITY_COVERAGE.md的RB-001六边界场景 | not_executed |
| R13-RB-002 | CAPABILITY_COVERAGE.md的RB-002六边界场景 | not_executed |
| R13-RB-003 | CAPABILITY_COVERAGE.md的RB-003六边界场景 | not_executed |
| R13-RB-004 | CAPABILITY_COVERAGE.md的RB-004六边界场景 | not_executed |
| R13-RB-005 | CAPABILITY_COVERAGE.md的RB-005六边界场景 | not_executed |
| R13-RB-006 | CAPABILITY_COVERAGE.md的RB-006六边界场景 | not_executed |
| R13-RB-007 | CAPABILITY_COVERAGE.md的RB-007六边界场景 | not_executed |
| R13-RB-008 | CAPABILITY_COVERAGE.md的RB-008六边界场景 | not_executed |
| R13-RB-009 | CAPABILITY_COVERAGE.md的RB-009六边界场景 | not_executed |
| R13-RB-010 | CAPABILITY_COVERAGE.md的RB-010六边界场景 | not_executed |
| R13-RB-011 | CAPABILITY_COVERAGE.md的RB-011六边界场景 | not_executed |
| R13-RB-012 | CAPABILITY_COVERAGE.md的RB-012六边界场景 | not_executed |
| R13-RB-013 | CAPABILITY_COVERAGE.md的RB-013六边界场景 | not_executed |
| R13-RB-014 | CAPABILITY_COVERAGE.md的RB-014六边界场景 | not_executed |
| R13-RB-015 | CAPABILITY_COVERAGE.md的RB-015六边界场景 | not_executed |
| R13-RB-016 | CAPABILITY_COVERAGE.md的RB-016六边界场景 | not_executed |
| R13-RB-017 | CAPABILITY_COVERAGE.md的RB-017六边界场景 | not_executed |
| R13-RB-018 | CAPABILITY_COVERAGE.md的RB-018六边界场景 | not_executed |
| R13-RB-019 | CAPABILITY_COVERAGE.md的RB-019六边界场景 | not_executed |
| R13-RB-020 | CAPABILITY_COVERAGE.md的RB-020六边界场景 | not_executed |
| R13-RB-021 | CAPABILITY_COVERAGE.md的RB-021六边界场景 | not_executed |
| R13-RB-022 | CAPABILITY_COVERAGE.md的RB-022六边界场景 | not_executed |
| R13-RB-023 | CAPABILITY_COVERAGE.md的RB-023六边界场景 | not_executed |
| R13-RB-024 | CAPABILITY_COVERAGE.md的RB-024六边界场景 | not_executed |
| R13-RB-025 | CAPABILITY_COVERAGE.md的RB-025六边界场景 | not_executed |
| R13-RB-026 | CAPABILITY_COVERAGE.md的RB-026六边界场景 | not_executed |
| R13-RB-027 | CAPABILITY_COVERAGE.md的RB-027六边界场景 | not_executed |
| R13-RB-028 | CAPABILITY_COVERAGE.md的RB-028六边界场景 | not_executed |
| R13-RB-029 | CAPABILITY_COVERAGE.md的RB-029六边界场景 | not_executed |
| R13-RB-030 | CAPABILITY_COVERAGE.md的RB-030六边界场景 | not_executed |
| R13-RB-031 | CAPABILITY_COVERAGE.md的RB-031六边界场景 | not_executed |
| R13-RB-032 | CAPABILITY_COVERAGE.md的RB-032六边界场景 | not_executed |
| R13-RB-033 | CAPABILITY_COVERAGE.md的RB-033六边界场景 | not_executed |
| R13-RB-034 | CAPABILITY_COVERAGE.md的RB-034六边界场景 | not_executed |
| R13-RB-035 | CAPABILITY_COVERAGE.md的RB-035六边界场景 | not_executed |
| R13-RB-036 | CAPABILITY_COVERAGE.md的RB-036六边界场景 | not_executed |
| R13-RB-037 | CAPABILITY_COVERAGE.md的RB-037六边界场景 | not_executed |
| R13-RB-038 | CAPABILITY_COVERAGE.md的RB-038六边界场景 | not_executed |
| R13-RB-039 | CAPABILITY_COVERAGE.md的RB-039六边界场景 | not_executed |
| R13-RB-040 | CAPABILITY_COVERAGE.md的RB-040六边界场景 | not_executed |
| R13-RB-041 | CAPABILITY_COVERAGE.md的RB-041六边界场景 | not_executed |
| R13-RB-042 | CAPABILITY_COVERAGE.md的RB-042六边界场景 | not_executed |
| R13-RB-043 | CAPABILITY_COVERAGE.md的RB-043六边界场景 | not_executed |
| R13-RB-044 | CAPABILITY_COVERAGE.md的RB-044六边界场景 | not_executed |
| R13-RB-045 | CAPABILITY_COVERAGE.md的RB-045六边界场景 | not_executed |
| R13-RB-046 | CAPABILITY_COVERAGE.md的RB-046六边界场景 | not_executed |
| R13-RB-047 | CAPABILITY_COVERAGE.md的RB-047六边界场景 | not_executed |
| R13-RB-048 | CAPABILITY_COVERAGE.md的RB-048六边界场景 | not_executed |
| R13-RB-049 | CAPABILITY_COVERAGE.md的RB-049六边界场景 | not_executed |
| R13-RB-050 | CAPABILITY_COVERAGE.md的RB-050六边界场景 | not_executed |
| R13-RB-051 | CAPABILITY_COVERAGE.md的RB-051六边界场景 | not_executed |
| R13-RB-052 | CAPABILITY_COVERAGE.md的RB-052六边界场景 | not_executed |
| R13-INT-001 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-002 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-003 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-004 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-005 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-006 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-007 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |
| R13-INT-008 | CONTRACT_INTEGRATION.md的同ID一致性场景；材料/运行分别报告 | not_executed |

竞品追加：R13-CMP-01至R13-CMP-12、R13-CMP-GATE 的完整执行卡见 [COMPETITIVE_RUNBOOK.md](COMPETITIVE_RUNBOOK.md)；R13-OC-42640/59301/69160/91420/84544/177909/186700 的自足场景见 [OPENCLAW_FEEDBACK.md](OPENCLAW_FEEDBACK.md)。所有跨产品和新增上游回归软件结果初始not_executed。叶子及组件新验收随各包ACCEPTANCE继续执行，不由根表替代。

## 1.15 生成实现布局验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-21 | 全新克隆主仓库并递归初始化子模块；核对generated下gitlink、URL、精确提交和源码内容；无generated目录时独立检查配方归档 | 远程真实记录模式160000和可读的独立源码仓库，保留已确认本地改动；源码/构建缓存/数据库/密钥不进入纯Markdown配方归档；保留正在运行实例的本机兼容路径，禁止把别名提交为第二份实现 |

## 1.16 对话标题与隐式模型调用验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-22 | 新会话接入记录出站次数的本地模型夹具，发送一次并读回；刷新、重启后再读回；测试标题为空/过长及生成失败 | 默认唯一生成请求对应用户发送；本地标题不造成第二次调用，读回不产生调用；若未来显式启用模型标题，先展示请求、费用和数据范围，记录额外调用与未知费用，标题失败不丢对话；未实现可选能力保持未实现而不伪称通过 |

## 1.17 已接纳消息与供应商回退验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-23 / R13-OC-185423 | 按 OPENCLAW_FEEDBACK 执行两个供应商独立账本、一次明确401、同消息合法复用，以及变正文/主体/权限、本地存储冲突、unknown、预算不足和重启反例 | 在功能实际提供且策略启用时，唯一合法用户消息不被尝试字段误判冲突，确知拒绝后备用恰调用一次并读回；真实冲突与unknown不触发备用，失败/取消/重启不擅自重放。没有功能和运行证据时记 gap/not_executed |

## 1.18 会话模型选择验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-24 | 在 Alpha 发一轮；添加 Beta，导航回原会话、刷新/重启后继续；再显式选 Beta，切换多个会话；测试旧会话、模型删除/撤权、私有模型用于共享会话、迟到读取和重复提交 | 原会话始终恢复可用的最近已接纳模型，只有显式切换后的发送使用 Beta；两端账本与模型选择一致。无权、删除、旧数据缺绑定或读取失败时待用户明确选择，不自动降级；服务端原子写绑定，幂等重放不覆盖新选择；其他会话、其他用户和共享可见性不泄露私有模型身份；服务端拒绝共享会话使用私有模型，拒绝前不创建消息或运行 |
