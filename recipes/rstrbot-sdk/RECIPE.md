# rstrbot-sdk 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot-sdk"
version = "0.2.0-recipe.1"
kind = "software"
description = "Rust、Python、TypeScript、C# 的共同插件 SDK"
entry = "BUILD_PROMPT.md"

[dependencies.protocol]
source = "../rstrbot-plugin-protocol"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
