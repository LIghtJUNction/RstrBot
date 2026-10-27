# RstrBot 设计来源

研究日期：2026-10-28，Asia/Taipei。链接可能继续变化；以下记录本次读取范围与冻结参考版本。

## 1. 需求优先级

用户本轮明确决定：独立项目 RstrBot、Rust、原生多用户、SQLite/PostgreSQL 初始化选择、可选 Valkey、重新设计前端、参考指定两个项目的设计方法与代码品味，以及仅交付产品要求/目标/蓝图。

此前明确需求继续有效：插件与核心解耦；语言运行时也是插件；Python/Rust/C#/TypeScript SDK；Python 入口点发现；mise 与 uv；旧版、OpenCode、Pi 加载器；供应商和平台均插件化；平台可独立 MCP；Unix socket 优先；桌面与服务器部署；真实兼容测试；详细插件开发文档。它们在本轮转化为待实现目标，不能继承旧实现的通过状态。

## 2. 冻结的设计参考

| 来源 | 冻结提交 | 本次读取 | 使用方式 |
| --- | --- | --- | --- |
| [particle-sculpture-skill](https://github.com/LIghtJUNction/particle-sculpture-skill) | 2a6f5ac818f722be95ff12aca5f807adf593b135 | SKILL.md、DESIGN.md、PRODUCT.md、references/design-directions.md、references/color-and-motion.md | 提炼主体先行、构图、独立主题/动作、可访问回退和生命周期；没有复制演示页面、素材或运行时代码 |
| [BeautifulRust](https://github.com/morganlinton/BeautifulRust) | 58757ee890da9b3c8d5b785adc7a8d53cce27307 | README.md、skills/rust-best-practices/SKILL.md | 提炼类型、最小公开面、所有权、错误、并发和验证目标；没有照抄代码片段或引用其模型基准作为效果证明 |

本文档包不含上述仓库代码。未来若复用其运行时、示例或素材，需要按相应来源的实际许可和 NOTICE 处理，不能把设计参考当成复制代码的许可判断。

## 3. 官方规范与事实边界

| 原始资料 | 支持的事实/决策 |
| --- | --- |
| [SQLite WAL](https://www.sqlite.org/wal.html) | 单写者、读与检查点关系、网络文件系统限制、耐久设置取舍 |
| [PostgreSQL 行级安全](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | 行级策略及所有者/高权限角色例外；应用归属校验仍必要 |
| [Valkey replication](https://valkey.io/topics/replication/) | 异步复制及故障边界；本设计不将其作为唯一权限/任务事实来源 |
| [PyPA Entry points](https://packaging.python.org/en/latest/specifications/entry-points/) | 标准包入口点元数据与可互操作发现 |
| [GitHub Topics](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/classifying-your-repository-with-topics) | topic 合法字符；清单标识与 GitHub topic 需区别 |
| [WCAG 文字对比度](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) | 普通文字与大字的对比度目标；不等于完整可访问性认证 |

官方资料用于确认机制边界。默认值、产品优先级、性能预算和界面方向是本轮提出的设计目标，不是官方保证或实测成绩。

## 4. 用户需求快照

第一阶段需求表：[需求清单](https://docs.google.com/spreadsheets/d/1kIAfZgB_pXpb9Xe1Sbdh_HVGqz-PBG3jhoNzVL8bk7E/edit)。本次使用已保存的 2026-10-27 快照，215 个需求族、287 条验收；未把该表在本轮宣称为再次下载的最新 revision。

快照 SHA-256：a14a3759187fc44aa4f6b228cccd01bb135f223ce91ec122af6ba596f66f5a36。

原始字段完整保存在 BOOTSTRAP_TRACEABILITY.json 的 baseline_snapshot 中；每条旧验收另有新需求归属。旧源码、可执行文件与第一阶段构建结果没有放入新设计包。

## 5. Issue 研究范围

| 仓库 | 深读 issue | open / closed | 评论与后续核对 |
| --- | ---: | --- | --- |
| [openclaw/openclaw](https://github.com/openclaw/openclaw/issues) | 18 | 9 / 9 | 作者纠正、关闭原因、相关评论和 6 个 PR 元数据；长线程未声称逐条读完 |
| [Mai-with-u/MaiBot](https://github.com/Mai-with-u/MaiBot/issues) | 15 | 6 / 9 | 所选 issue 的 33 条评论，以及必要的关闭提交与 PR 状态 |
| [langbot-app/LangBot](https://github.com/langbot-app/LangBot/issues) | 15 | 9 / 6 | 所选 issue 的 31 条评论，以及迁移议题与相关合并记录 |
| 合计 | 48 | 24 / 24 | 目的抽样，非全仓库穷尽审计或用户调查 |

候选来自近期更新、主题检索与历史高信息问题，选择依据是用户影响、可复现描述和与 RstrBot 目标的关系。热度和重复评论不代表独立用户数量；上游优先级不直接复制为本项目优先级。

本轮经 GitHub 插件读取官方仓库正文、评论与元数据。没有运行这些产品或复现其故障；报告者实验、维护者说明、自动审查和本轮推断分别记录。自动关闭、not_planned、迁移到其他仓库、合并到 main、进入正式版本，都不是同一种状态。

完整 issue URL、标题、状态、更新时间、证据说明与 RstrBot 推导保存在 research/ 下对应的 Markdown 与 JSON 文件。RESEARCH_AND_DECISIONS.md 负责把证据归并为设计决定，不重复复制长日志或个人信息。
