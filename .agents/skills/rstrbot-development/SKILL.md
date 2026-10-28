---
name: rstrbot-development
description: Use when updating RstrBot recipes, generating the implementation, or changing product behavior.
---

# rstrbot-development

先阅读recipes/rstrbot/BUILD_PROMPT.md、SOURCE_EVIDENCE.md及受影响组件合同。配方是主要输入，生成实现位于generated/rstrbot-implementation/且不作为配方依赖。

用户每次改进要求先更新相应配方、验收和根CHANGELOG，保留历史，再修改/生成代码。核心Rust、前端TypeScript、四SDK语言保持；UI选项依据真实API，权限、密钥和副作用规则由服务端执行。

运行recipkg check recipes/rstrbot、pack、归档check及完整build提示词验证；临时输出放在仓库外的系统临时目录，不创建固定产物目录。业务实现检查在generated/rstrbot-implementation/执行，来源版本/环境/通过和未执行明确记录。代码可生成，未知用户改动仍须先提炼或完整保留。项目名保持RstrBot，不新建带迁移后缀的仓库。

浏览器本地草稿/版本也属于用户私有状态：按实例来源和已认证用户隔离，旧匿名数据不自动认领；会话切换清理旧视图和迟到操作。计划预览使用真实只读API；网络未知结果仅核对，不自动重放。对应本地浏览器入口为generated/rstrbot-implementation/tests/browser/verify_feedback.mjs和verify_configuration.mjs；真实商业端点/原生设备仍单独验收。

完整能力按根CAPABILITY_COVERAGE和ACCEPTANCE_HARNESS逐项实现，当前v1/1.0与未来v2/1.1-draft分开。竞品任务和OpenClaw反馈按COMPETITIVE_RUNBOOK/OPENCLAW_FEEDBACK保留版本、来源状态、下游证据与not_executed；不能把材料检查、已关闭issue或模拟用户当作软件通过/体验胜出。生成教程和内嵌技能须同步当前合同，来源文档缺失时读递归配方，不能把尚未生成的源码文档当作隐藏前提。

1.14上游模型拒绝按协议补充PROVIDER_ERRORS分类：四SDK向量保持同一整数码、旧host和旧worker通用回退；中英文提示只使用安全分类，不能暴露响应体或把上游401当作用户登出。实际未运行的语言须单独标记。

1.15生成实现位于 `generated/rstrbot-implementation/` 源码子模块；核对子模块URL和固定提交，配方归档不包含源码或Git元数据。

1.16 用户确认 MaiBot 为对比对象。首次对话默认不为标题另发模型请求；可选模型标题需用户主动启用并说明请求、费用和数据范围。按RSTRBOT-22核对账本，模拟结果不代表整体领先。

1.17 按 R13-OC-185423 验证可选供应商回退：复用已接纳消息，真冲突和 unknown 禁止回退；没有实现或实际账本时保持 gap/not_executed。

1.18 会话按已接纳发送恢复模型；缺绑定或模型失效时显式选择，共享会话拒绝私有模型。按 RSTRBOT-24 核对实际地址、隐私、重启和幂等。

1.19 按 RSTRBOT-25 检查长草稿视口重排，区分末尾编辑与手动滚动阅读；模拟浏览器单路径不能代替输入法、实体手机和全矩阵验收。

1.20 按 RSTRBOT-26 核对已接纳但确认丢失后的刷新、参数/身份隔离和显式新意图；实际消息/运行/模型请求计数不可由浏览器提示代替。

1.21 按 RSTRBOT-27 测试多标签页意图协调和过期确认；不支持协调时安全降级，跨设备与未跑的竞态不得标通过。
