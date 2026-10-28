# Rust 核心服务

重建 rstrbot 可执行程序及公共HTTP API v1。默认127.0.0.1:8787，同源提供Web静态目录和API。读 [HTTP_CONTRACT.md](HTTP_CONTRACT.md)、[OPENAPI.md](OPENAPI.md) 与 [CLI.md](CLI.md)：OPENAPI.md 包含当前完整声明式OpenAPI 3.1对象，所有66个API操作和2个健康操作、DTO、状态码、错误、字节限制和示例均须保留。示例不是已执行回执。参考命令指向未来实现，配方未附带程序。

## 依赖使用合同

`storage`提供所有持久身份、权限、资源CAS、预算、事件和恢复事实；每个HTTP/任务/插件入口都必须使用这些规则。`plugins`提供受控进程宿主、模型/工具/Webhook worker并递归提供四语言SDK与协议。先分别验收，再通过真实HTTP和进程集成；不把包声明或握手当业务授权。

## 核心用户行为

首次终端证明有效15分钟，原子建立管理员和首工作区；已初始化拒绝再次接管。Cookie为HttpOnly SameSite Strict；Bearer为同一不透明会话token，不虚构独立范围APIKey。浏览器mutation带X-RstrBot-Request:1，Origin/Fetch-Site严格检查，不开放带凭据CORS。身份页无需工作区，邮箱、密码恢复、Google显式绑定与多Passkey支持详见HTTP契约；相同邮箱不能自动合并账号。

聊天POST接受异步run与固定幂等键，相同键同参数返回原操作，变参数冲突。保存消息、授权代理快照、检索来源、model/tool往返、流片段、预算及唯一终态。普通文字不是工具调用；当前通用工具循环只开放read_only；副作用审批仍是后续蓝图任务。共享会话只检索共享知识/记忆/工具且仅允许共享代理；用私有代理必须在创建run/消耗幂等键前拒绝。

交互run绑定发起会话租约，注销/过期/改密取消后续调用；计划任务持独立所有者授权并重新校验权限。截止、步骤和重复无进展有界。run状态queued/running/completed/failed/cancelled/interrupted/unknown分别保留，生成、投递、持久化各自状态不混为成功。

知识仅UTF-8文本/Markdown词法检索、可引用原文片段；原文、chunk和版本关联，删除/撤权立即约束检索。记忆由用户显式维护，enabled/archived/pinned不同状态。PDF、音视频、向量、隐式学习不被当前接口暗示为支持。计划支持once/interval/带IANA时区的cron、preview与skip；时区/夏令时场景要实际验证。

## 模型与出站消息

模型提供方选项从已安装且声明model能力的插件枚举。支持`model.list`的插件通过`POST /workspaces/{wid}/models/discover`读取可用模型ID；此发现调用只访问提供方列表接口，不发起生成、不预留RstrBot模型预算，供应商可能记录请求或按其规则计费。临时密钥只传给选中的可信插件且不保存；复用已存密钥前验证工作区、provider、完整原始地址和私网授权完全匹配。权限/会话撤销必须在排队后、握手后和调用期间持续检查；发现失败不得伪装成功，手动模型ID仍可作为回退。提供方地址沿用HTTPS、公网DNS固定解析和禁止重定向；私网只允许实例管理员在实例政策开启时批准。模型测试使用已保存完整配置，输出限制8 token并可能收费；失败不自动重试。模型凭据写入时加密、读取时只返回存在标志；改provider或完整规范化URL必须提供新密钥或明确清除。

Webhook路由默认停用，实例批准、配置保存、启用、发送分开。发送仅接受text/idempotency_key，UTF-8正文最多32768字节，不能覆盖目标/秘密。先持久意图，再受授权派发；排队、握手后、在途检查发起凭据、成员、route版本、enabled及私网政策。已知not_sent与发送后unknown分开，不重放。重复同键保存不同route/text冲突；已有回执GET才是只读查询。相同key重试POST仍可能首次发送，必须讲清并保留原意图。

## 运维和边界

配置/数据目录独立于工作目录，显式便携目录、进程锁和受保护文件。TLS由HTTPS反向代理提供，secure_cookie不建立TLS。正向布尔CLI保存true，省略不自动关掉。check只测数据库/恢复，不测供应商。备份范围和升级schema2不可逆的恢复条件必须清楚。Windows本地worker、恶意代码沙箱、第三方生态兼容和多副本不是现有交付证明。

完整未来需求与缺口由根配方保留；新增端点须更新契约及验收，不用虚构200成功满足蓝图。

## 1.13 可重建的扩展合同

继续读取 [DOMAIN_CONTRACTS.md](DOMAIN_CONTRACTS.md) 的 D01–D09 和 [EXTENSION_API.md](EXTENSION_API.md) 的 E01–E09。身份/服务主体、配置继承、模型路由/预算、写工具审批、知识索引/摘要、计划投递、入站群聊/媒体、子代理/编排和多节点都必须按其输入、输出、归属、状态及恢复边界生成；不能只写“将来插件提供”。所有新增行为当前均未实现/未执行，部分既有能力也需重新验收。

当前 API v1 的唯一完整接口仍是 OPENAPI.md，不修改其既有66+2操作；未来扩展使用独立 `/api/v2`、`rstrbot-http/2-draft` 和封闭操作表。协议的 1.1-draft 能力/可选 broker 通过 `plugins → sdk → protocol` 读取；不靠握手/声明授予业务权。数据扩展需 `storage/DURABILITY_CONTRACT.md` 的 schema 3 迁移及故障门槛。已有本地 RFB 修复的真实预览、私有账号草稿、单语言失败/手动核对合同必须保留。

新增验收为 ACCEPTANCE.md 的 `R13-CORE-001`–`R13-CORE-042`；每项读取实际下游记录/字节/计数，并验证失败、取消、重启、撤权及预算/副作用。上游历史 issue 仅提供回归场景，不能成为本项目或竞品当前通过/失败证据。

## 1.14 模型提供方拒绝

宿主报告的已声明上游拒绝转换成固定安全API分类，HTTP v1仍用网关错误且不能触发本机登出；异步run持久化可选`error_code`和安全消息，重启读回。其他插件及未知码保持通用错误，未知外部效果和费用不能当作零，也不自动重放。
