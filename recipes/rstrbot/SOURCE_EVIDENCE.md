# 来源、迁移布局与证据范围

日期：2026-10-28，Asia/Taipei。项目与远端保持RstrBot，https://github.com/LIghtJUNction/RstrBot.git 。源码观察基线main提交 `ead5bf2cea735078e1a37655dcc136485f781b7f`，原工作区包含用户已确认纳入的未提交UI、SDK与文档改动。

## 用户最终确定的布局

RstrBot原仓库以recipes/为主要输入。既有实现、开发缓存及未提交改动完整移动到generated/rstrbot/，按用户指示作为可重新生成的本机产物由Git忽略。未建立观察源码子模块或新迁移仓库。早期额外创建的工作区已按用户要求移入系统回收站；本说明覆盖早期布局选择。

源码旧提交仍在原Git历史；移动前555个已跟踪/未忽略新增文件已与移动后逐文件核对SHA-256相同。generated/保存当前本机增量，但不进入配方提交或归档。没有把现有数据库、密钥、会话或用户数据根当作生成物移动。

## 来源身份

- 原工作区tracked diff的binary格式SHA-256：`5c15b92474b661aa898d0cd719def62c81ea25d9aac2ff621359a793b17efef8`。
- 原555文件集合指纹：`7d43724de7e22db0316e8a3d4c3562d1dd33cc6ff2771f6bc6702e1343a2d7e3`；逐项见SOURCE_HASHES.md，表内路径映射到generated/rstrbot/。
- Recipkg工具使用本机项目当前生成的Python参考CLI及当前recipes/模板；该工具源码不属于RstrBot配方依赖。初期工具源目录在并行迁移中调整为generated/（实际CLI源码为generated/src/），最终按新版指南与CLI校验。
- MIT许可保留LICENSE.md，外部依赖各自许可，未复制第三方视觉代码/资产。

## 观察与推断

| 类型 | 材料 | 支持的结论/限制 |
| --- | --- | --- |
| 直接Git/文件检查 | HEAD、origin/main、status、工具链/Cargo、LICENSE、移动前后文件摘要 | 来源及本地增量完整保留，不是运行验收 |
| 直接源码段查看 | main CLI、api router、store身份/错误类型、插件帧及超时定义；此前会话中UI组件/样式与表单 | 入口、公开类型、guard及UI增量；未逐函数全面审计 |
| 声明式接口 | OpenAPI、SDK共同向量 | 65API＋2健康操作/DTO/输入约束与协议样例；示例不是运行回执 |
| 文档事实 | README、API/插件协议、认证/运维/桌面/UI/教程/技能 | 公开语义、信任边界与缺口，历史描述按CURRENT_SCOPE处理 |
| 原需求 | 基线1.4、52项requirements、215族/287验收映射 | 原目标完整保留；not_started等属于设计时点 |
| 测试定义/旧证据 | tests各HTTP/权限/恢复/浏览器入口、docs/evidence历史说明 | 提炼未来场景；本次迁移未重新跑原软件测试，不继承通过数量 |
| 作者选择 | 八包依赖分工、分阶段重建方案 | 组织合同，不是新增已实现能力 |
| 未知 | 性能硬指标、最低OS/架构矩阵、实体设备/真实服务/原生桌面 | 未验证，按ENVIRONMENT/PERFORMANCE及各包验收记录 |

用户语言选择保持Rust核心、TypeScript前端、四语言SDK，辅助工具可选；性能暂无数字预算；此前已确认中英文、双导航/移动体验、开发教程/插件技能与每次需求更新构建提示词继续有效。

## 迁移前工作区状态

以下为移动前状态，不是迁移后根仓库git status；本地增量在generated/rstrbot/保留。

```text
 M .github/workflows/ci.yml
 M README.md
 M docs/ACCEPTANCE.md
 M docs/PLUGIN_PROTOCOL.md
 M docs/blueprint/BUILD_PROMPT.md
 M docs/blueprint/SHA256SUMS
 M docs/blueprint/manifest.json
 M mise.toml
 M scripts/check.sh
 M scripts/package.py
 M scripts/verify_package.py
 M sdk/csharp/EchoPlugin/EchoPlugin.csproj
 M sdk/csharp/RstrBot.Plugin/RstrBot.Plugin.csproj
 M tests/plugin_conformance.py
 M web/src/App.tsx
 M web/src/Chat.tsx
 M web/src/Delivery.tsx
 M web/src/Resources.tsx
 M web/src/Sculpture.tsx
 M web/src/style.css
?? .agents/skills/rstrbot-development/SKILL.md
?? .agents/skills/rstrbot-plugin-development/SKILL.md
?? docs/DEVELOPER_TUTORIALS.md
?? docs/blueprint/history/BUILD_PROMPT-v1.3.md
?? web/src/BuildPromptEditor.tsx
?? web/src/DeveloperTutorials.tsx
```

## 本次验证边界

验证配方、递归依赖、归档和完整提示词；检查移动前后文件内容一致。为了保持用户先前打开的本地预览，使用新路径重启该预览并检查HTTP可访问；这不是独立重建或功能回归验收。未重跑原应用测试、未触发CI、未发送商业模型请求或外部消息。检查回执在工作区artifacts/VALIDATION.md，最终软件是否实现仍须另行重建和验收。
