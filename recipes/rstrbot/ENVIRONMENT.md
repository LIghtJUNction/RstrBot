# 语言、平台与外部前提

| 项 | 用户确认/来源事实 | 未知或条件 |
| --- | --- | --- |
| 核心/存储/宿主/参考worker | Rust，当前edition2024；工具链1.99.1，Cargo声明rust-version1.90 | 固定版本是来源快照，不自动代表重建时最新稳定版 |
| Web | TypeScript/React，Node开发工具链24.22.1，静态资源同源交付 | 浏览器运行无需Node；目标最低浏览器版本待确认 |
| 插件SDK | Rust/Python/TypeScript/C#；Python开发3.15.2、uv0.13.6、.NET10.0.403，C# net10.0 | Python SDK最低3.11与开发固定版分开；辅助工具语言可选 |
| 原生桌面 | Tauri2路线，Linux/macOS/Windows | 原生构建/安装/签名需各平台；Windows当前本机worker不支持 |
| 服务/Web部署 | Linux/服务器/容器目标，loopback默认，远程经HTTPS代理 | CPU架构、最低OS、glibc/musl全矩阵未知；已知历史Linux包x86_64不覆盖ARM |
| 国际化 | 中文和英文完整分支、深浅主题；cron IANA时区 | 现有zh/en语言选择不擅自改成新增地区locale；RTL、新地区格式尚未指定 |
| 预算 | 启动、延迟、吞吐、内存、包体积/网络硬指标未知 | 先记录工作负载与实测再协商阈值；PERFORMANCE.md规定口径 |

## 外部前提的职责

操作系统文件/进程/套接字、Git、mise及编译器/链接器是开发/构建工具。OpenSSL与AWS-LC等系统构建要求、SQLite库、PostgreSQL服务、Valkey服务、Tauri平台依赖和WebView属于环境或成熟第三方库，使用维护版本与锁文件；本包不重建这些上游库。

模型供应商/本地模型服务、API凭据、SMTP、Google OAuth项目/公开HTTPS域名、实体或同步Passkey、真实平台账号、证书/签名与分发服务需部署者提供。diagnostic、本地SMTP/OIDC签名夹具、浏览器虚拟认证器仅验证协议路径。没有生产账号/设备时保留not_executed，不伪造连通或计费结果。

真实业务数据根、备份、secrets.key、登录cookie、bootstrap凭据不属于迁移输入。所有未来验收使用一次性数据根、临时数据库/schema、明确测试账号和受控本地服务；不得自动指向现有用户数据库。

现有RstrBot仓库保持原名，recipes/是主要输入，generated/rstrbot/保存既有源码及未提交改动并作为未来生成位置。用户将其视为临时生成物，本次使用Git忽略目录，不新建子仓库或迁移仓库。历史源码及Recipkg工具源码都没有被声明成配方依赖，也不进入配方归档。Recipkg是生成/校验材料的工具，不是RstrBot运行时依赖。
