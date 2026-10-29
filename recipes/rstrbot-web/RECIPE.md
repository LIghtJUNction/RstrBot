# rstrbot-web 软件配方

```toml
format = "recipkg/1"

[package]
name = "rstrbot-web"
version = "0.2.0-recipe.19"
kind = "software"
description = "单语言双主题、双形态导航与完整业务状态的Web工作台"
entry = "BUILD_PROMPT.md"

[dependencies.service]
source = "../rstrbot-core"
```

版本号标识本次配方材料，不表示完整蓝图已经实现。依赖均为重建输入；系统库、工具链、账号和远程服务另列为外部前提。
