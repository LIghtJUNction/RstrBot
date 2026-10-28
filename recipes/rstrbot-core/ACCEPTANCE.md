# rstrbot-core 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-CORE-01 | 临时SQLite启动，使用终端bootstrap凭据建立账户，再重复/并发setup | 仅有效证明一次成功；产生登录会话，15分钟过期或已初始化后拒绝 |
| RSTRBOT-CORE-02 | 不匹配Origin、跨站Fetch-Site、缺浏览器mutation头、有效/无效Bearer | 按OpenAPI拒绝越界；Bearer不绕过Origin校验 |
| RSTRBOT-CORE-03 | 遍历OPENAPI.md全部操作，以合法与非法DTO调用 | 注册路径/方法、200/204及错误封套、必填和UTF-8边界一致 |
| RSTRBOT-CORE-04 | 配置diagnostic模型并启用工具实例，发送/diagnostic/tool hello | 真实worker发现echo、调用后续轮、消息/事件/终态持久化 |
| RSTRBOT-CORE-05 | 真实本地模型夹具流出片段后断开/重启核心 | 已落库片段可恢复，run interrupted，重启不再调用上游 |
| RSTRBOT-CORE-06 | 重复chat幂等键同参数/变正文；共享会话尝试私有代理 | 重复不多发，变参数冲突；私有代理原子拒绝无run/键消费 |
| RSTRBOT-CORE-07 | 调用中注销、改密、到期或移出工作区 | 租约失效后不启动下一模型/工具；已知使用仍保留 |
| RSTRBOT-CORE-08 | 模型完整URL路径或provider变更，不给新密钥/clear | 400拒绝；明确清除后保存，原秘密不转发新目标 |
| RSTRBOT-CORE-09 | 普通成员启用私网，或实例政策禁用后调用原已获准私网模型 | 授权拒绝；阻塞调用不预留新费用 |
| RSTRBOT-CORE-10 | 模型测试在本地上游记录请求 | 实际调用保存配置，最多8输出token，usage缺失为未知，无自动重试 |
| RSTRBOT-CORE-11 | 知识含Unicode与HTML文本；私有/共享检索、删除、归档记忆 | 引用准确、显示安全、ACL先过滤；归档与删除不混淆 |
| RSTRBOT-CORE-12 | 一次/间隔/时区cron任务preview与执行，执行中所有者降权 | preview无派发；正确认领与skip，重新核权 |
| RSTRBOT-CORE-13 | 路由保存后直接发送，批准启用后发送；UTF-8恰好/超过32768字节 | 未启用拒绝；启用后最多一次派发；超限验证失败 |
| RSTRBOT-CORE-14 | 第9个Webhook等待槽位时注销发起会话，另一个同用户会话仍有效 | 旧凭据不被新凭据续命，派发前拒绝并保存真实状态 |
| RSTRBOT-CORE-15 | Webhook在目标接收后断线，再重复POST同key和查询receipt | unknown不伪造成未发送；不创建第二投递；GET只读 |
| RSTRBOT-CORE-16 | 数据库连接只读探测、无关schema、Valkey中断 | 探测不迁移、不覆盖；可选缓存degraded不导致授权放宽 |
| RSTRBOT-CORE-17 | 密码/邮箱/Google/Passkey一次性挑战和双凭据换绑 | 不重复消费；新凭据验证成功前保留旧凭据；稳定sub显式绑定 |
| RSTRBOT-CORE-18 | 数据库备份恢复及recovery/complete；非空目标和旧版本回滚尝试 | 按恢复边界拒绝危险覆盖；暂停外部执行，不重放未知操作 |
