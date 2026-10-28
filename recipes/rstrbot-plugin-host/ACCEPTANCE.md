# rstrbot-plugin-host 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-PLUGIN-HOST-01 | 校验正确与错误摘要的本地 manifest，模拟缺失可执行文件 | 错误分类清楚，校验失败不启动进程 |
| RSTRBOT-PLUGIN-HOST-02 | 9 个等待调用占据 8 槽并让第9个超时；队列期间撤权 | 槽位有界；超时/撤权调用不派发业务请求 |
| RSTRBOT-PLUGIN-HOST-03 | 握手成功后撤权、禁用实例或更改 generation | 旧调用拒绝继续；新调用检查当前权限 |
| RSTRBOT-PLUGIN-HOST-04 | 取消等待进程及其子进程，并丢弃宿主调用 future | 进程组回收、直接子进程被 reap、临时目录清理 |
| RSTRBOT-PLUGIN-HOST-05 | 模拟慢通知消费者、超大结果和流末尾缺失 | 明确 slow_consumer/协议错误，内存有界，不静默成功 |
| RSTRBOT-PLUGIN-HOST-06 | 本地 OpenAI 测试服务返回稀疏工具片段与缺 usage | ID/片段顺序和 opaque state 保留，usage 与成本未知 |
| RSTRBOT-PLUGIN-HOST-07 | 发送到公开HTTPS、重定向、私网和DNS重新绑定情形 | 默认策略拒绝越界；私网须双重授权，TLS失败不降级 |
| RSTRBOT-PLUGIN-HOST-08 | MCP 固定目标发送一次，再用同ID重复、改正文、重启后重复 | 一次真实接收；同参数读回执，改参数冲突，不重放未知意图 |
| RSTRBOT-PLUGIN-HOST-09 | Webhook目标返回2xx、4xx、断线，并注入回执持久化失败 | accepted/rejected/unknown及persistence分离；无自动重试 |
