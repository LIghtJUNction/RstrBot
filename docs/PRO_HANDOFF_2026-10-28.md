# RstrBot配方完成 · Pro交接

任务ID：20261028-rstrbot-recipe-completion；输入revision：r2；状态：recipe_applied / recipe_validated（2026-10-28 13:23 UTC）。

用户要求将当前配方交给Pro完成，新增真实端到端对比AstrBot、MainBot、OpenClaw并将OpenClaw issue转配方。MainBot暂为MaiBot候选，准确仓库仍待确认；不阻断其余工作。

## 交接与材料

- [Pro会话](https://chatgpt.com/c/6ae1e5ba-4d1c-83e9-8b2e-b61a88313a36)：新Chat会话，已核实Pro显示并确认消息实际发送，页面已显示读取Drive材料的处理说明；保留为handoff。
- [私有Drive任务目录](https://drive.google.com/drive/folders/1u5OUFcn8-19XjpdFU9ps5TXXuNtwsMTv)：9文件，00简报、10上下文、11当前合同、12完整build提示词、13递归归档、14输入摘要、20请求、30完整回传、99状态。无分享权限变更。
- [20-request.md](https://drive.google.com/file/d/15ttG1qSNKvADNoxFHsiCLq3JoYVYzhB1/view)：完整交付要求；发送时revision为`0B_SvJwjiD4TMTUtKWWw5VUtlUTNzeTEzeWdhS1h0SHROdGtrPQ`。
- [30-response.md](https://drive.google.com/file/d/12QypSiiXGhxZgcBF2yLf-QWvQbGahojx/view)：已收到完整原文并保留独立版本；原文与ZIP的50份文件逐字校验一致后合入，新增软件尚未运行。

输入是HEAD `25c36e26ab1a0abe12393bd0480a373cf801c714`加当前未提交配方，不是仅远端源码。根提示词1.12/根包0.2.0-recipe.6、Web .5、core .2；保留52需求、215族、287原验收以及八包和历史。新增COMPETITIVE_ACCEPTANCE/OPENCLAW_FEEDBACK与RSTRBOT-19/20，旧1.11原文另存history。

## 输入r2验证与限制

00简报回读逐字相等；9文件远端字节数均与本地一致，保持not_shared。13归档从Drive重新下载后与原输入逐字节相等，357970字节，SHA-256 `dcd077e452f02b7da0f97ac1f50639d6810df96636d047e062050bfb0ca247a4`；下载归档解包后八个软件包均通过recipkg check。中文build与git diff --check通过。

本次只更改配方/文档与授权的私有Pro交接，没有新增软件实现、跨产品实测或发布。当前生成应用仍为上一轮1.11修复产物，34项Web及11+7浏览器证据不得外推成竞品胜出。5条OpenClaw issue均closed，原文/关键评论/状态已审阅，但未在本项目复现；作为待执行回归合同。

## 回传合并准则

检查返回文件路径/UTF-8/普通Markdown、依赖闭包、接口及版本/现有需求保留和真实未知；无需机械采纳每条建议。完整原文保存在30回传，具体改动另留可审阅差异。合并后同步提示词/组件/ACCEPTANCE/CHANGELOG并重做相关Recipkg检查；涉及实现时另行本地真实验收。任何等待、下载或打包均不能记作完成配方或全面超越。

## Pro回传与本地合并结果

Pro已完成根1.13/根包.7、Web.6、core.3与其他五包.2的47文件增量（30修改/17新增），202个新增验收ID。完整原文593620字节，SHA-256 `ac80692cd8af91a107c3342a284d0d0cc27153460a7b4c0007a57291a342aab1`，Drive revision `0B_SvJwjiD4TMM0tNMWFTcytRUk1yeGZzQWc4Tm9HS3VsZklnPQ`。ZIP 266443字节，SHA-256 `6cc7c616641a0696d530cd764420d83213ba32e642a0da7beba00ebc7cf823c6`，文件ID `1ETTogHkmGKF59pygAGJf19cOuV2_PJIU`。没有改写Pro原文。

本地先核72份输入与47项前后摘要，再应用全部文件。补正随包技能依赖尚未生成文档的问题，同步实际项目技能，澄清既有授权持续有效和验收ID简称，更新README中旧归档文件名；这些Codex复核变化单独写入根CHANGELOG/SOURCE_EVIDENCE。39份输入文件仍逐字不变，另有明确修改的来源说明和两份随包技能；当前OpenAPI、协议/向量、LICENSE、原需求/追踪和旧历史均未改写，1.12全文精确保留。

| 本地检查 | 结果 |
| --- | --- |
| 8个作者包Recipkg check | 全通过 |
| pack→归档check→unpack→解包入口check→中文build | 全通过 |
| 现行本地文档链接 | 90项存在，含合法目录链接；历史按原包上下文保留 |
| 52/215/287、旧接口/协议/历史与新增ID | 保留；202个唯一新ID，包含非数字后缀R13-CMP-GATE |
| 配方归档 | 89负载Markdown+LOCK，共90个普通UTF-8 Markdown；无源码/链接/执行钩子 |
| 三份固定兼容样本 | 独立来源读取摘要一致，仅证明源身份，没有执行样本 |

最终配方归档528816字节，SHA-256 `3f4532c136867c487a5069a68677814332fd44df1e88492d00fc38ed8e9ac694`；中文完整提示词1917139字节，SHA-256 `bd73dec12578302a2f967857689e5deab6aae9e6b138995a615104d7ee8b69f7`。临时验证路径为`/tmp/rstrbot-recipe-1.13-validation-35t1lywi`，没有仓库内固定产物目录。Recipkg入口为现有uv工具环境，入口脚本SHA-256 `5110a7490286ff4a4d7c8771b081787b20787d47af1bd36e6d7e74432759d003`。校验器最初只接受数字ID及文件链接，修正了这两项误判后完成；不是软件故障。

OpenClaw首批5条closed仍是历史回归，另核177909/186700两条正文和全部评论、177956实际合并元数据及固定WhatsApp授权文档；不将open/closed或上游测试报告冒作本轮实机。竞品候选release由Pro冻结，MainBot仍未确认，全部跨产品not_executed。源码生成应用仍为1.11，本轮未重跑其34+11+7，也没有验证新增软件、设备、商业平台或部署。
