# rstrbot-web 验收

以下是未来重建必须执行的验收方案。本次迁移未执行这些软件场景。每项使用独立临时数据与测试账号，记录平台和结果。

| 编号 | 输入与操作 | 预期结果 |
| --- | --- | --- |
| RSTRBOT-WEB-01 | 深浅主题、中文/英文分别在320/390/768/1024/1440px打开主要页面 | 无整页横向溢出，当前页面和主操作清楚，文本不双语堆叠 |
| RSTRBOT-WEB-02 | 两种导航间切换/刷新，抽屉选择页面，用户降权后刷新成员资格 | 布局偏好保持、内容和权限一致，失权页面与入口撤销 |
| RSTRBOT-WEB-03 | 侧栏与会话列表手动折叠/展开，向下/向上滚动、轻微抖动 | 控件可恢复，阈值迟滞有效，不改变阅读锚点 |
| RSTRBOT-WEB-04 | 输入/搜索/选择框聚焦，打开菜单/弹窗后滚动 | 导航不突然收走正在操作的控件，焦点和滚动保留 |
| RSTRBOT-WEB-05 | 导航/编辑器只用键盘；中文IME确认和Enter/Shift+Enter | 焦点可见可达、Escape关闭弹窗并回焦点；IME不误发送 |
| RSTRBOT-WEB-06 | 慢请求期间切工作区或用户，再返回；发送确认晚到时另输入草稿 | 旧读不污染新上下文，不抹掉新草稿，无私有缓存泄露 |
| RSTRBOT-WEB-07 | 重放乱序delta/terminal、坏历史与HTML样文本、历史前向分页 | 终态不复活，坏项不拖垮好项，HTML不执行，ID去重且锚点不跳 |
| RSTRBOT-WEB-08 | 创建超过200资源并继续加载，尝试标题搜索 | 可达全部分页，搜索说明只覆盖已加载项，不谎称全库搜索 |
| RSTRBOT-WEB-09 | 进入模型向导，枚举已安装的model能力provider并改变provider/完整URL，留空秘密或明确清除 | provider列表来自真实安装数据，不硬编码关闭；地址校验清楚，秘密不回显、不误转发 |
| RSTRBOT-WEB-10 | 模型测试和Webhook发送/继续原请求/新投递确认 | 明确真实请求和费用；无伪测试、无自动重复发送或把POST叫查询 |
| RSTRBOT-WEB-11 | 日期、cron时区、费用、token和中文正文临界输入 | 枚举可选项，自定义格式明确；UTF-8与整数约束跟随API |
| RSTRBOT-WEB-12 | 保存冲突、401、403、网络未知结果、加载和空列表 | 各有准确说明与恢复入口，不把拒绝/未知显示成功 |
| RSTRBOT-WEB-13 | 打开教程、展开起步步骤及SDK/插件技能/文档链接 | 教程在页内可读，链接指向交付文档；未知能力不伪装支持 |
| RSTRBOT-WEB-14 | 修改提示词、保存两版、比较差异、恢复、导出、刷新 | 本地版本最多20份、有时间与差异，恢复正确，不冒称已写回仓库 |
| RSTRBOT-WEB-15 | 暂停粒子、开启prefers-reduced-motion、切后台/离屏，触摸滚动缩放 | 静态替代、离屏暂停、无焦点/滚动覆盖，不把装饰伪作遥测 |
| RSTRBOT-WEB-16 | 对支持`model.list`的真实测试provider执行发现，再用不支持发现的插件、空列表及失败响应配置模型 | 建议来自核心发现API；加载/错误/空/截断状态准确，最多500项；发现只列模型、不生成或占预算；手填ID始终可用；临时密钥不落持久草稿，换目标不复用已存密钥 |
| RSTRBOT-WEB-17 | 延迟模型发现，修改连接字段、取消、关闭重开并放行迟到成功/失败 | 浏览器请求取消，旧结果不回填新目标；无自动重试，取消说明不冒称供应商撤销 |
| RSTRBOT-WEB-18 | 模型/渠道逐步输入非法URL、超长UTF-8名称、旧密钥换目标、非法token和单价，用键盘Enter提交 | 当前步骤显示并聚焦字段错误，label和说明可访问；核对前无保存，最终请求沿用同一验证规则；保存期间无重复提交 |
| RSTRBOT-WEB-19 | 在有根配方和独立源码包两种布局构建并打开提示词编辑器 | 前者显示当前根配方版本和内容，后者使用随包快照；旧浏览器草稿不被自动覆盖，历史1.4保留 |
| RSTRBOT-WEB-20 | A保存本机提示词/版本并导出→退出→B登录，再A重登；旧标签页/迟到回调、另一实例来源、旧全局键及存储拒绝 | 草稿/历史/差异/恢复/导出按实例来源和已认证用户隔离；B无A哨兵，A版本保留且最多20份；无身份不显示，旧键不自动归属或静默删除，降级说明准确 |
| RSTRBOT-WEB-21 | Tokyo搜索并选Asia/Tokyo、每日09:00与自定义cron/IANA；调用真实preview，改字段/身份后旧响应返回；非法/缺模型会话及DST区域 | 选项有真实来源、开放输入可用；未来时间显示计划与设备时区/暂停状态，台北08:00与东京09:00对应；匹配服务端DST/skip策略且无保存/派发/模型调用；错误关联字段保留草稿，旧响应不回填 |
| RSTRBOT-WEB-22 | 中英分别对新/既有会话离线发送，恢复网络后核对/手动重试；服务端接受但客户端失联；晚确认前另写新稿；401/403 | 当前语言给连接检查/结果核对/手动继续说明并保留原文，技术错误仅次要详情；不自动发送、不将未知说成未发，新稿不被旧确认清空，权限错误不伪作断网或泄露缓存 |

## 1.13 追加验收（全部 not_executed）

[CAPABILITY_WORKFLOWS.md](CAPABILITY_WORKFLOWS.md) 是下列场景的完整输入、操作、故障与下游断言来源；不是可选建议，不能只运行本表引用检查就声称软件通过。

| 编号 | 完整场景 | 初始状态 |
| --- | --- | --- |
| R13-WEB-001 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-002 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-003 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-004 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-005 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-006 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-007 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-008 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-009 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-010 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-011 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
| R13-WEB-012 | CAPABILITY_WORKFLOWS.md 的同ID场景 | not_executed |
