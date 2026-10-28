# RstrBot

面向个人与团队的私有AI工作区。这个仓库以Recipkg配方保存产品行为、接口、依赖和验收，供Agent在独立生成目录重建实现。

从[完整重建入口](recipes/rstrbot/BUILD_PROMPT.md)开始。配方覆盖多用户与工作区、对话和工具、模型/渠道、知识与记忆、计划任务、可靠投递、四语言SDK、Web与桌面目标；完整第二阶段蓝图仍有待实现范围，见[当前范围](recipes/rstrbot/CURRENT_SCOPE.md)。

| 路径 | 用途 |
| --- | --- |
| recipes/rstrbot/ | 产品入口、52项需求、215需求族/287原验收、设计、环境、性能计划和开发技能 |
| recipes/rstrbot-*/ | 插件协议、四语言SDK、宿主/worker、存储、核心、Web与桌面的递归配方 |
| generated/rstrbot/ | 既有源码及本地未提交改动、未来重建输出；临时生成物，Git忽略 |
| .agents/skills/ | 两个独立技能仓库的固定提交子模块；配方内另有可独立打包的Markdown快照 |

项目与仓库保持RstrBot原名。配方是主要输入，生成源码不作为配方依赖。历史源码提交仍在Git历史；当前本机源码完整移入generated/rstrbot/，555个来源文件内容已核对，详见[来源记录](recipes/rstrbot/SOURCE_EVIDENCE.md)。

两个技能各有独立仓库：[rstrbot-development](https://github.com/LIghtJUNction/rstrbot-development)和[rstrbot-plugin-development](https://github.com/LIghtJUNction/rstrbot-plugin-development)。克隆或更新后同步技能快照：

```sh
git submodule update --init --recursive
./scripts/sync-skills.sh
```

迭代时先在技能仓库修改、提交并推送。回到主仓库确认对应子模块无未提交改动，再快进并同步配方快照：

```sh
skill=.agents/skills/rstrbot-development # 插件技能则改为 rstrbot-plugin-development
git -C "$skill" status --short
test -z "$(git -C "$skill" status --porcelain)"
git -C "$skill" fetch origin main
git -C "$skill" merge --ff-only FETCH_HEAD
./scripts/sync-skills.sh
```

最后在主仓库提交新的子模块指针与`recipes/rstrbot/skills/`快照。子模块提交号记录在[技能来源清单](recipes/rstrbot/skills/SOURCES.md)。

安装Recipkg工具后，在仓库根执行：

```sh
tmpdir="$(mktemp -d)"
printf '临时输出目录: %s\n' "$tmpdir"
recipkg check recipes/rstrbot
recipkg pack recipes/rstrbot -o "$tmpdir/rstrbot-0.2.0-recipe.1.recipe.tar.gz"
recipkg build "$tmpdir/rstrbot-0.2.0-recipe.1.recipe.tar.gz" --lang zh-CN --request '在 generated/rstrbot/ 中重建，遵守配方与验收要求' --output "$tmpdir/rstrbot-build.md"
```

打包文件和build提示词写入系统临时目录，不在仓库中建立产物目录；需要继续使用时从命令输出中取该目录路径。build只生成提示词。把该Markdown交给Agent，才会开始实际实现与验收。源码语言保持Rust核心、TypeScript前端及Rust/Python/TypeScript/C# SDK，辅助工具可选。性能预算暂无数字，重建后按[性能计划](recipes/rstrbot/PERFORMANCE.md)测量。

本机已有生成实现可从该目录运行：

```sh
cd generated/rstrbot
mise exec -- cargo run --locked -- --web-dir web/dist
```

默认打开http://localhost:8787，完整首次初始化与使用说明见本机generated/rstrbot/README.md。打包后的Rust服务和worker无需Node/Python/.NET运行时；原生桌面、外部账号与目标平台需要分别验证。

每次改进先更新[构建提示词](recipes/rstrbot/BUILD_PROMPT.md)、相关组件合同/验收和[版本记录](recipes/rstrbot/CHANGELOG.md)，再生成实现。开发技能在[.agents/skills/](.agents/skills/)；产品内教程/SDK技能材料随根配方保留。

[依赖图](recipes/rstrbot/DEPENDENCIES.md) · [集成验收](recipes/rstrbot/ACCEPTANCE.md) · [设计](recipes/rstrbot/DESIGN.md) · [本次校验](docs/MIGRATION.md) · [许可](LICENSE)
