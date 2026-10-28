# rstrbot-plugin-protocol 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-PLUGIN-PROTOCOL-01 | 使用 workspace-a / plugin-a / generation=7 与正确 nonce 初始化，再调用 tools.list | 返回相同上下文、协议 1.0 与声明能力；无授权扩大 |
| RSTRBOT-PLUGIN-PROTOCOL-02 | 分别改变 nonce、workspace_id、instance_id、generation 或协议版本 | 初始化失败；不进入业务方法 |
| RSTRBOT-PLUGIN-PROTOCOL-03 | 发送含中文、换行转义、emoji、引号和反斜杠的 echo 请求 | 内容逐字保留；仍为单行帧；进度和终态关联原 ID |
| RSTRBOT-PLUGIN-PROTOCOL-04 | 发送非法 JSON、批次、超大帧、未知通知、错误 request_id 或双重终态 | 明确协议失败并有界清理，不能当成功 |
| RSTRBOT-PLUGIN-PROTOCOL-05 | 发送未知方法与错误类型参数 | 分别返回 -32601 与 -32602 |
| RSTRBOT-PLUGIN-PROTOCOL-06 | 调用期间取消并模拟远端已接收的写入 | 本地终止；已知回执保留，不确定结果维持 unknown |
