# 四语言 SDK

保持 Rust、Python、TypeScript、C# 四种 SDK。Rust SDK 供本机 worker 复用；Python 提供异步 serve 与 Plugin 入口，TypeScript 提供 serve 与取消信号，C# 提供 PluginServer.ServeAsync 及独立 echo 样例。内部类型可重设，线协议不可因语言而变化。

## 依赖合同

依赖 `protocol` 是唯一 wire-format、初始化上下文、进度、错误和取消的事实来源；归档内为 `.deps/protocol/`。先实现该依赖的向量，再用四种真实进程逐一执行本包验收。不能以字段名相似或单语言通过代替四语言一致。

公开 SDK 支持能力发现、工具调用和结构化结果、模型/平台消息的协议传递、进度及取消。能够传递某个域不表示核心已支持所有钩子、长期插件数据、旧生态或沙箱。标准输出在协议使用期间不得混入日志；宿主协议实际走 UDS，诊断日志写 stderr 且宿主可抑制。普通工具不接收模型凭据或数据库句柄。

Python 的 rstrbot.plugins 入口点发现只读取元数据；--list 不导入插件，--entry-point NAME 仅装载唯一匹配、导出 Plugin 的入口；重名明确失败。依赖环境按实例隔离，使用 uv 管理。运行时安装是显式部署动作，核心不隐式下载代码。

## 工具链与文档

来源快照固定 Rust 1.99.1、Node 24.22.1、Python 3.15.2、uv 0.13.6、.NET SDK 10.0.403；C# 目标 net10.0，Rust edition 2024。Python 源码声明的最低兼容版本与开发固定版本分开。未来更新应核实稳定版本、锁定解析并重跑同一向量，不能在提示词中浮写“最新”。辅助验证工具可自行选语言。

交付 SDK 公共说明、每语言 echo 示例的重建实现、共同向量和运行方式；SDK/语言运行时属于重建产物或外部工具，禁止复制现有源码进配方。

## 1.13能力补全

阅读 [SDK_CAPABILITY_CONTRACT.md](SDK_CAPABILITY_CONTRACT.md)，按依赖protocol的当前1.0和未来1.1-draft分别重建。四语言共同覆盖编码/取消/进度、受控资源与数据、钩子/命令、模型/平台/媒体/MCP和持久子任务的同义表面。扩展当前未实现，新增验收均未执行。不能以一个语言通过、仅转发JSON或历史基础向量代替完整能力；运行时仍按需安装。

## 1.14 可选上游错误码

四种语言SDK共同遵守协议补充 递归依赖 protocol 的 `PROVIDER_ERRORS.md` 的三种整数错误码与旧host/worker回退向量；消息安全、请求ID和失败终态一致，不能把新增配方当成四语言运行通过。

## 1.15 工具调用关联字段

四种SDK都将`tools.call.params.call_id`不改写地交给业务handler；缺少该可选扩展字段时仍兼容旧核心。该字段不同于JSON-RPC请求`id`和通知`params.request_id`，不承担权限或重放语义。共同向量为带唯一call_id的真实四语言进程，并检查handler返回值与输入完全相同。
