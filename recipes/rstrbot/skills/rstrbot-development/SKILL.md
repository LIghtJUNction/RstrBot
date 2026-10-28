---
name: rstrbot-development
description: Use when updating RstrBot recipes, generating the implementation, or changing product behavior.
---

# rstrbot-development

先阅读recipes/rstrbot/BUILD_PROMPT.md、SOURCE_EVIDENCE.md及受影响组件合同。配方是主要输入，生成实现位于generated/rstrbot/且不作为配方依赖。

用户每次改进要求先更新相应配方、验收和根CHANGELOG，保留历史，再修改/生成代码。核心Rust、前端TypeScript、四SDK语言保持；UI选项依据真实API，权限、密钥和副作用规则由服务端执行。

运行recipkg check recipes/rstrbot、pack、归档check及完整build提示词验证；临时输出放在仓库外的系统临时目录，不要创建固定产物目录。业务实现检查在generated/rstrbot/执行，来源版本/环境/通过和未执行明确记录。代码可生成，未知用户改动仍须先提炼或完整保留。项目名保持RstrBot，不新建带迁移后缀的仓库。
