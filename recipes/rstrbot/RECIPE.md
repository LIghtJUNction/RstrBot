# rstrbot 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot"
version = "0.2.0-recipe.19"
kind = "software"
description = "RstrBot 完整产品重建入口、需求追踪与跨组件集成"
entry = "BUILD_PROMPT.md"

[dependencies.desktop]
source = "../rstrbot-desktop"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
