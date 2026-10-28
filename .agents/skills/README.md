# 开发技能目录

本目录包含两类开发技能：Recipkg维护的共享技能作为Git子模块接入；RstrBot自有的开发和插件开发技能保留为本仓库普通文件。

## 初始化

```sh
git submodule update --init --recursive
```

父仓库记录每个共享技能的固定提交。检出RstrBot时，子模块不会自动下载；执行上面的命令后即可使用。

## 共享技能及上游

| 技能 | 上游仓库 |
| --- | --- |
| add-dependency | https://github.com/recipkg/add-dependency |
| build-software-package | https://github.com/recipkg/build-software-package |
| discover-dependencies | https://github.com/recipkg/discover-dependencies |
| manage-git-repository | https://github.com/recipkg/manage-git-repository |
| migrate-to-recipkg | https://github.com/recipkg/migrate-to-recipkg |
| prompt-to-software | https://github.com/recipkg/prompt-to-software |
| rewrite-software | https://github.com/recipkg/rewrite-software |
| rewrite-to-recipkg | https://github.com/recipkg/rewrite-to-recipkg |
| software-to-prompt | https://github.com/recipkg/software-to-prompt |
| test-feedback | https://github.com/recipkg/test-feedback |
| update-recipkg-in-project | https://github.com/recipkg/update-recipkg-in-project |

每个共享技能各自维护在规范上游仓库中。同步时先检查上游变更，再把子模块检出到已审阅的提交，并提交父仓库中的gitlink更新；不要把所有技能复制到一个新仓库。

## RstrBot自有技能

- `rstrbot-development/SKILL.md`
- `rstrbot-plugin-development/SKILL.md`

这两个文件由RstrBot维护，保持普通文件，不另建项目专属技能仓库。

这些是仓库开发工具，不是产品运行依赖或Recipkg配方内容。产品教程和插件SDK说明见`recipes/rstrbot/`。
