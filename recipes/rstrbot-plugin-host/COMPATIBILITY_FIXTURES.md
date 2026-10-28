# 首批真实兼容样本冻结清单

核对日期：2026-10-28（Asia/Taipei）。本清单是为R13-HOST-023/024选定的**官方公开、未修改候选样本**；选择已完成，不需要用户重新决定普通只读研究事项。已读取精确commit的文件，计算UTF-8字节SHA-256，并与Git blob身份核对。只记录引用、摘要和验收，不把第三方源码/许可证正文复制进配方。

三项统一状态：`source_status=frozen_read_verified`；`imported/behavior_verified/recovery_verified/cleanup_verified=not_executed`。冻结来源不是兼容已通过；没有调用任何真实模型、聊天平台或安装样本。后续按这些固定输入生成加载器，并以 [ADAPTERS_AND_COMPATIBILITY.md](ADAPTERS_AND_COMPATIBILITY.md) A-05的五阶段结果登记。

<a id="fixture-astrbot"></a>
## FX-AST-01 AstrBot官方指向的helloworld模板

- 来源：[AstrBot官方开发入口](https://docs.astrbot.app/dev/star/plugin-new.html)指向 `Soulter/helloworld`。
- 样本仓库：`Soulter/helloworld`；精确commit：`0c0d52b17e2feb76a7bcff887cac3d0e12eecc09`。
- 未修改样本：[main.py](https://github.com/Soulter/helloworld/blob/0c0d52b17e2feb76a7bcff887cac3d0e12eecc09/main.py)，1413字节；SHA-256 `a88ddc03117bf020c1eaa8797ab953618cbcd0c23fd767446f605dd5d2d46a52`；Git blob `e6af67e18e1fb8cb8cc43491fd58df7c54b88b7e`。
- 元数据：[metadata.yaml](https://github.com/Soulter/helloworld/blob/0c0d52b17e2feb76a7bcff887cac3d0e12eecc09/metadata.yaml)，442字节；SHA-256 `31b9b579796ca005d44f2ab461e403c81b4102f80d4a8762d335a9f981fc9f02`。元数据version为v1.3.0，源码register标注1.0.0，必须原样保留这两个观测值并在适配诊断中区分包元数据与注册元数据，不能悄悄修样本使其通过。
- 选定运行/SDK基线：`AstrBotDevs/AstrBot` v4.29.3、commit `f61a7f42696da6df25dc7a249ec9b8d54ed9da18`；[pyproject.toml](https://github.com/AstrBotDevs/AstrBot/blob/f61a7f42696da6df25dc7a249ec9b8d54ed9da18/pyproject.toml)声明Python≥3.12，SHA-256 `d1e7d597c1e1fe059cbc09df85ee96206e7c178d567a3fd6b638fac78dc1c59f`。这只是固定候选配对，尚未证明模板对该版本或RstrBot加载器兼容。
- 许可证来源：[模板LICENSE](https://github.com/Soulter/helloworld/blob/0c0d52b17e2feb76a7bcff887cac3d0e12eecc09/LICENSE)，GNU AGPL v3文本，34531字节，SHA-256 `f440d399276dcbd8a845dee6e873a54c1f59ff44b502b45f3bc9352224ed5789`。不把模板标为本项目MIT，不随本配方分发源码；任何将来的捆绑必须按实际完整许可处理。

验收操作：保持样本源码和元数据摘要不变，在隔离Python环境加载Star/Context/装饰器兼容层，合成sender_name=Recipe Tester，使用受控入站适配器向私有会话投递`/helloworld abc`。核对实际yield结果包含原sender和原消息正文、仅绑定该会话产生一条出站意图，接收器实际一次接受。随后撤权再投同事件、重复event_id、禁用/卸载，核对无新发送/残留命令，其他工作区零事件。initialize/terminate存在但函数体不做业务动作，不能用其“存在”证明复杂初始化或清理已通过；必须结合宿主资源/注册表检查。

覆盖：发现/元数据、未修改Python导入、Star构造、命令装饰器、发送者/消息链读取、文本结果、基本停用卸载入口。**尚不覆盖**旧LLM前后hook、多轮工具、真正媒体字节、旧内部对象复杂代理、跨会话持久后台推送、收费供应商和真实平台账号；这些是R13-HOST-023的具体剩余样本集，不是用户必须重新选择的事项。

<a id="fixture-opencode"></a>
## FX-OCODE-01 OpenCode官方历史自定义工具样例

- 仓库：`anomalyco/opencode`；精确commit：`0fd73a29760536bfcf9ead0832a1cf3c29612022`。
- 未修改样本：[packages/plugin/src/example.ts](https://github.com/anomalyco/opencode/blob/0fd73a29760536bfcf9ead0832a1cf3c29612022/packages/plugin/src/example.ts)，402字节；SHA-256 `505311957fefbf936e259045ac44a5e7ca4b45d3df2c0dd5acd7d5f51c0c85b4`；Git blob `bf79ab511e404bc04d387a0cd88b2be8c9a98e6f`。
- 精确SDK/profile：[packages/plugin/package.json](https://github.com/anomalyco/opencode/blob/0fd73a29760536bfcf9ead0832a1cf3c29612022/packages/plugin/package.json)，源码包名`@opencode-ai/plugin`、版本1.18.4、引用`@opencode-ai/sdk` 1.18.5；1606字节，SHA-256 `8e47e8c8fd7ffe63328498c851f1a62bb28282b4a7a380b0ef0c41d2078c4a1b`。不把源码manifest中的版本当作已经核验的npm发布包。
- 许可证：[LICENSE](https://github.com/anomalyco/opencode/blob/0fd73a29760536bfcf9ead0832a1cf3c29612022/LICENSE)，MIT；1065字节，SHA-256 `625f0f619133f89bbbb2abe37369613dfa1885eba1e50d02170deb62bb42cb6b`。
- 选择边界：通过该路径提交历史定位到仍含真实样本的commit；其后提交`8db7487c894b46d8093135a18f5cf23c8447a4e9`重构了工具架构。核对时当前主干commit `b94a30547f7f677b9b9a7e36d7c66ca1ac14787d`的[插件manifest](https://github.com/anomalyco/opencode/blob/b94a30547f7f677b9b9a7e36d7c66ca1ac14787d/packages/plugin/package.json)已为`@opencode/plugin` 2.2.0，故本样本明确是**1.18.4旧profile**，不承诺当前2.2.0或所有历史版本兼容。

验收操作：保留整个固定checkout的相对模块结构，不改样本`./index.js`/`./tool.js`导入；通过对应构建/兼容模块解析器加载其**命名导出ExamplePlugin**，不能假设只有default。从实际工具目录发现mytool，提交foo="recipe"，核对真实返回文本及调用计数；number类型foo应在handler前按schema拒绝。对两个workspace加载、停用、重启、重复调用意图分别测隔离和注册回收。编译生成物可改变形式，源样本hash前后必须一致。没有执行runtime依赖安装；将来运行须冻结实际解析锁文件与工具链，不使用浮动latest。

覆盖：真实未修改工厂导出、相对模块导入、schema注册、字符串参数验证、只读工具结果。**尚不覆盖**旧event/compaction hook、Bun shell授权、会话状态、媒体、长任务、UI以及2.2.0新profile；R13-HOST-024必须按这些子范围显示未覆盖，不能把此工具返回成功扩展为OpenCode生态全支持。

<a id="fixture-pi"></a>
## FX-PI-01 Pi官方hello工具扩展

- 仓库：`earendil-works/pi`；精确commit：`c3d8ce44d3231c7e46071d2d859f41ca7d39d21b`。旧`badlogic/pi-mono`来源重定向仅用于识别，不混用新旧SDK包名。
- 未修改样本：[packages/coding-agent/examples/extensions/hello.ts](https://github.com/earendil-works/pi/blob/c3d8ce44d3231c7e46071d2d859f41ca7d39d21b/packages/coding-agent/examples/extensions/hello.ts)，640字节；SHA-256 `0aa4e9800c2526914d4c1edb00b2cfa9bd9dd6da5218289994cccd5f5bfa4934`；Git blob `52da3910bdc3cb316ced113dc1c95a2dd6b5f7cb`。
- SDK/profile：[packages/coding-agent/package.json](https://github.com/earendil-works/pi/blob/c3d8ce44d3231c7e46071d2d859f41ca7d39d21b/packages/coding-agent/package.json)，`@earendil-works/pi-coding-agent` 1.4.0、`@earendil-works/pi-ai` 1.4.0，声明Node≥22.19.0；3873字节，SHA-256 `f9c21f2fe252a5f988a19f6e4097bb058c411cb98e2eb60e39f85e88c54f05c9`。
- 许可证：[LICENSE](https://github.com/earendil-works/pi/blob/c3d8ce44d3231c7e46071d2d859f41ca7d39d21b/LICENSE)，MIT；1069字节，SHA-256 `0457f5bcec3b3b211605dfb5d1a49042fd638f3686a410fe099c24a25af13c48`。

验收操作：固定checkout构建的SDK供加载器使用，原样加载default扩展函数，确认defineTool→registerTool真实注册hello；用name="Recipe Tester"调用，核对content文本与details.greeted均保留同一值，错误类型在调用前拒绝。两个workspace分别调用，禁用/撤权/卸载后真实目录和调用权更新，重启后按持久启用配置恢复一次注册。该例execute没有长等待和外部动作，不能据此宣布取消传播、外部副作用幂等或多轮Agent通过；这些另跑受控合同夹具与后续真实样本。

覆盖：真实未修改TS导入/default导出、defineTool/registerTool、结构schema、content和details双结果。**尚不覆盖**registerCommand、on生命周期、appendEntry持久数据、自定义TUI、MCP/provider、媒体、子代理和跨版本旧包名。R13-HOST-024按子行为记录，不能把hello工具成功扩展为Pi完整兼容。

<a id="fixture-procedure"></a>
## 固定来源的执行与证据门槛

1. 按上述完整commit取得样本及其上下文，核对路径、字节数、SHA-256和许可证来源；不从其他branch补同名文件，不修改样本迎合加载器。来源读取失败记blocked/source_unavailable，不当样本行为失败。
2. 只在隔离的临时实现环境准备精确运行时/依赖；记录实际命令与解析后的锁文件摘要。下载/构建行为不进入配方依赖图，不改变八包闭包，不把本次源码读取带入最终ZIP。
3. 依次在原生态固定profile与RstrBot对应加载器执行同一输入，保存detected/imported/behavior/recovery/cleanup逐项结果、输出摘要和下游计数。上游能跑但RstrBot失败是兼容缺口；上游环境也未启动则保持blocked/not_executed，不推定任何一方优劣。
4. 新样本只填补上述明确的未覆盖集。可以由实施者自行选择官方或许可明确的公开样本并按同一冻结格式增补，无需把常规样本选择升级为用户决定；用户只需决定真正影响授权/费用/目的地/分发的事项。

本轮已完成来源与摘要校验，尚未进行上述运行步骤。实际兼容完成度仍为未实现/未执行；这与“样本已经固定”是两个独立事实。
