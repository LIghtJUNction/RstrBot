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

## 1.13扩展验收

合同位置：[EXTENSION_PROTOCOL.md](EXTENSION_PROTOCOL.md)。下表为可执行测试说明，当前全部`not_executed`。每项创建两个工作区A/B、两个主体、独立插件实例及受控下游计数器；先读回核心持久记录，再核对下游实际请求/字节/退出记录。`0`请求是需测断言，不能仅根据错误码推定。无运行时/平台或未实现能力分别记blocked/not_executed。

| 编号 | 输入、操作与故障点 | 下游和恢复断言 |
| --- | --- | --- |
| R13-PROTO-001 | 1.0与1.1-draft分别启动；前者跑原向量，后者显式启用features；再用不支持版本初始化 | 原1.0无回归；扩展不泄入旧进程；不支持时0业务请求、无降版重放 |
| R13-PROTO-002 | 依次伪造nonce、workspace、instance、generation、authority_version；正常握手后切代再发业务 | 均拒绝；旧代无新外发，已存在回执仍可按原ACL读取 |
| R13-PROTO-003 | manifest/handshake/core/配置缺任一host.services；再对七个host方法逐个删除其具体feature；完整授权后注入错误value类型并调用prepare | 缺能力时读取/写意图/外发均0；完整交集后仍核实际资源授权，B不能读A；错类型protocol_error；prepare仅持久proposed且下游发送0 |
| R13-PROTO-004 | 同时发两个w:子请求、错root、重复root终态；根取消后发送迟到帧 | 单挂起限制与终态门成立；句柄失效；进程有界清理，内存不无限增长 |
| R13-PROTO-005 | 授权读取version=asset-v007a的asset字节并核sha256；用数值7替代版本、改工作区/范围，删除源再读缓存 | 内容version逐字保留，数值替代拒绝；仅原范围字节；撤权和删除后0新增读取；输出无宿主绝对路径 |
| R13-PROTO-006 | 两次data.cas使用同旧version；提交后崩溃、同mutation_id重试，另用同ID改值 | 一个提交一个conflict；重启返回原结果，异内容拒绝；无跨namespace写入 |
| R13-PROTO-007 | 下游接受写入后切断回执，随后cancel/重启/重复operation_id | 保留unknown或已知accepted，不自动重发；下游总接受次数1；状态不变not_sent |
| R13-PROTO-008 | 发送中文emoji/转义、safe整数边界和越界、null usage、稀疏工具流/opaque签名 | 四SDK原值一致；越界拒绝；未知费用非0，签名/工具ID未伪造 |
| R13-PROTO-009 | 子服务排队、授权复查后和返回前分别撤权/取消父run；另一同用户会话在线 | 旧租约无后续请求；已知结果仍保存，不能借另一个会话继续 |
| R13-PROTO-010 | poll返回重复event_id和旧generation；持久化后重启；Web查看外部会话 | 一份入站事实/消费游标；来源和目标不被查看入口改写；0重复规划 |
| R13-PROTO-011 | 观察权限默认关闭，显式只开账号A，再撤销且制造队列迟到 | 开启前A/B订阅内容0；开启后只有A获准内容；撤销后无新增广播或模型/投递 |
| R13-PROTO-012 | 模型已返回，终态保存注入storage conflict；同时配置备用模型 | domain=storage；备用模型请求0；保留真实已计费尝试及待核对保存状态 |

通过只表示所测协议/下游条件；不表示所有竞品、外部平台、商业模型或三平台实现通过。

## 1.14 验收

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-PROTOCOL-20 | 交换新上游拒绝码、旧通用错误码和未知码 | 新host正确识别可选分类；旧host不崩溃并通用处理，其他码不误判，消息不含远端响应或秘密 |
