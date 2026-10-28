# rstrbot-desktop 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot-desktop"
version = "0.2.0-recipe.2"
kind = "software"
description = "三平台桌面启动器、本地sidecar与远程工作区连接"
entry = "BUILD_PROMPT.md"

[dependencies.client]
source = "../rstrbot-web"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
