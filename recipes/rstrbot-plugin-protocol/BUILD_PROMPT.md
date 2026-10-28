# RstrBot 插件协议重建合同

定义协议 1.0，不限定协议消费者的内部实现；四语言 SDK 的选择由上层配方确定。完整当前参考见 [PROTOCOL_REFERENCE.md](PROTOCOL_REFERENCE.md)，声明式共同样例见 [CONFORMANCE_VECTORS.md](CONFORMANCE_VECTORS.md)。参考中的源码路径、构建命令是未来实现的说明，不是包内已有程序。

## 必须成立的公开行为

区分安装包、工作区实例、调用。Unix 使用私有 UDS：目录 0700、套接字 0600，绑定调用者 UID 和可用的子进程 PID；路径短于 104 字节。环境提供 RSTRBOT_SOCKET、RSTRBOT_TOKEN 和 JSON 的 RSTRBOT_CONTEXT（workspace_id、instance_id、generation）。Windows 的历史宿主返回 unsupported_platform；完整跨平台目标的 named pipe、ACL 和进程树回收须另行实现及原生验收。

每行一个 UTF-8 JSON-RPC 2.0 对象，最大帧 1 MiB，参数最多 512 KiB；不接受批次或未转义跨行 JSON。字符串请求 ID；initialize 验证协议版本、nonce 和完整上下文后，只执行一次业务调用。允许零个或多个进度通知，最后恰好一份相同 ID 的 result 或 error。工作进程不能主动发核心管理请求。

声明能力域为 model、tools、platform、events、tasks、knowledge、runtime；声明、握手和当前业务授权必须分别满足。通知限 model.delta、tool.progress、platform.progress、task.progress，params.request_id 绑定当前调用。取消参数带 request_id；取消不能被解释成已撤回远端副作用。

未知方法 -32601，参数错误 -32602，处理错误 -32603，初始化拒绝 -32001，取消 -32800。诊断去除密钥、头、私有目标路径与远端原始响应正文。限时、背压和协议错误必须使调用有界结束，不吞通知、伪造终态或无限增长。

## 输出与外部前提

交付可供 SDK 和宿主共同使用的协议文档、数据示例及独立验证向量。JSON-RPC 标准、操作系统套接字/进程设施是外部规范和环境前提，不是已打包的软件依赖。本包无其他配方依赖。

## 1.13扩展重建输入

继续阅读 [EXTENSION_PROTOCOL.md](EXTENSION_PROTOCOL.md)。其中1.1-draft为明确未实现的扩展，定义版本/能力交集、受限host服务、媒体/入站/钩子类型与新增验收。1.0原参考与向量保持不变，不能将草案方法放入当前已支持列表。新实现分别验收两套协议，未知版本/能力拒绝，业务失败不得隐式降版重发。身份、授权、预算和永久意图仍由核心负责。

## 1.14 安全分类

[PROVIDER_ERRORS.md](PROVIDER_ERRORS.md)声明可选的三个模型上游拒绝码；旧worker、旧host和未知错误保留通用回退，不改变1.0握手或把上游拒绝伪装成本机登录失败。
