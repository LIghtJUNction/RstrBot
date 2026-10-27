# Issue 研究与 RstrBot 设计决定

日期：2026-10-28。48 条精选 issue 的正文已读取，相关评论与关闭/修复状态按研究附录说明核对。本轮没有独立复现上游问题，不能由样本推断整个生态的故障率或用户数量。研究的用途是发现值得验证的产品边界。

## 一、如何判断是否纳入

优先考虑影响数据隔离、已发生操作的真实性、可恢复性和日常使用的问题；其次考虑插件可扩展性与维护成本。具体供应商的单次异常用于构造协议验收，不直接把其私有实现写入核心。意见或提案是需求线索，不自动成为最终方案。

用户已明确要求的多用户、双数据库、可选 Valkey 与独立品牌具有最高优先级。以下取舍由这些要求和研究证据共同推导，优先级属于 RstrBot 的设计判断。

## 二、采用的设计决定

### D01 原生隔离覆盖所有入口

**设计：**用户、工作区、会话和服务身份分开；附件、记忆、搜索、运行流、插件数据和导出使用同一归属边界。单独建一个随机文件夹不能代替隔离。

**验收重点：**双工作区与同工作区私聊同时验证，包括文件工具、shell、搜索与后台任务。

**归属：**RB-003、004、005、006、007。**证据：**[OC-01](https://github.com/openclaw/openclaw/issues/8081)、[OC-02](https://github.com/openclaw/openclaw/issues/20305)、[OC-03](https://github.com/openclaw/openclaw/issues/3259)、[OC-04](https://github.com/openclaw/openclaw/issues/186216)、[LB-09](https://github.com/langbot-app/LangBot/issues/2639)。

### D02 权限随身份与能力变化

**设计：**安装入口不影响权限审查；包权限与工作区实例权限分开。个人 key 随所属用户权限收缩；服务身份按自身授权。Runner、工具、监听器、命令分别检查。

**验收重点：**降权、热安装、旧任务迟到、能力升级均不能偷偷扩大范围。

**归属：**RB-004、014、017、019。**证据：**[OC-05](https://github.com/openclaw/openclaw/issues/186189)、[LB-08](https://github.com/langbot-app/LangBot/issues/2629)、[LB-09](https://github.com/langbot-app/LangBot/issues/2639)。

### D03 数据库替换不替代正确性

**设计：**SQLite 和 PG 共享业务契约；Valkey 可加速但不保存唯一任务/权限事实。执行完成、消费延迟、落库失败、投递未知分别记录。

**验收重点：**锁等待和终态保存失败不误报成模型超时；缓存中断不放开硬限制。

**归属：**RB-008、009、010、011、045。**证据：**[LB-01](https://github.com/langbot-app/LangBot/issues/2761)、[LB-02](https://github.com/langbot-app/LangBot/issues/2747)、[LB-10](https://github.com/langbot-app/LangBot/issues/2241)、[OC-16](https://github.com/openclaw/openclaw/issues/143524)、[OC-17](https://github.com/openclaw/openclaw/issues/114612)。

### D04 群聊可以合理保持安静

**设计：**在昂贵规划前做可扩展门控，静默消费推进游标；群聊/私聊独立策略，自身回声不重新唤醒。

**验收重点：**持续背景闲聊不触发强制昂贵请求；历史保留，已消费不伪装成已回答。

**归属：**RB-026、027。**证据：**[MAI-03](https://github.com/Mai-with-u/MaiBot/issues/2276)、[MAI-11](https://github.com/Mai-with-u/MaiBot/issues/1238)、[MAI-12](https://github.com/Mai-with-u/MaiBot/issues/1854)。

### D05 上下文与记忆可恢复可纠正

**设计：**历史、工作集、滚动摘要、长期记忆分开；保存来源与版本，稳定 ID 消歧；权重下降不等于删除，归档恢复需要恢复检索资格。

**验收重点：**重启核对实际模型输入，删除/修改失效派生数据，同名人物不会互相污染。

**归属：**RB-003、026、050。**证据：**[MAI-02](https://github.com/Mai-with-u/MaiBot/issues/2286)、[MAI-06](https://github.com/Mai-with-u/MaiBot/issues/2259)、[MAI-07](https://github.com/Mai-with-u/MaiBot/issues/2257)、[OC-06](https://github.com/openclaw/openclaw/issues/7707)、[LB-12](https://github.com/langbot-app/LangBot/issues/2066)。

### D06 多轮工具要有可靠结尾

**设计：**完整保存工具证据与供应商元数据；收尾不重新授权工具；无效循环和无进展提前停止，普通文本不执行。

**验收重点：**模型重复提议不重复副作用；未知操作只核对；跨协议不伪造签名。

**归属：**RB-023、028、029。**证据：**[LB-04](https://github.com/langbot-app/LangBot/issues/2511)、[LB-13](https://github.com/langbot-app/LangBot/issues/1899)、[OC-12](https://github.com/openclaw/openclaw/issues/186205)、[MAI-10](https://github.com/Mai-with-u/MaiBot/issues/1710)。

### D07 调度、热变更和恢复有代次

**设计：**任务和投递持久化，旧代回调不能复活新状态；迁移失败不扩大安静时段或权限；恢复先暂停副作用。

**验收重点：**并发认领、漏跑、重启和配置失败可追溯，不通过删除未知锁或盲目重放“修复”。

**归属：**RB-008、012、013、031、032。**证据：**[OC-07](https://github.com/openclaw/openclaw/issues/186186)、[OC-08](https://github.com/openclaw/openclaw/issues/111897)、[OC-09](https://github.com/openclaw/openclaw/issues/179927)、[OC-10](https://github.com/openclaw/openclaw/issues/185399)、[OC-11](https://github.com/openclaw/openclaw/issues/186218)。

### D08 插件离线不会拖垮核心

**设计：**依赖准备、启动、握手、调用和退出分别有状态及期限；可重试暂时故障，永久错误停止自动循环。

**验收重点：**相同配置再次连接能够恢复；坏插件退出后核心健康且所属进程树被回收。

**归属：**RB-015、017、033、045。**证据：**[MAI-08](https://github.com/Mai-with-u/MaiBot/issues/2260)、[MAI-13](https://github.com/Mai-with-u/MaiBot/issues/1812)、[OC-15](https://github.com/openclaw/openclaw/issues/144911)、[LB-15](https://github.com/langbot-app/LangBot/issues/2713)。

### D09 能力目录按实际作用域产生

**设计：**平台与供应商都是插件；当前会话、权限和健康共同决定可用能力，并解释不可用原因。模型配置分别验证能连接、能发现和能实际调用，测试结果不能仅凭模型名称推断。

**验收重点：**私聊不暴露仅群聊可用动作；定时任务不靠扩大 allowlist 解决工具缺失。

**归属：**RB-007、023、024、038、039。**证据：**[MAI-09](https://github.com/Mai-with-u/MaiBot/issues/2250)、[MAI-14](https://github.com/Mai-with-u/MaiBot/issues/1930)、[OC-13](https://github.com/openclaw/openclaw/issues/120385)、[LB-05](https://github.com/langbot-app/LangBot/issues/2433)。

### D10 资源读取与执行分离

**设计：**获准只读资源不强制启动代码执行沙箱；二进制要验证真实消费；产物确认交付前保留可恢复副本。

**验收重点：**无沙箱可读授权说明和附件；发送失败不会清空待交付文件。

**归属：**RB-006、034、037。**证据：**[LB-06](https://github.com/langbot-app/LangBot/issues/2410)、[LB-07](https://github.com/langbot-app/LangBot/issues/2775)、[LB-11](https://github.com/langbot-app/LangBot/issues/2634)。

### D11 索引需要坏记录隔离与检查点

**设计：**上传、解析、索引发布分别展示；永久失败项隔离，其他项继续；外部检索和本地索引区分来源与新鲜度。

**验收重点：**超长输入不会永久堵住所有重建，坏快照可恢复且不伪报成功，媒体类型按真实内容规范化。

**归属：**RB-035、036、037。**证据：**[MAI-04](https://github.com/Mai-with-u/MaiBot/issues/2268)、[MAI-05](https://github.com/Mai-with-u/MaiBot/issues/2267)、[MAI-15](https://github.com/Mai-with-u/MaiBot/issues/2293)、[LB-14](https://github.com/langbot-app/LangBot/issues/2030)。

### D12 流、历史和桌面验证同样重要

**设计：**平台侧节流合并保留尾部；重复消息不拖垮页面；桌面安装器按真实环境验收。

**验收重点：**刷新/重连/分段不重复内容，Web 通过不能替代 macOS/Windows 原生验证。

**归属：**RB-008、037、040、044、046。**证据：**[MAI-01](https://github.com/Mai-with-u/MaiBot/issues/2292)、[LB-03](https://github.com/langbot-app/LangBot/issues/2540)、[OC-14](https://github.com/openclaw/openclaw/issues/186191)、[OC-18](https://github.com/openclaw/openclaw/issues/186215)。

## 三、没有直接照搬的建议

| 来源现象或建议 | 本轮处理 | 理由 |
| --- | --- | --- |
| 用共享目录中的新子目录或保留期解决附件隔离 | 不采用为授权方案 | 可减少混乱/磁盘，但 shell 仍可能读到同一挂载下的其他文件 |
| 工具收尾失败就重新放开工具配置 | 不作为通用修复 | 可能重新开放副作用；应形成目标协议合法、工具仍禁用的收尾 |
| Valkey 超时一律 fail-open | 不用于权限与硬预算 | 性能降级不能扩大可执行操作集合 |
| 改 PostgreSQL 或增大连接池即可解决终态错误 | 不作为正确性证明 | 事件顺序、状态提交与重试语义仍需同一测试 |
| 同一个扩展开关决定 Runner、命令、工具与监听器 | 拆分能力角色 | 不同角色的启用与授权规则不相同 |
| 长期不回复就强行启动一次规划来清空积压 | 不采用 | 静默消费可以推进游标，保留历史而不增加无意义调用 |
| 记忆未被召回几天就永久删除 | 不作为默认 | 降权、归档、停用和删除应分别由用户理解和控制 |
| 未知锁文件可直接删除；未知操作可直接重试 | 不采用 | 可能属于仍在运行的执行者，或已在外部完成 |
| 接入更多模型名字等于全面兼容 | 不采用 | 必须实际验证端点、认证、工具、多模态和流行为 |

## 四、状态纠偏记录

- OpenClaw 的多用户需求中，#8081 为 not_planned，#20305 因不活跃关闭，#3259 因分流/优先级关闭；不能写成多用户隔离已完成。#186216 的作者后续收窄了可行隔离方案。
- OpenClaw #186205 的修复 PR 已核实合并；上游关闭评论称其当时核对的发行版本尚未包含。本轮未独立验证各发行包覆盖，修复目标是保持收尾无副作用。#143524 的作者更换运行时后回报未复发，不支持“SQLite 永远不可用”的结论。
- MaiBot #2267、#2268 的关闭来自模板流程，不是修复证明。#2292 有关闭提交而没有本轮运行验证；#1854 提到的 PR #1922 未合并。#2286 明确区分本地观察与对新分支的源码核对。
- LangBot #2629 的初始授权建议在后续被更正；#2410 的完成范围不等于开放的 #2775 二进制资源能力。#2066 是转移讨论，不是上下文预算已完成。#1899 和 #2713 有关联已合并修复，应作为回归需求保留。

以上状态的具体 URL、日期、角色与证据均在对应研究附录。没有以自动审查评论代替本轮复现；也没有把上游缺陷写成 RstrBot 已存在的漏洞。

## 五、下一阶段如何使用证据

每项实现先选择对应 RB 编号，再展开所承接的旧验收和外部案例，建立自己的测试输入。可以借鉴失败场景，但不能复制报告者的成功数字。新证据推翻旧判断时更新结论与版本，不删除原始来源和推断边界。
