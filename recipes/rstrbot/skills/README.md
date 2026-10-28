# 随配方交付的开发技能

本目录的两个`SKILL.md`是独立技能仓库的快照：[`rstrbot-development`](https://github.com/LIghtJUNction/rstrbot-development)和[`rstrbot-plugin-development`](https://github.com/LIghtJUNction/rstrbot-plugin-development)。固定提交记录在[SOURCES.md](SOURCES.md)。

主仓库通过`.agents/skills/`下的Git子模块迭代技能；更新子模块后运行`./scripts/sync-skills.sh`，同步技能文件和来源提交号。不要直接编辑本目录的快照。

快照让Recipkg包不依赖Git子模块，也确保开发技能与重建配方一起交付。技能里的源码路径、命令应映射到新实现；它们不授予宿主权限，也不表示引用的实现已存在。重建后与离线教程、SDK说明一起随运行归档交付，按新结构维护链接。

Recipkg的`skills sync --dest <directory>`用于把已安装配方携带的技能部署到明确指定的Agent技能目录；它负责安装分发，不更新本仓库的技能子模块或配方快照。
