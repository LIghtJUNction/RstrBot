# rstrbot-plugin-host 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot-plugin-host"
version = "0.2.0-recipe.2"
kind = "software"
description = "受控插件宿主与 OpenAI、diagnostic、Webhook 参考 worker"
entry = "BUILD_PROMPT.md"

[dependencies.sdk]
source = "../rstrbot-sdk"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
