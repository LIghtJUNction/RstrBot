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

## 1.19 长草稿重排

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-25 / R13-OC-153139 | 多行草稿在窄屏/横屏及面板改变宽高后返回；测试首、中、末插入点与选区、手动滚动、失焦、输入法、会话切换、阅读锚点与软键盘 | 文本、焦点、选区按原有语义保留；编辑位置原本可见时重排后仍可见；主动滚动查看旧内容不被强制拉到底；无发送或跨会话串稿。真实未执行设备和浏览器保持未验 |

## 1.20 未确认发送跨刷新

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-26 | 拦截已接纳响应、刷新后重开原会话并继续；未接纳对照；变更内容/模型/代理、不同会话/账号、存储失败、迟到确认及重启 | 自动重连不派发；原请求相同身份/参数/键手动继续至多一条用户消息和一次模型运行；不同请求经明确确认才能开新意图，未确认状态可见；新草稿和跨账号隔离保留，权限/错误/未知状态不冒成功；存储不可用时说明限制 |

## 1.21 多标签页待确认请求

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-27 | A/B共同会话；B打开旧确认，A创建新待确认后B确认；同/不同请求并发、迟到回执、重载、权限/账号隔离、协调/存储失败 | 旧确认不能删除或替换新请求，状态变化可见；同意图至多接纳一次，不同意图须显式确认；不支持协调或存储时不发送且提示，不自动重放；设备间与未测场景单列 |

## 1.22 知识引用原文

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-28 | 导入含唯一标记的文档，发起可检索聊天，核对模型实际输入及持久记录；删除文档后读取检索、对话、运行；对旧格式含引用原文的历史、事件进行读取 | 当前获准上下文能含原文；新对话/运行持久引用保留可追溯定位但不含原文；历史/运行/事件响应不返回旧引用原文；删除后搜索不再返回文档。模型回答、旧存储及备份、在途撤权竞态单列且不得声称清除或验证 |

## 1.23 回答中的来源核对

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-29 | 导入含中文与特殊字符的来源并对话；打开资料；编辑、停用、删除、撤权或断网后再打开；切换会话/账号/工作区及迟到响应；测试窄屏/键盘 | 明确候选非已引用；当前授权且来源摘要、索引版本、分块标识及字节范围匹配时，显示原文并准确标出片段；其他情况不显示旧或新原文冒充旧版本，说明状态并允许恢复；不存在 HTML 执行或跨身份残留 |

| RSTRBOT-30 / OC-157827 | 合并同一文件非连续段落及不同文档为候选摘要/记忆；撤销其中一条来源、旧记录缺映射及取消/重启 | 每条派生片段对应全部真实来源版本与位置；不可核对时不发布或提升，不猜测旧证据归属；可按每条来源回读；自动派生未实现时记 not_executed |

## 1.24 索引代次与读取视图

| 编号 | 操作 | 预期 |
| --- | --- | --- |
| RSTRBOT-31 / OC-16844 | 长时间保持检索连接；SQLite WAL 多连接或外部索引进程先后更新/删除来源并发布新索引，在发布前后并发查询并重启；对每个索引后端和 PostgreSQL/Valkey 分开执行 | 已发布代次的来源、命中及位置映射一致，无旧快照造成静默丢失；未发布显示处理中，失配或定位失败明确降级/失败；已删内容不在后续结果中，运行中与未执行后端单列 |

| RSTRBOT-32 | 注入已启用文档块与原文或索引元数据不一致；搜索、对话、普通用户访问他人私有坏文档；修正后重试 | 搜索明确标示索引不一致，聊天在调用模型前失败且用户得到单语言修复提示和受众有权查看的待修资料名称；修正保存后检索与模型调用恢复；无权、停用、归档文档不得影响其他人的正常结果 |

| RSTRBOT-33 | 本地夹具让一次性计划成功、失败、取消；重复计划发生失败；运行中强制终止并多次重启 | 一次性终态与真实运行一致，周期计划保留未来调度且显示最近结果；UI以当前语言标识失败/待核对、展示可访问的运行详情，不自动重试；无权访问仍由服务端拒绝；原运行/计划与模型请求计数可核对 |

| RSTRBOT-34 | 实际执行计划时终止核心，恢复后依次尝试其他成员、撤权、旧版本、活跃运行和本人确认；核对事件/原运行/请求计数；对周期及过去的一次性计划重新启用 | 只有有权所有者可确认已核对，已知活动运行不可确认；CAS 和原 claim 绑定，持久记录确认而不编造旧结果；确认后保持暂停、无模型/投递/费用变化；单独启用只安排未来时刻，过去的一次性任务必须修改时间，重复操作和重启不重放旧请求 |

| RSTRBOT-35 | 浏览器中让当前模型读取延迟，重选同一个工作区，再放行读取、输入并发送；另切到不同工作区，放行旧请求 | 同工作区保留进行中的授权读取和草稿，发送可完成且恰有一个下游运行；不同工作区继续隔离，离开前旧意图和迟到响应不发起/污染其他工作区 |

