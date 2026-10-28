# 插件宿主与三个独立 worker

Rust 实现进程宿主、manifest 检查、调用监管、OpenAI 兼容模型 worker、显式 diagnostic 夹具、Webhook 出站适配器。保持可执行名称 rstrbot-openai、rstrbot-diagnostic、rstrbot-webhook。本包依赖 `sdk` 重建 Rust 公共协议与四语言一致性；`.deps/sdk/` 递归提供 protocol。其他语言 SDK 仍随产品交付，Rust 运行包不强制安装其运行时。

## 宿主合同

清单包含 id、version、绝对 executable、args、capabilities、权限披露、可选 sha256、timeouts。仅实例管理员通过本地安装入口登记可信程序；工作区管理员只管理隔离实例。清单和可执行文件校验通过不意味着用户获准操作。每次调用启动独立进程，清空环境，仅保留 PATH 及协议变量，使用私有临时目录，调用后回收进程组并清理目录。

全局 8 个 worker 槽位、排队 10 秒；连接和握手各默认 5 秒；调用默认 120 秒、最多 300 秒。业务授权在取得槽位后、握手后且写入业务请求前再次核验，核验单独限时 10 秒，调用中监听撤权。回调不持数据库事务或共享业务锁。取消、宿主 future 丢弃、超时均回收子进程组，宽限后强制终止并 reap；逃离进程组的恶意程序需 OS 隔离，当前信任模型不宣称防御它。

## 模型

OpenAI 兼容 worker 提供 model.list、model.test、model.generate；输入保留 base_url、model、api_key、max_tokens、messages、tools、tool_choice、temperature、stream 等协议含义。流式保留稀疏 tool-call 索引、ID、参数片段和供应商不透明状态，不伪造签名或费用。缺 usage 为 null。SSE 使用成熟解析设施，总输入和 JSON 均有界，缺少末尾完成标记失败。任何网络请求不自动重试。model.test 真实发请求，上层当前测试最多请求 8 个输出 token并明确可能收费。

默认只准公开 HTTPS。一次解析并固定经批准地址，拒私有/保留地址、重定向与继承代理；私网模型需实例政策和绑定目标的批准同时成立。更换完整目标或 provider 必须新凭据或明确清除。

diagnostic 明确标作非 AI，固定响应并支持 /diagnostic/tool hello 与 /diagnostic/wait；成功只说明链路和取消。

## Webhook 与独立 MCP

platform.capabilities 明确只支持出站文本、HTTP 接受回执，不宣称接收消息或远端去重。platform.send 接收核心决定的 url、token、text、operation_id 和获准私网标志。发送一次 JSON {text,operation_id} 与 Idempotency-Key；2xx accepted，非2xx rejected，未获可靠回执的网络失败 unknown。

同一适配器支持 --mcp-stdio。运营者提供固定 RSTRBOT_WEBHOOK_URL、绝对私有 RSTRBOT_WEBHOOK_STATE、可选 TOKEN 和私网开关。send_message 工具仅允许 text、operation_id，不能覆盖目标/凭据。先持久化哈希意图再发送；相同 ID 同参数返回原回执，改变参数冲突；pending/unknown 与重启均不重放。已发送但保存失败须返回真实投递结果、persistence=failed、reconciliation_required=true。它的独立状态需单独保护，不共享浏览器会话租约。

当前信任边界、协议细节和完整错误分类随依赖 protocol 一起提供。旧生态、Windows named pipe、OS 沙箱是保留目标，不能从本包当前源实现推定已支持。

## 1.13扩展重建合同

继续阅读 [LIFECYCLE_AND_ISOLATION.md](LIFECYCLE_AND_ISOLATION.md) 与 [ADAPTERS_AND_COMPATIBILITY.md](ADAPTERS_AND_COMPATIBILITY.md)，补齐原子安装/代次、实例数据、凭据代理、钩子命令、真实沙箱、通用MCP、双向平台/媒体和兼容加载。新能力均为未实现目标，不改写三个现有worker的当前能力。线格式只经声明依赖`sdk → protocol`读取；不反向依赖产品根或复制HTTP定义。重建时使用显式1.1-draft特性并执行新增验收，不重放未知副作用。

兼容验收先使用 [COMPATIBILITY_FIXTURES.md](COMPATIBILITY_FIXTURES.md) 已冻结的三个官方未修改候选；保留精确profile和未覆盖集，不能修改样本或将旧profile通过扩大为当前全部版本兼容。

## 1.14 模型提供方拒绝

OpenAI兼容worker在确实收到401、403、429时按协议给出认证、授权、限流分类；发现和生成路径一致。保留未知错误及原有状态语义，不读取、记录或返回上游原始响应与凭据。
