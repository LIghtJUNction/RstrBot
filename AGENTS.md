# RstrBot 开发约定

RstrBot仓库本体是Recipkg配方，入口recipes/rstrbot/BUILD_PROMPT.md。项目名和远端保持原名；不得另建带迁移后缀的项目或仓库。用户最新布局选择优先于技能中的默认独立工作区/子模块方案。

recipes/只含UTF-8 Markdown；根包声明的依赖必须指向有效配方，递归材料完整交付。源码、二进制、安装脚本、文件系统链接、执行钩子和真实用户数据不得进入配方。不要把 generated/ 加入 Git 忽略规则；现有源码及未提交改动在 generated/rstrbot-implementation/ 完整保留，不将内嵌仓库误当作普通文件批量提交。保护尚未提炼或未完成的用户改动。

每次收到产品改进，先更新主BUILD_PROMPT、相应组件/ACCEPTANCE和CHANGELOG，保留历史要求，再生成或修改实现。服务端仍独立执行授权/验证；UI隐藏不授予权限。未知费用、未知发送和未执行验收须如实记录。

使用Recipkg当前CLI运行check、pack、归档check和build --lang zh-CN；临时输出放在仓库外的系统临时目录，不要在仓库中创建固定的artifacts目录。核对归档只有普通UTF-8 .md、完整依赖/LOCK，生成提示词包含全部材料。模板生成/打包通过不表示软件已独立重建；保留验证结论于docs/MIGRATION.md。

Recipkg共享开发技能位于.agents/skills/<skill-name>，一技能一上游Git子模块，提交指针由本仓库固定；初始化用`git submodule update --init --recursive`。更新前审阅对应上游提交，再更新子模块指针。现有rstrbot-development与rstrbot-plugin-development是本项目普通技能文件，保留原文件，不新建它们的仓库或副本。仓库技能仅供开发流程使用，不作为配方依赖，也不改变recipes/仅含Markdown的规则。

核心与四SDK语言保持用户选择，辅助工具可选。完整需求、视觉/交互、国际化、平台和性能未知见根配方。源实现检查在generated/rstrbot-implementation/运行，结果按对应源码/产物摘要记录；保持本地验证，不主动触发CI。正式发行构建/签名/部署要求在配方和生成实现中继续保留。

GitHub发布仍使用LIghtJUNction/RstrBot，配方仓库须保留既有topic并添加recipkg。只有现有发布授权覆盖时推送/改远端元数据；不得把配方迁移写成已上线新软件。
