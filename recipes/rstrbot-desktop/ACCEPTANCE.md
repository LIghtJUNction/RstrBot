# rstrbot-desktop 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-DESKTOP-01 | Linux/macOS/Windows各自在本机准备依赖、构建匹配安装器并安装启动 | 逐平台记录实际结果；缺环境为not_executed，不能继承他平台通过 |
| RSTRBOT-DESKTOP-02 | 本机首启并重复启动同数据根，改工作目录再启动 | 数据归属稳定，一次初始化，实例锁防并发写 |
| RSTRBOT-DESKTOP-03 | 远程HTTPS模式连接并断开既有服务器 | 不启动本机核心/新数据库；服务端任务按自身授权继续 |
| RSTRBOT-DESKTOP-04 | 远程页面尝试调用文件/命令/启动器原生接口 | 未授权宿主能力不可用，不泄露本机秘密 |
| RSTRBOT-DESKTOP-05 | 正常退出/强制崩溃/升级后重启并检查进程树 | 正常退出回收；恢复保持未知副作用，不自动重发 |
| RSTRBOT-DESKTOP-06 | 在缺Windows本机transport时选择本机模式 | 明确不支持/阻塞，不能假装成功 |
| RSTRBOT-DESKTOP-07 | 升级/回滚/卸载流程检查数据库、密钥和外部插件状态 | 不被安装资源替换或删除；回滚使用兼容备份而非旧程序读新schema |

## 1.13 追加验收（全部 not_executed）

[NATIVE_DELIVERY.md](NATIVE_DELIVERY.md) 是下列场景的完整输入、操作、故障与下游断言来源；不是可选建议，不能只运行本表引用检查就声称软件通过。

| 编号 | 完整场景 | 初始状态 |
| --- | --- | --- |
| R13-DESKTOP-001 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-002 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-003 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-004 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-005 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-006 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-007 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
| R13-DESKTOP-008 | NATIVE_DELIVERY.md 的同ID场景 | not_executed |
