# RstrBot

**让个人与团队把消息、模型、工具和资料组织成可控制、可恢复的工作。**

第二阶段产品设计基线 1.0 · 2026-10-28

本项目本轮交付是独立的构建提示词、产品需求与软件蓝图。核心采用 Rust，原生支持多用户；初始化选择 SQLite 或 PostgreSQL，Valkey 按需开启。所有实现与测试状态均为未开始，文档中的验收项是未来实施目标。

## 从哪里读

| 文件 | 用途 |
| --- | --- |
| [BUILD_PROMPT.md](BUILD_PROMPT.md) | 下一轮开发的独立构建提示词；不需要旧源码才能理解主要目标 |
| [PRODUCT_BLUEPRINT.md](PRODUCT_BLUEPRINT.md) | 用户旅程、数据归属、插件边界、数据库、可靠性、实施顺序 |
| [FRONTEND_DESIGN.md](FRONTEND_DESIGN.md) | 原创前端方向、页面流程、粒子设计、视觉规范、手机与可访问性 |
| [ENGINEERING_GOALS.md](ENGINEERING_GOALS.md) | Rust 质量、成熟工具选择原则、协议与 SDK 文档交付目标 |
| [REQUIREMENTS_AND_ACCEPTANCE.md](REQUIREMENTS_AND_ACCEPTANCE.md) | 52 项稳定需求编号和可观察验收 |
| [RESEARCH_AND_DECISIONS.md](RESEARCH_AND_DECISIONS.md) | 48 条 issue 如何影响设计，哪些原建议没有直接采用 |
| [BOOTSTRAP_TRACEABILITY.md](BOOTSTRAP_TRACEABILITY.md) | 第一阶段 215 个需求族与 287 条验收的映射索引 |
| [BOOTSTRAP_TRACEABILITY.json](BOOTSTRAP_TRACEABILITY.json) | 原始需求完整快照、全部验收与新归属，便于后续工具处理 |
| [requirements.json](requirements.json) | 52 项新需求的结构化版本 |
| [SOURCES.md](SOURCES.md) | 冻结参考版本、来源和研究范围 |
| [VALIDATION.md](VALIDATION.md) | 本设计包实际完成的文档检查与未执行事项 |
| research/ | 三个参考项目的 issue 证据附录；每项含状态与推断边界 |

## 本轮确定的默认方向

- 个人与团队使用同一套用户、工作区和权限模型，私有会话不因同属工作区而默认共享。
- SQLite 面向轻量单实例，PostgreSQL 面向更高并发；Valkey 不保存唯一业务事实。
- 核心处理身份、授权、运行与资源；供应商、平台、语言运行时和生态加载器通过协议扩展。
- 四语言 SDK、旧版/OpenCode/Pi 加载器与旧 HTTP 兼容保留为完整需求，逐能力验证。
- 运行、工具副作用、平台投递和记录保存分别显示；未知结果先核对，避免重复执行。
- 前端采用“墨色折线”方向，粒子用于品牌展示，工作台优先清晰、轻量和可访问。

## 如何进入下一轮

以 BUILD_PROMPT.md 为持续演进的主输入；详细验收与证据在同包内可追溯。旧阶段代码独立归档，不把旧内部实现作为必须延续的约束。新增用户要求先更新主提示词与相关需求编号，再安排实现和测试。

本轮没有创建或发布可运行软件、GitHub Release 或已验证安装器；也没有执行新的兼容性、数据库或桌面测试。不能将设计完成写成软件完成。
