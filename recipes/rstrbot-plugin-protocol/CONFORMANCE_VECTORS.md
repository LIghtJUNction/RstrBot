# 共同声明式协议向量

来自sdk/conformance.json；仅为输入/输出样例，原harness另含非法初始化凭据检查。

```json
{
  "protocol_version": "1.0",
  "scope": "Authenticated Unix-socket handshake, method dispatch, errors and streaming events; this is not legacy-ecosystem compatibility certification.",
  "context": {"workspace_id":"workspace-a","instance_id":"plugin-a","generation":7},
  "cases": [
    {"name":"tool discovery","method":"tools.list","params":{},"expect_tool":"echo"},
    {"name":"unicode and structured framing","method":"tools.call","params":{"name":"echo","arguments":{"text":"你好\n🙂 quote: \" and slash: \\"}},"expect_content":"你好\n🙂 quote: \" and slash: \\","expect_event":"tool.progress"},
    {"name":"unknown method","method":"tools.unknown","params":{},"expect_error":-32601},
    {"name":"invalid argument","method":"tools.call","params":{"name":"echo","arguments":{"text":12}},"expect_error":-32602}
  ]
}
```
