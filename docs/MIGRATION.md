# RstrBot 配方迁移校验 · 2026-10-28

项目与远端保持 **RstrBot / LIghtJUNction/RstrBot**。现有仓库以 recipes/ 为主要输入，既有实现及未提交改动完整移到受Git忽略的 generated/rstrbot/。用户撤销的额外迁移工作区已移入系统回收站，不存在同名后缀项目路径。没有创建新的远端仓库。

## 实际完成的检查

| 检查 | 本次结果 | 范围 |
| --- | --- | --- |
| 八个配方各自recipkg check | 全部退出0 | 当前Recipkg生成的Python CLI，离线递归校验 |
| 递归pack、归档check、中文build提示词 | 全部退出0 | 包版本0.2.0-recipe.1，完整八包上下文 |
| 归档对象/路径/编码 | 66个普通UTF-8 .md，无NUL、文件系统链接、源码/二进制/钩子 | 独立读取tar成员检查 |
| LOCK完整性 | 所有65个负载文件摘要匹配，锁清单无缺项 | 八个包身份齐全 |
| 完整提示词内容 | 65份负载Markdown原文全部包含 | pack改写后的manifest一并包含；LOCK为完整性元数据 |
| 需求保留 | 52项需求、215族（含R和B前缀）、287条原验收身份与顺序一致 | 52目标/验收逐文本核对；原始追踪和快照完整保留 |
| HTTP声明 | 原OpenAPI JSON完整对象相等 | 65API操作＋2健康操作，所有DTO/示例/约束 |
| 本地文档链接 | 配方内相对链接全部存在 | 外部链接是来源引用，不作为缺失依赖 |
| 原实现保留 | 555个来源文件移动前后SHA-256相同 | 原工作区快照7d43724de7e22db0316e8a3d4c3562d1dd33cc6ff2771f6bc6702e1343a2d7e3 |
| 本地预览 | 新源码路径启动后HTTP200 | 127.0.0.1:8788，业务数据根没有移动；不是功能验收 |
| GitHub发现标签 | 已添加并通过REST回读recipkg | 仓库名称和原远端未改 |

## 临时校验文件（已清理）

- 迁移检查期间生成过配方归档（333298字节，SHA-256 `d83b155f539a7356044dc643cbdb5457a7d34bd9581c28a15e559ffc788a8b44`）和完整build提示词（1391671字节，SHA-256 `733cefa3a21638aeaf1ec63fe9cc07c851b431020ad1aa6ef2822e7b32b4b7e1`）。
- 另有中文观察清单、配方检查与归档检查JSON、校验摘要及迁移前源码状态快照。
- 这些都是本机临时文件，已按用户要求移入系统回收站；当前仓库不保留固定产物目录。上面的检查结论记录在本文件中，配方内容仍可重新打包。

配方本身位于recipes/，所有递归输入可独立打包。重新执行pack/build时，把输出写到仓库外的系统临时目录。

## 过程中的失败与修正

早期检查引用的Recipkg源码路径在该项目并行调整布局后失效，返回No module named recipkg；最终定位到它的generated/src/，重新执行全部八包及归档检查通过。第一次需求计数只识别R前缀漏算26个B前缀；最终改为与原始215个ID集合及顺序逐一比对，通过。GitHub GraphQL标签回读曾EOF，后通过REST确认，没有重复修改标签。

## 开发技能独立仓库

核对[Recipkg当前上游迁移指南](https://github.com/recipkg/recipkg/blob/main/recipes/recipkg/skills/migrate-to-recipkg/SKILL.md)后，没有发现“开发技能必须作为子模块”的要求；指南说明的是源码工作区可采用子模块布局。本项目按用户要求为两个技能各建一个公开仓库：[`rstrbot-development`](https://github.com/LIghtJUNction/rstrbot-development)和[`rstrbot-plugin-development`](https://github.com/LIghtJUNction/rstrbot-plugin-development)，并固定为主仓库`.agents/skills/`下的Git子模块。

主仓库`./scripts/sync-skills.sh`把两个子模块的`SKILL.md`同步到`recipes/rstrbot/skills/`，并更新[SOURCES.md](../recipes/rstrbot/skills/SOURCES.md)中的完整提交号。这样技能能独立迭代，Recipkg配方仍以普通Markdown快照完整交付，不依赖打包器解析Git子模块。脚本要求子模块工作区干净，避免把未提交修改误记为固定提交内容。Recipkg CLI的`skills sync`是将已安装配方里的技能安装到Agent目录的分发功能，不负责推进这些Git版本。

拆分后的本地校验：同步脚本语法检查与执行通过，两个快照逐字匹配子模块；根配方`check`、递归`pack`、归档`check`及中文`build`提示词生成全部通过。临时归档和提示词位于系统临时目录，校验后自动清理。

## 未执行与边界

本次未独立重建RstrBot，未重新运行原软件测试、未测性能、未验证原生安装/签名/升级、读屏/实体设备或商业模型/真实邮件/聊天平台。未触发CI。当前源版本是 `ead5bf2cea735078e1a37655dcc136485f781b7f` 加迁移前本地增量；旧已提交源码留在Git历史，本地增量连同全部原实现保存在generated/rstrbot/。性能数字保持未知，未来实现按配方验收。

来源、范围、推断与未完成蓝图见recipes/rstrbot/SOURCE_EVIDENCE.md和CURRENT_SCOPE.md。打包通过证明材料可交付，不证明未来重建已经成功。
