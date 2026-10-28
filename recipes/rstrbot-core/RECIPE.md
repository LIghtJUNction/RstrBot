# rstrbot-core 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot-core"
version = "0.2.0-recipe.4"
kind = "software"
description = "HTTP、CLI、执行引擎、计划、知识、身份和可靠投递核心"
entry = "BUILD_PROMPT.md"

[dependencies.storage]
source = "../rstrbot-store"

[dependencies.plugins]
source = "../rstrbot-plugin-host"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
