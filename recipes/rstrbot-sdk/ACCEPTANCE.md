# rstrbot-sdk 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-SDK-01 | 构建 Rust echo，运行共同协议向量 | 初始化、发现、Unicode、通知、错误及非法凭据均按 protocol 一致 |
| RSTRBOT-SDK-02 | 分别以 Python、TypeScript、C# 真实进程运行同一组向量 | 各语言逐项给出结果；缺运行时为 not_executed |
| RSTRBOT-SDK-03 | 四个示例各处理一个等待调用，分别发 cancel、截止超时和断开 | 取消传递，无后续正常成功；资源在有界时间内回收 |
| RSTRBOT-SDK-04 | Python 列出含有导入副作用的入口点元数据 | 不执行模块；明确指定唯一名字后才导入；重名失败 |
| RSTRBOT-SDK-05 | 在两个工作区、两个 generation 中并发启动示例 | 进度、参数、取消与结果不串实例 |
| RSTRBOT-SDK-06 | 仅安装 Rust 产品运行包，未安装 Python/Node/.NET | Rust 核心与 Rust worker 可运行；只有选择相应语言插件时才需要它的运行时 |
