---
name: rstrbot-development
description: Use when updating RstrBot recipes, generating the implementation, or changing product behavior.
---

# rstrbot-development

先阅读recipes/rstrbot/BUILD_PROMPT.md、SOURCE_EVIDENCE.md及受影响组件合同。配方是主要输入，生成实现位于generated/rstrbot/且不作为配方依赖。

用户每次改进要求先更新相应配方、验收和根CHANGELOG，保留历史，再修改/生成代码。核心Rust、前端TypeScript、四SDK语言保持；UI选项依据真实API，权限、密钥和副作用规则由服务端执行。

运行recipkg check recipes/rstrbot、pack、归档check及完整build提示词验证；临时输出放在仓库外的系统临时目录，不创建固定产物目录。业务实现检查在generated/rstrbot/执行，来源版本/环境/通过和未执行明确记录。代码可生成，未知用户改动仍须先提炼或完整保留。项目名保持RstrBot，不新建带迁移后缀的仓库。

浏览器本地草稿/版本也属于用户私有状态：按实例来源和已认证用户隔离，旧匿名数据不自动认领；会话切换清理旧视图和迟到操作。计划预览使用真实只读API；网络未知结果仅核对，不自动重放。对应本地浏览器入口为generated/rstrbot/tests/browser/verify_feedback.mjs和verify_configuration.mjs；真实商业端点/原生设备仍单独验收。

完整能力按根CAPABILITY_COVERAGE和ACCEPTANCE_HARNESS逐项实现，当前v1/1.0与未来v2/1.1-draft分开。竞品任务和OpenClaw反馈按COMPETITIVE_RUNBOOK/OPENCLAW_FEEDBACK保留版本、来源状态、下游证据与not_executed；不能把材料检查、已关闭issue或模拟用户当作软件通过/体验胜出。生成教程和内嵌技能须同步当前合同，来源文档缺失时读递归配方，不能把尚未生成的源码文档当作隐藏前提。
