---
name: rstrbot-plugin-development
description: Use when updating RstrBot plugin protocol/SDK recipes or generating and validating plugins.
---

# rstrbot-plugin-development

先读recipes/rstrbot-plugin-protocol/、rstrbot-sdk/与rstrbot-plugin-host/的BUILD_PROMPT和验收，再读核心集成边界。修改协议时同时更新四语言SDK合同与共同向量，不能只改某一语言实现。

在generated/rstrbot-implementation/生成插件与SDK；不将原源码放入配方或依赖来源。区分包、实例、调用；声明能力不授予权限，worker不接受核心管理权限。凭据只传目标provider，普通工具不接收秘密。

取消须回收进程树，未知外部副作用不重放；可信独立进程不等于恶意代码沙箱。用真实四语言进程运行共同向量，缺运行时记not_executed。先更新配方/验收/根版本记录，再生成代码并按当前产物实际验证。

当前1.0参考和向量保持原字节；EXTENSION_PROTOCOL的1.1-draft、SDK_CAPABILITY_CONTRACT、宿主生命周期/兼容合同及核心EXTENSION_API仅为未来协商合同，不能写成已有能力。受控host服务是逐方法/资源/调用代次授权的白名单，不是核心管理RPC。真实未修改兼容样本按COMPATIBILITY_FIXTURES核摘要，再分别验证导入、行为、恢复和清理；样本冻结不算兼容通过。生成文档和页内教程同步这些边界；命令仅在对应生成文件实际存在时运行。

1.14上游模型拒绝按协议补充PROVIDER_ERRORS分类：四SDK向量保持同一整数码、旧host和旧worker通用回退；中英文提示只使用安全分类，不能暴露响应体或把上游401当作用户登出。实际未运行的语言须单独标记。

1.15生成实现位于 `generated/rstrbot-implementation/` 源码子模块；核对子模块URL和固定提交，配方归档不包含源码或Git元数据。

1.29 按 RSTRBOT-38 实测交互及计划运行的工具目录分页、权限和超限错误；工具目录完整性不得扩大授权。

1.30 按 RSTRBOT-39 核对文本附件原文、模型账本、重启、权限和同意图恢复；文本附件不能冒充图片或完整媒体支持。

1.31 接入方按 RSTRBOT-40 不得依赖聊天接口忽略未知字段；未支持的媒体必须明确拒绝，不能静默遗漏。


1.32 仅向明确支持图片的 OpenAI 兼容模型发原始图片；核对图文、调用预算和协议上限，不能把假夹具视为真实服务验收。
