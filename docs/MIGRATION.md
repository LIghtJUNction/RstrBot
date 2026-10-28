# RstrBot 配方迁移校验 · 2026-10-28

项目与远端保持 **RstrBot / LIghtJUNction/RstrBot**。现有仓库以 recipes/ 为主要输入，既有实现及未提交改动完整移到受Git忽略的 generated/rstrbot/。用户撤销的额外迁移工作区已移入系统回收站，不存在同名后缀项目路径。没有创建新的远端仓库。

## 2026-10-29 后续布局

用户要求远程包含 generated 源码子模块；旧版“Git忽略”的描述仅为本记录创建时的历史事实，现已由独立源码仓库和 `generated/rstrbot-implementation/` gitlink 取代。配方仍不依赖源码或把它打入归档。源码仓库为 [rstrbot-implementation](https://github.com/LIghtJUNction/rstrbot-implementation)，固定提交 `4fe18fdb2dbd5ae1892c11e503811801bd463287`，父提交为原基线 `ead5bf2cea735078e1a37655dcc136485f781b7f`。运行中的旧路径由本机未跟踪的兼容链接维持，递归克隆不包含该链接。

### 2026-10-29 子模块验证

从主仓库本地提交 `053d1836ae0a09b69097379950fd6f70f77010e2` 新建临时克隆，执行 `git submodule update --init --recursive`：11 个共享技能及生成源码共 12 个子模块均成功检出；源码 gitlink 为模式 `160000`，实际检出 `4fe18fdb2dbd5ae1892c11e503811801bd463287`，工作区干净。临时移开源码目录后，根配方检查、递归打包、归档检查和中文构建提示词生成均通过；归档 91 个普通文件均为 Markdown，不包含源码或 Git 元数据。测试克隆是隔离环境，不代表其他平台软件验收。

本机原路径 `generated/rstrbot` 为未提交的临时兼容链接，供迁移前已运行的本机实例继续读取资源；干净克隆不含此链接。旧实例停止后可移除该链接。迁移期间该实例的 HTTP 页面仍返回 200，不代表其使用了最新构建。

## 实际完成的检查

| 检查 | 本次结果 | 范围 |
| --- | --- | --- |
| 八个配方各自recipkg check | 全部退出0 | 当前Recipkg生成的Python CLI，离线递归校验 |
| 递归pack、归档check、中文build提示词 | 全部退出0 | 包版本0.2.0-recipe.1，完整八包上下文 |
| 归档对象/路径/编码 | 66个普通UTF-8 .md，无NUL、文件系统链接、源码/二进制/钩子 | 独立读取tar成员检查 |
| LOCK完整性 | 所有65个负载文件摘要匹配，锁清单无缺项 | 八个包身份齐全 |
| 完整提示词内容 | 65份负载Markdown原文全部包含 | pack改写后的manifest一并包含；LOCK为完整性元数据 |
| 需求保留 | 52项需求、215族（含R和B前缀）、287条原验收身份与顺序一致 | 52目标/验收逐文本核对；原始追踪和快照完整保留 |
| HTTP声明 | 原OpenAPI JSON完整对象相等 | 65API操作＋2健康操作，所有DTO/示例/约束 |
| 本地文档链接 | 配方内相对链接全部存在 | 外部链接是来源引用，不作为缺失依赖 |
| 原实现保留 | 555个来源文件移动前后SHA-256相同 | 原工作区快照7d43724de7e22db0316e8a3d4c3562d1dd33cc6ff2771f6bc6702e1343a2d7e3 |
| 本地预览 | 新源码路径启动后HTTP200 | 127.0.0.1:8788，业务数据根没有移动；不是功能验收 |
| GitHub发现标签 | 已添加并通过REST回读recipkg | 仓库名称和原远端未改 |

## 临时校验文件（已清理）

- 迁移检查期间生成过配方归档（333298字节，SHA-256 `d83b155f539a7356044dc643cbdb5457a7d34bd9581c28a15e559ffc788a8b44`）和完整build提示词（1391671字节，SHA-256 `733cefa3a21638aeaf1ec63fe9cc07c851b431020ad1aa6ef2822e7b32b4b7e1`）。
- 另有中文观察清单、配方检查与归档检查JSON、校验摘要及迁移前源码状态快照。
- 这些都是本机临时文件，已按用户要求移入系统回收站；当前仓库不保留固定产物目录。上面的检查结论记录在本文件中，配方内容仍可重新打包。

配方本身位于recipes/，所有递归输入可独立打包。重新执行pack/build时，把输出写到仓库外的系统临时目录。

## 过程中的失败与修正

早期检查引用的Recipkg源码路径在该项目并行调整布局后失效，返回No module named recipkg；最终定位到它的generated/src/，重新执行全部八包及归档检查通过。第一次需求计数只识别R前缀漏算26个B前缀；最终改为与原始215个ID集合及顺序逐一比对，通过。GitHub GraphQL标签回读曾EOF，后通过REST确认，没有重复修改标签。

## 未执行与边界

本次未独立重建RstrBot，未重新运行原软件测试、未测性能、未验证原生安装/签名/升级、读屏/实体设备或商业模型/真实邮件/聊天平台。未触发CI。当前源版本是 `ead5bf2cea735078e1a37655dcc136485f781b7f` 加迁移前本地增量；旧已提交源码留在Git历史，本地增量连同全部原实现保存在generated/rstrbot/。性能数字保持未知，未来实现按配方验收。

来源、范围、推断与未完成蓝图见recipes/rstrbot/SOURCE_EVIDENCE.md和CURRENT_SCOPE.md。打包通过证明材料可交付，不证明未来重建已经成功。

## 共享开发技能子模块后续 · 2026-10-28

将Recipkg维护的11个规范共享技能按一仓库一个子模块接入`.agents/skills/`，各自固定在RstrBot父仓库记录的提交；具体仓库和初始化/更新方法见[技能目录说明](../.agents/skills/README.md)。两个RstrBot自有技能仍是已跟踪的普通文件（Git模式100644），没有为它们创建新仓库。迁移技能上游的边界说明已在其独立仓库更新到`cd061fd`。

本次执行`recipkg check recipes/rstrbot`、打包、解包后再次check，以及中文build提示词生成，均退出0。归档复核为66个普通文件，仅含UTF-8 Markdown和LOCK，没有Git子模块元数据。输出放在系统临时目录并自动清理。未运行RstrBot软件测试、未触发CI；这些检查只证明配方和技能目录结构可交付，不证明软件重建或运行验收完成。

## 本地实现迭代：模型 ID 发现 · 2026-10-28

在忽略跟踪的`generated/rstrbot/`接通`POST /api/v1/workspaces/{wid}/models/discover`与模型设置向导。调用已安装、声明model能力且实现`model.list`的可信插件，只返回最多500个去重后的ID/名称；向导保留手动输入。临时密钥仅用于该请求，重用已保存密钥要求同工作区模型、provider、完整地址和私网授权均匹配。请求期间复核会话、管理员权限、恢复状态、私网策略及资源版本。发现不执行生成、不预留RstrBot预算，供应商仍可能记录或计费。

配方同步到`rstrbot`、`rstrbot-core`和`rstrbot-web`的`0.2.0-recipe.2`，更新构建提示词、验收、HTTP/OpenAPI、教程、来源范围和变更记录。输出归档及完整中文build提示词均放在系统临时目录，没有创建仓库内产物目录。

| 检查 | 结果 | 范围 |
| --- | --- | --- |
| `cargo fmt --check` | 通过 | Rust格式 |
| `cargo check --workspace --locked` | 通过 | Rust workspace编译；未执行测试目标 |
| `npm --prefix web run build` | 通过 | TypeScript检查与Vite生产构建 |
| `scripts/validate_openapi.py` | 通过 | 66个API操作、2个健康操作，路由覆盖、DTO、引用与示例静态校验 |
| Recipkg根包/核心包check、递归pack、解包后check、中文build | 全部退出0 | 归档和生成提示词包含model.list要求与新增契约 |

未运行RstrBot测试套件、真实供应商请求或浏览器验收，也未触发CI；因此编译和契约检查不表示外部端点或实际界面行为已验收。

## 本地实现迭代：配置向导、取消与基线同步 · 2026-10-28

本轮修改忽略跟踪的`generated/rstrbot/`：模型发现按当前请求身份隔离，换provider/地址/密钥/私网授权、取消等待和卸载编辑器均中止浏览器请求并拒绝迟到成功/失败；取消不宣称提供方撤回处理。模型/渠道的逐步校验与最终保存共用配置准备器，覆盖UTF-8长度、完整URL、凭据目标绑定、token整数及精确微美元。错误关联label、控件与说明并聚焦，Enter在核对前仅前进，保存期间禁止重复提交与关闭。服务端授权仍独立执行。

根提示词升级1.9，1.8原文保存在`recipes/rstrbot/history/BUILD_PROMPT-v1.8.md`；根包与Web包升至`0.2.0-recipe.3`，核心接口不变，核心包保持`0.2.0-recipe.2`。更新相应提示词、验收、工作流、范围/来源和变更记录。Web的predev/prebuild从当前根配方同步`docs/RECIPE_BUILD_PROMPT.md`内嵌快照；独立源码布局使用随包快照，旧1.4文档与浏览器草稿不覆盖。帮助/教程链接指向当前真实配方路径，已本地核对10个文档/技能目标存在。

最终复核时间为2026-10-28 10:28 UTC附近；环境Linux x64、Node v24.22.1、Rust 1.99.0、Chromium 156.0.8078.47、临时SQLite实例、无Valkey。Rust版本是本次实际编译环境，不改写配方中记录的来源工具链版本。

| 检查 | 最终结果 | 实际覆盖 |
| --- | --- | --- |
| `npm --prefix web test` | 28通过，0失败/跳过 | 新增6项配置边界检查，连同既有Web测试；不代表整个Rust/集成测试套件 |
| `npm --prefix web run build` | 通过 | 自动同步根基线、TypeScript检查、Vite生产构建 |
| `cargo build --workspace --bins --locked` | 通过 | 实际核心和worker二进制；未运行完整Rust测试 |
| `node tests/browser/verify_configuration.mjs --chromium /usr/bin/chromium` | 7场景通过 | 字段关联/错误焦点；真实诊断插件发现；改目标取消与迟到响应隔离；取消/关闭重开；Enter逐步提交/单次保存/精确价格；320px停用Webhook；当前内嵌基线与浏览器草稿保留 |
| 独立临时源码布局执行同步脚本 | 通过 | 找不到根配方时保持随包快照字节不变 |
| Recipkg根/Web/core check、pack、unpack后check、中文build | 全部通过 | 根/Web .3、core .2；生成提示词包含WEB-17/18/19与1.9需求 |
| 归档逐项检查 | 通过 | 67个普通UTF-8 Markdown/锁文档，含1.8历史，无源码、Git元数据、链接或执行钩子 |
| `git diff --check` | 通过 | 当前已跟踪配方与文档修改 |

开发时初次同步脚本路径多上移一级、归档自查初次误用LOCK而非LOCK.md，均已修正并重新通过；表格记录最终状态。验证归档与中文提示词只放系统临时目录并已自动清理。浏览器脚本同样使用系统临时目录并自动清理，没有创建固定产物目录。

对应源码与产物SHA-256：

| 文件/集合 | SHA-256 |
| --- | --- |
| `web/src/Resources.tsx` | `0c51101565410131503450ef611a0b610df4adc9c5d2cbe5fb51dea24846019f` |
| `web/src/resourceConfiguration.ts` | `0462b8350d79ade96347399a0743a2f320bc6b91521bd172e544e5fb611c2feb` |
| `web/src/ui.tsx` | `5e33690672f6dedb0b709cb4cafc0d9bc32c9e9af23b0d466b07d6e8b27592ba` |
| `web/src/BuildPromptEditor.tsx` | `707a3c0f56840372621f27c5890aba7149ed58eb3a52150caf7db613cf2b69b1` |
| `web/src/DeveloperTutorials.tsx` | `b79991bfbc514344d231086850a51724f53613bf3113d9a1462452a5400f75f2` |
| `web/scripts/sync-recipe-prompt.mjs` | `8a783f364401e2daf7e4142523ff358fe2a3268eb6735bc25b302b729a306220` |
| `docs/RECIPE_BUILD_PROMPT.md` | `c8dd329dcc23e5562a2eed8c1c603596bf5d6224e7a485eb04b75350056287ac` |
| `target/debug/rstrbot` | `5fdd9950ae2e056e4874ed86de1f9988077864df5cfcee4e17c1a9cc58fcbbd4` |
| `tests/browser/verify_configuration.mjs` | `f40551d7b08cbb8d280f6b63a1b8512d28d1989ce8beb5746bc12aed18a7c956` |
| `web/dist`文件树 | `edf608f8bc6c7eaa31c9b10725a8605cf1644a11f3e365dbf53a35686fdada4f` |
| 临时Recipkg归档 | `10c7efa954f598cc0d2cfb94c6d2c8b1082ef71c1f02b617f0b20576563a28ab` |
| 临时完整中文提示词 | `297c711305290966a109d84e1edfa2f3b380fffa282f176a5cd7158df88475c3` |

文件树指纹按相对路径排序，依次累计UTF-8路径、零字节和该文件SHA-256二进制摘要。源码/二进制路径均相对于generated/rstrbot/，临时归档已删除。

浏览器正常发现路径通过真实核心/诊断worker，延迟、取消及迟到失败使用受控HTTP响应夹具。仅验证配置保存，未向外部Webhook发送消息。商业提供方、完整浏览器/读屏/实体触摸/原生桌面矩阵、完整后端回归和CI本轮未执行；不把7项配置回归当全产品验收。

## 实际任务测试反馈与需求入配方 · 2026-10-28

按test-feedback先操作后解释，使用独立临时SQLite、同源55005实例、Chrome 156.0.8078.47和合成admin/member账号，约10:35—11:07 UTC执行。实际路径、初始状态、输入/结果、截图证据、上游issue正文/评论的检索范围及局限见[本轮反馈](FEEDBACK_2026-10-28.md)。角色为Agent模拟，不是人类访谈。HEAD为25c36e26ab1a0abe12393bd0480a373cf801c714加已存在的本地生成增量；实际核心和dist摘要与上一节完全一致。

| 实际任务 | 本轮结果/限制 |
| --- | --- |
| 新实例初始化→真实诊断插件发现/模型保存→对话→刷新/重开 | 完成，确定性非AI夹具回复；无商业模型效果结论 |
| Markdown导入→正文编号筛选→读原文→核心重启后重开 | 完成，MOSS-2048正文和索引版本保留；不是语义检索质量验收 |
| 每日计划→错误时区纠正→暂停保存→手动运行→目标会话读结果 | 功能完成；Tokyo输入需自行找到IANA，UI缺时区/频率选项和预览（RFB-002，P2）。只读preview补充调用200，现有核心已提供5次UTC时间及Asia/Tokyo |
| 普通成员390px→共享模型私聊→模型权限→换管理员 | 私聊双方隔离、无页面横向溢出、直接模型写入403；仅所测页面/角色组合 |
| 成员保存提示词/历史→导出→退出→管理员登录 | **失败**：管理员可直接读成员正文/历史（RFB-001，P1）；局限在本机提示词编辑器，未发现私聊/API跨账号越权 |
| 中文离线发送→联网手动继续→核对消息 | 正文保留、一次手动发送得到一组回复；主要错误仅Failed to fetch、恢复指引不足（RFB-003，P2），未复现丢稿/重复 |

有界查询本项目issue为0条；补充只读审阅Open WebUI #23332/#29760和AstrBot #10239的正文及全部评论，保留版本/状态/修复结论边界。其生成中刷新、排队与后台失败路径不自动当作RstrBot已复现缺陷。没有向上游发帖/改状态或外部真实发送。

本轮按所选技能默认交付可评审要求，未实现三项修复。根提示词1.10，根/Web包0.2.0-recipe.4，核心合同/版本保持0.2.0-recipe.2；旧1.9原文保留在history/，新增TEST_FEEDBACK.md及根16—18/Web20—22验收，更新范围/来源/变更记录。先前28+7通过不能替代这次新增失败场景。

| 配方检查 | 结果 |
| --- | --- |
| `recipkg check`根/Web/core | 全通过 |
| 根递归pack→unpack→解包check→`build --lang zh-CN` | 全通过；含1.10、RFB-001/002/003、Web20/21/22及完整递归合同 |
| 归档逐项检查 | 69个普通UTF-8 .md，含LOCK.md、反馈合同及1.9历史，无源码/链接/Git元数据/执行钩子 |
| `git diff --check` | 通过；保留此前未提交工作与固定技能子模块 |

本轮临时归档SHA-256为`2b718b3c51ec151aae946a517f4001e1647d64d76ffefa92b428b0d79986c44a`，完整中文提示词为`86b6207d65f5c5facaddfe0293a76761494fe96e293ce1d4af9c04be60d6b4a7`（1436135字节），校验后自动清理。二者证明材料可生成，不证明三个缺陷已修复。

独立命名浏览器与55005实例已停止，确认端口关闭后删除合成账号数据库；本轮10张截图、脱敏actions.log、issues.json、identities.json和recipe-validation.json保留于`/tmp/rstrbot-feedback-p_a580iz/`，不创建仓库内固定产物目录。系统临时目录有清理生命周期，反馈文档保留复现步骤和证据摘要。结束时复核核心二进制、dist树和三份关键源码哈希未变；内嵌基线仍1.9，未执行Web重构建、CI、商业端点或原生平台验收。

## 三项反馈的实现修复 · 2026-10-28

按用户随后明确授权修复，业务实现位于忽略跟踪的generated/rstrbot/。根提示词1.11，根/Web包0.2.0-recipe.5，1.10原文保留；核心合同/0.2.0-recipe.2保持。三项行为、调试失败、最终验收与截图见[修复记录](REPAIR_2026-10-28.md)。旧1.10反馈保留原始失败，不覆盖为成功。

提示词浏览器记录按来源/已认证用户原子保存；旧匿名键不读取、不自动认领且不删除。会话变化通过storage事件/BroadcastChannel清理旧标签页，OAuth返回单独通知。计划表单提供每日/整点、自定义、真实IANA建议及既有核心preview，字段验证与保存白名单共用，暂停/两地时区明确且丢弃旧响应。传输错误本地化、技术详情折叠、正文保留、只读核对与手动幂等继续，390px提示按钮移到下一行。开发教程和既有开发技能同步，不新建技能仓库。

| 检查 | 最终结果与身份 |
| --- | --- |
| `mise exec node@24.22.1 -- npm --prefix web test` | 34通过，0失败/跳过；系统Node 26.12.1也通过 |
| `mise exec node@24.22.1 -- npm --prefix web run build` | TypeScript/Vite通过；同步内嵌1.11；与Node26构建的dist逐字节树摘要相同 |
| `node tests/browser/verify_feedback.mjs --evidence <系统临时目录>` | 11场景通过；Chrome156.0.8078.47、SQLite、真实诊断worker；脚本由Node26.12.1启动 |
| `node tests/browser/verify_configuration.mjs --chromium /usr/bin/chromium` | 原7场景通过；最终dist与上项相同 |
| 根/Web/core check、pack/unpack/check、中文build | 全通过；70个普通UTF-8 Markdown，含LOCK.md及1.10历史，无源码/链接/Git元数据/钩子 |
| `git diff --check` | 通过 |

新增浏览器场景逐项覆盖A/B草稿和导出、旧标签页、未归属旧键保留、拒绝存储、模拟OAuth返回、真实东京09:00/台北08:00、纽约DST序列、无预览写入/运行、非法时区定位、改配置迟到预览、保存重开与真实计划回复、中英新/既有会话离线、确认丢失后同幂等意图仅一条消息、晚确认不清新稿；包含多步的场景数为11，不把其中每个断言另计一次。HTTP延迟/丢失属于夹具，诊断回复不代表AI质量。

初次精确label定位及一次发送时序等待未通过，具体记录保存在修复报告；后者改为等待持久终态、重开实际记录并核验注入请求上游200/run ID后通过，没有减少验收断言。OAuth加载等待先注册以避免测试错过重载。最终未处理页面错误为0；这些结论不扩展为完整后台/原生/商业供应商验收。

最终产物SHA-256：

| 文件/集合 | SHA-256 |
| --- | --- |
| `target/debug/rstrbot`（未改） | `5fdd9950ae2e056e4874ed86de1f9988077864df5cfcee4e17c1a9cc58fcbbd4` |
| `web/src/BuildPromptEditor.tsx` | `9a7bcb36d890867ba55b934def61783f64a5f7e66f247693d5e001fb3660e41d` |
| `web/src/ScheduleFields.tsx` | `fcfc2386d3ea0b28b35eb0d9d83af55151d84a1f0a4ce0fc0ddbdd509275d94b` |
| `web/src/scheduleConfiguration.ts` | `18a0d8b10bc5e445cb9846ece0df0f11c2863fb33169004e7f054bebdd18e588` |
| `web/src/api.ts` | `c391b312a0c68cea3bf1332a1a1766e08c1ecd07e47ed8c16aaf317a0fabf8ca` |
| `web/src/App.tsx` | `f5fac6971d2e2297edc6998e5aaaaf1fe70590c5a37f9c592ce1bc5764ed8ead` |
| `tests/browser/verify_feedback.mjs` | `ed0778a11702c80ad7088a71637b7c19e5aad3094507e0ebe392520e7428f732` |
| `docs/RECIPE_BUILD_PROMPT.md` | `da97d756a3ffaf3fde9409630b680d0458d3f92660448020c5c763a650ac5f0a` |
| `web/dist`树 | `ff1122682fc665c113262a6ed8b5a2a54790f06c924f7aecb67b72782d51c734` |
| 临时Recipkg归档 | `41d8b98911a0c10b8766ad3bc1db15bbf41dc9eb6e339704e550d015b40d29b0` |
| 临时完整中文提示词 | `db89819c1dccc9d8f1e135896032d539f7c031c8de59a2ae60ae9e1f1cef037e` |

归档/完整提示词（1449144字节）已自动清理。最终浏览器报告、源码/截图摘要与配方校验摘要保留在`/tmp/rstrbot-repair-evidence-x6vzn42u/`；测试实例、浏览器和合成数据库已清理。该系统临时目录可能被系统回收。实现仍属本地生成增量，未推送生成源码、触发CI、发布或部署；未做真实Google/实体认证器、商业端点/投递、跨实例浏览器、PG/Valkey、完整Rust测试和原生设备矩阵。

## 配方1.12 · Pro交接

用户新增端到端竞品对比/OpenClaw issue反馈目标，已保留1.11历史并同步根.6及相关合同。完整输入已交给Pro，状态与私有材料见[PRO_HANDOFF_2026-10-28.md](PRO_HANDOFF_2026-10-28.md)；根归档从Drive重下载字节相等，八包解包校验通过。尚未收到Pro成稿，也未进行竞品实机对比；生成实现仍为1.11。本节不改写前述已完成/未完成检查。

## 配方1.13 · Pro回传与本地验证

已合入Pro的47份完整文件，并同步现有开发/插件技能与归档消费者入口；根1.13/.7、Web.6、core.3，其余五包.2。89份作者Markdown、90份归档Markdown，八包check、pack/归档check/解包check/中文build通过，52/215/287原文与旧权威接口/协议/历史保留，新增202个验收ID。最终归档SHA-256为`3f4532c136867c487a5069a68677814332fd44df1e88492d00fc38ed8e9ac694`。原始回传、局部复核变更和证据见[PRO_HANDOFF_2026-10-28.md](PRO_HANDOFF_2026-10-28.md)。这些是配方材料验证，软件和竞品实测仍not_executed，不宣称已全面超越。

## 1.19 配方校验

`recipkg check`、递归打包、归档check、解包check及中文提示词生成通过；归档有95个普通UTF-8 Markdown文件，生成提示词包含94个非LOCK文件的全文。归档及临时提示词放在系统临时目录；该检查不等于独立重建或完成软件验收。首次打包命名未使用规定的 `.recipe.tar.gz` 后缀而被CLI拒绝，改正后通过。

## 1.20 配方校验

根配方 `.14` 的 `recipkg check`、递归打包、归档检查、解包检查与中文提示词生成通过；归档为96个UTF-8 Markdown文件，非LOCK材料均包含在完整提示词中。临时归档保存在仓库外；这些检查不等于独立软件重建或所有功能验收完成。

## 1.21 配方校验

根配方 `.15` 的 `recipkg check`、递归打包、归档检查、解包检查及中文提示词生成通过；归档共97个UTF-8 Markdown文件，非LOCK材料均包含在完整提示词中。临时归档保存在仓库外；这不是独立软件重建或全面验收证明。
