# RstrBot

![软件与依赖汇聚为可重建的配方](https://raw.githubusercontent.com/recipkg/recipkg/main/assets/recipkg-cover.png)

面向个人与团队的私有AI工作区。这个仓库以Recipkg配方保存产品行为、接口、依赖和验收，供Agent在独立生成目录重建实现。

从[完整重建入口](recipes/rstrbot/BUILD_PROMPT.md)开始。配方覆盖多用户与工作区、对话和工具、模型/渠道、知识与记忆、计划任务、可靠投递、四语言SDK、Web与桌面目标；完整第二阶段蓝图仍有待实现范围，见[当前范围](recipes/rstrbot/CURRENT_SCOPE.md)。

逐项实现看[52项能力覆盖](recipes/rstrbot/CAPABILITY_COVERAGE.md)和[验收约定](recipes/rstrbot/ACCEPTANCE_HARNESS.md)；竞品任务与来源改进见[执行册](recipes/rstrbot/COMPETITIVE_RUNBOOK.md)和[OpenClaw反馈](recipes/rstrbot/OPENCLAW_FEEDBACK.md)。合同定义与材料校验不代表软件已实现或比较已胜出。

| 路径 | 用途 |
| --- | --- |
| recipes/rstrbot/ | 产品入口、52项需求、215需求族/287原验收、设计、环境、性能计划和开发技能 |
| recipes/rstrbot-*/ | 插件协议、四语言SDK、宿主/worker、存储、核心、Web与桌面的递归配方 |
| generated/rstrbot-implementation/ | 独立源码仓库的 Git 子模块，固定可重新生成的实现；不属于配方依赖 |

项目与仓库保持RstrBot原名。配方是主要输入，生成源码不作为配方依赖。既有源码及本地改动保留在独立子模块中；原始来源和后续迁移见[来源记录](recipes/rstrbot/SOURCE_EVIDENCE.md)。

安装Recipkg工具后，在仓库根执行：

```sh
tmpdir="$(mktemp -d)"
printf '临时输出目录: %s\n' "$tmpdir"
recipkg check recipes/rstrbot
recipkg pack recipes/rstrbot -o "$tmpdir/rstrbot.recipe.tar.gz"
recipkg build "$tmpdir/rstrbot.recipe.tar.gz" --lang zh-CN --request '在 generated/rstrbot-implementation/ 中重建，遵守配方与验收要求' --output "$tmpdir/rstrbot-build.md"
```

打包文件和build提示词写入系统临时目录，不在仓库中建立产物目录；需要继续使用时从命令输出中取该目录路径。build只生成提示词。把该Markdown交给Agent，才会开始实际实现与验收。源码语言保持Rust核心、TypeScript前端及Rust/Python/TypeScript/C# SDK，辅助工具可选。性能预算暂无数字，重建后按[性能计划](recipes/rstrbot/PERFORMANCE.md)测量。

首次克隆后先执行 `git submodule update --init --recursive`。生成实现可从该目录运行：

```sh
cd generated/rstrbot-implementation
npm --prefix web ci
npm --prefix web run build
mise exec -- cargo run --locked -- --web-dir web/dist
```

默认打开http://localhost:8787，完整首次初始化与使用说明见generated/rstrbot-implementation/README.md。打包后的Rust服务和worker无需Node/Python/.NET运行时；原生桌面、外部账号与目标平台需要分别验证。

每次改进先更新[构建提示词](recipes/rstrbot/BUILD_PROMPT.md)、相关组件合同/验收和[版本记录](recipes/rstrbot/CHANGELOG.md)，再生成实现。开发技能在[.agents/skills/](.agents/skills/)；产品内教程/SDK技能材料随根配方保留。

Recipkg 的共享开发技能按一仓库一个技能固定为 Git 子模块。新克隆后运行 `git submodule update --init --recursive`。要更新技能时，先在对应上游仓库审阅并选定提交，再更新本仓库的子模块提交指针。现有 `rstrbot-development` 和 `rstrbot-plugin-development` 是本项目维护的普通技能文件，继续留在原位置；不为它们另建仓库。完整清单和更新方法见[技能目录说明](.agents/skills/README.md)。

[依赖图](recipes/rstrbot/DEPENDENCIES.md) · [集成验收](recipes/rstrbot/ACCEPTANCE.md) · [设计](recipes/rstrbot/DESIGN.md) · [本次校验](docs/MIGRATION.md) · [许可](LICENSE)