| RSTRBOT-36 / OC-181952 | 对会话分别构造已完成占用、缺终态但无活动执行的遗留占用、仍有活动执行、待投递及不明证据；重启后继续同一会话 | 仅释放可证明无活动所有者且无未决副作用的孤儿占用，保留原历史/模型选择；仍有活动或不明时保留占用并可核对；新请求只由用户主动发起，旧消息和外部动作不自动重放；逐种实际运行与账本验证，未运行保持 not_executed |

## RSTRBOT-37 · 投递恢复与持久意图

独立真实 HTTP 接收端记录收到的消息并丢弃确认；未知回执在勾选核对后可创建新草稿，创建草稿不触发请求；重试原编号不重复接收。关闭或重载标签页、同账户另一标签页恢复原意图；跨用户、工作区、渠道隔离。存储写入失败、数据损坏、协调锁不可用、旧状态冲突时保留草稿且请求数不增加；旧页面不得覆盖已变更意图。以服务端、浏览器与接收端账本分别记录事实；不把清除站点数据或跨设备称为已保障。

## RSTRBOT-38 · 交互和计划工具目录完整性

在隔离实例中创建可用只读工具和100个较新的停用插件，分别从交互消息和实际触发的一次性计划调用；两条路径各产生真实工具结果，不能只核对目录或模型自报。验证私有实例与工作区权限不会扩大；超过工具/扫描上限需给出当前语言明确原因和可操作的修复建议，交互和运行记录一致，且不发起模型生成或业务工具调用（只读目录发现允许），边界值不得误报。成员撤销、页间并发修改和大量资源的结果需单列验证，未测试不得声称通过。

## RSTRBOT-39 · 文本附件端到端

实际浏览器选择UTF-8 txt/md文件，与用户正文一起发送；独立真实HTTP模型夹具核对正文、名称及每个文件的完整原文；历史读取并下载原字节、重启后摘要一致。验证旧核心不支持专用入口时失败且零模型请求、用户/工作区权限及共享会话告知、非法编码与超限拒绝、同key重试/变参冲突、断线恢复无自动重发和会话切换草稿隔离。当时其他格式明确不可用；1.32 中图片以 RSTRBOT-41 为准，完整媒体 CMP-09 仍 not_executed。

## RSTRBOT-40 · 未知聊天字段不会静默丢失

对普通和文本附件聊天接口分别发送带图片字段、拼错字段及未知附件字段的请求，确认明确拒绝，消息和运行均不增加且模型接收端零请求；同样的已声明文本和附件请求仍能完成。核对OpenAPI与实际拒绝行为一致。旧版兼容性变化和图片支持状态必须说明；测试不能代替真实图片、视频或完整媒体验收。


## RSTRBOT-41 · 图文输入端到端

实际浏览器选择 PNG/JPEG 与正文和文本附件一起发送；独立 OpenAI 兼容 HTTP 接收端解码图像并比对原始 SHA-256，同时核对正文及文件文字；预览与下载原字节、重启和私有/共享权限。测试格式伪装、损坏、动画、尺寸/数量/体积边界、危险文件名、未知模型能力、非兼容提供方、预算并发与图片调用期间崩溃恢复、模型切换、旧核心专用入口不存在、同键重试及改变字节冲突；拒绝时无模型请求，未知结果不自动重发。记录夹具、实际模型和 CMP-09 未验收范围。


## RSTRBOT-42 · 未知费用核对、更正与审计（本地SQLite/API/Chromium通过；PostgreSQL和商业账单未执行）

在隔离实例用本地模型夹具制造无用量/单价回执的图片调用，读回unknown reservation；owner/admin从预算页找到同一调用，选择来源类别、输入金额、勾选声明、复核后登记。核对budget的spent/reserved/unknown数量和reservation版本、金额、来源、管理员与时间。以两个管理员并发提交相同和不同值、丢失确认后用原请求重试、陈旧revision及金额更正验证CAS、幂等和差额；更正历史不可覆盖。

验证普通成员/只读者、其他工作区和撤权后访问被拒；reservation总数、首末页与续页完整可达，登记历史用独占事件ID逐页读取；reserved在途请求、负数/非整数/超界/未知来源/缺确认/额外字段均在记账前失败；不同管理员并发提交不同值恰有一个版本成功，预算设置竞态串行化，已发生费用即使超限仍照实入账。普通成员的事件流不含单笔金额/来源，管理员历史按ACL读取。备份并恢复到隔离空库后历史和汇总一致，重启不重复结算。浏览器实际操作中英文、键盘和320px；连接结果未知时保留输入，不自动重复提交。供应商页面、商业计费和未实测数据库/平台单列未验证。2026-10-29本地复验：Rust 33项通过；独立SQLite/OpenAI兼容夹具10项通过；Chromium 156的3组检查通过（中英、Enter复核、320px）；reservation offset分页、历史事件ID续页、同值幂等、异值管理员竞态、预算串行与备份恢复通过。证据及边界见 generated/rstrbot-implementation/docs/MIGRATION.md。

## RSTRBOT-43 · 插件工具调用 ID 可追溯

用固定模型工具调用ID触发真实独立插件进程，逐字比较插件实际收到的`call_id`、持久化工具消息的`tool_call_id`和工具事件ID；重启后再次读取历史。JSON-RPC请求ID与通知`request_id`使用其他值并保持独立。验收不得把关联字段当作授权或幂等依据；未测的WebUI自定义包安装/调用链继续单列，不扩大为CMP-03整体通过。
