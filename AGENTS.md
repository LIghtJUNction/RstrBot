# RstrBot 开发约定

RstrBot仓库本体是Recipkg配方，入口recipes/rstrbot/BUILD_PROMPT.md。项目名和远端保持原名；不得另建带迁移后缀的项目或仓库。用户最新布局选择优先于技能中的默认独立工作区/子模块方案。

recipes/只含UTF-8 Markdown；根包声明的依赖必须指向有效配方，递归材料完整交付。源码、二进制、安装脚本、文件系统链接、执行钩子和真实用户数据不得进入配方。generated/<implementation-name>/是可重新生成的本机实现目录，由Git忽略；现有源码及未提交改动在generated/rstrbot/完整保留，保护尚未提炼或未完成的用户改动。

每次收到产品改进，先更新主BUILD_PROMPT、相应组件/ACCEPTANCE和CHANGELOG，保留历史要求，再生成或修改实现。服务端仍独立执行授权/验证；UI隐藏不授予权限。未知费用、未知发送和未执行验收须如实记录。

使用Recipkg当前CLI运行check、pack、归档check和build --lang zh-CN；归档放artifacts/，位于生成源码目录外。核对归档只有普通UTF-8 .md、完整依赖/LOCK，生成提示词包含全部材料。模板生成/打包通过不表示软件已独立重建。

核心与四SDK语言保持用户选择，辅助工具可选。完整需求、视觉/交互、国际化、平台和性能未知见根配方。源实现检查在generated/rstrbot/运行，结果按对应源码/产物摘要记录；保持本地验证，不主动触发CI。正式发行构建/签名/部署要求在配方和生成实现中继续保留。

GitHub发布仍使用LIghtJUNction/RstrBot，配方仓库须保留既有topic并添加recipkg。只有现有发布授权覆盖时推送/改远端元数据；不得把配方迁移写成已上线新软件。
