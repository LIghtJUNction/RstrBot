> 来源：`docs/DESKTOP.md`，已提交基线 `ead5bf2cea735078e1a37655dcc136485f781b7f` 上叠加当前工作区内容；SHA-256见根包SOURCE_HASHES.md。这是原生工程说明与未验证范围，不能据此声明已生成安装器。

# Desktop source preview

日期：2026-10-28。对应 RB-001、RB-015、RB-043、RB-044、RB-046。

## 当前交付是什么

`desktop/` 是独立的 Tauri 2 桌面工程。它把已经存在的 Rust 核心和 WebUI 放进原生窗口，不复制聊天、授权或数据库业务逻辑。**这是未进行原生构建和安装验收的源码工程，不是已经验证的安装器。** 核心发行包可以独立运行；普通服务器构建不会拉取桌面系统依赖。

Tauri 版本固定为 `2.12.5`，构建辅助库固定为 `2.7.4`。这两个版本在本次实施时通过官方 crate 文档核实。桌面依赖尚未解析和编译，因此暂未提交桌面 `Cargo.lock`；首次原生构建应生成、审查并提交该锁文件，随后使用 `--locked` 重建。根服务的 `Cargo.lock` 不等于桌面锁文件。

## 两种运行路径

| 路径 | 实际行为 | 数据与生命周期 |
| --- | --- | --- |
| 本机工作区 | 从应用旁启动固定名称的 `rstrbot` sidecar，并加载打包的 WebUI | 使用与 CLI 相同的系统 RstrBot 数据目录；只停止本次启动的子进程，保留数据库、密钥、配置和插件数据 |
| 连接已有服务器 | 打开用户指定的 HTTPS 根地址 | 不启动核心，不创建本地业务数据库，不产生本机初始化凭据；浏览器引擎仍可能保存自己的会话数据 |

本机模式在 Unix 上有源码实现。Windows 本机模式明确拒绝启动，因为当前核心插件宿主尚未实现受验收的 Windows named pipe；Windows 可以选择远程连接。macOS 的 Unix 实现也没有原生运行证据，不能以 Linux 的插件测试替代。

本机启动先选择一个空闲回环端口，再启动 sidecar。只有从**该子进程的 stdout** 读到准确的监听成功标记后才打开工作区，避免把端口恰好已有 HTTP 响应当作本次后端已启动。竞争占用端口会导致本次子进程启动失败，不会自动连接其他服务。短期初始化凭据从固定的 `bootstrap.token` 文件读取，只返回给打包的启动器，不放在 URL 或远程网页中。

关闭工作区或退出桌面时，Unix 路径先发送 SIGTERM；核心的正常关闭流程取消运行和插件进程组。启动器最多等候 12 秒，再终止仍未退出的核心。强制杀死整个桌面应用、操作系统崩溃、失控后代进程和 macOS 签名权限等仍需原生故障测试。这里**没有**声称一个进程句柄就能保证所有异常情况下的完整进程树清理。

## 原生权限边界

- 只有名为 `launcher` 的打包页面拥有三个应用命令：本机启动、远程连接、断开。
- Rust 命令还验证调用窗口名称和打包页面的实际 origin。仅仅知道命令名不获得调用权限。
- `workspace` 窗口没有原生命令、Shell、文件系统或远程 IPC 的 capability。聊天内容和远程服务器页面不能启动本机程序或读取本机凭据。
- 远程地址只接受 HTTPS 根地址，拒绝 URL 中的用户名、密码、查询和 fragment；登录在实际服务器页面完成。
- 工作区只能在选定 origin 内导航，额外弹出窗口被拒绝。没有自动允许 TLS 错误或把远程页面当作可信本机代码。
- 启动器使用本地 HTML、CSS、JavaScript，不使用 CDN；CSP 不允许外部脚本、内联脚本、任意 iframe 或任意网络请求。

Tauri 的 capability 是原生 API 的授权边界，不是网页全部网络行为的沙箱。远程服务器的页面、会话和内容仍由该服务器及系统 WebView 处理。

## 构建步骤

先按 [Tauri 官方平台前置条件](https://v2.tauri.app/start/prerequisites/) 安装本机所需系统依赖及 Tauri 2 CLI。记录并固定 CLI 版本；此源码包没有假设当前机器已经具备 GTK/WebKit、Xcode 或 WebView2 构建环境。

在仓库根目录构建服务和 WebUI：

```sh
cargo build --workspace --bins --release --locked
cd web
npm ci
npm run build
cd ..
python3 desktop/prepare.py --profile release
```

`prepare.py` 将四个本机二进制复制为 Tauri 要求的带 Rust target triple 后缀的 sidecar，并复制 `web/dist`。它记录 SHA-256，但不会把复制成功当作目标操作系统已经运行成功。`--check` 只校验输入，不复制大文件。

随后在桌面工程目录构建：

```sh
cd desktop/src-tauri
cargo generate-lockfile
cargo tauri build
```

`desktop/src-tauri/binaries/` 和 `resources/` 是可重建的打包输入，已忽略 Git；不得把业务数据目录复制进安装包。native Windows 文件名需要 `.exe`。交叉编译时用 `prepare.py --target ... --binary-dir ...` 显式提供该目标的真实二进制，不能简单重命名 Linux 文件充当 Windows 或 macOS 产物。

`desktop/src-tauri/icons/` 包含与 WebUI 相同原创 R 标记的 SVG、RGBA PNG、ICO 与 ICNS，避免依赖 Tauri 默认图标或缺失图标文件。图标已随源码保存；开发者若要重建，可在具备 Pillow 的环境运行 `python3 desktop/generate_icons.py`。Pillow 不属于程序运行或普通构建依赖。

## 配置与运维细节

本机默认复用系统 RstrBot 数据目录。如果同一目录已有运行中的服务器，核心的实例文件锁拒绝重复启动。此时应停止旧服务或使用远程连接，而不是修改锁文件。

当前核心的 `secure_cookie` 和 `allow_private_network` 配置会持久保存，CLI 的正向布尔开关不能把保存的 `true` 自动改回 `false`。需要关闭时，应停止实例，在受保护的 `config.json` 中明确设为 `false` 后重启。已有 HTTPS 服务配置若用于桌面回环 HTTP，应先核对 Cookie 配置；启动器不会悄悄修改已有安全配置。

远程连接模式不具备对服务器的启动、升级或停止权限。断开只关闭此桌面窗口，保留服务器任务的实际状态。

## 必须执行的原生验收

| 项目 | Linux | macOS | Windows |
| --- | --- | --- | --- |
| 桌面依赖解析、Rust 编译、安装器生成 | 未执行 | 未执行 | 未执行 |
| 安装后启动、系统数据目录与首建用户 | 未执行 | 未执行 | 本机模式未支持 |
| 连接已有 HTTPS 服务器且不创建本地数据库 | 未执行 | 未执行 | 未执行 |
| 首次凭据仅在本地启动器可见 | 源码约束；未原生验证 | 同左 | 不适用本机模式 |
| 私聊、切换工作区、附件能力显示与断线恢复 | 未原生执行 | 未执行 | 远程模式未执行 |
| 正常退出及子进程树清理 | 核心插件测试已有；桌面未执行 | 未执行 | 本机模式未支持 |
| 强制退出、崩溃、升级、回滚和卸载数据保留 | 未执行 | 未执行 | 未执行 |
| 安装器签名、系统权限、自动升级 | 未实现或未配置 | 未实现或未配置 | 未实现或未配置 |

已执行的桌面范围只包括：JSON 文件解析、启动器 JavaScript 语法检查、`prepare.py --check` 对现有 Linux 二进制与 WebUI 的输入校验、图标文件解码，以及 Rust 源文件格式/语法解析。没有把这些检查写成原生编译或安装成功。

## 实现依据

- [Tauri sidecar 文件名与打包规则](https://v2.tauri.app/develop/sidecar/)
- [Tauri capability 与远程 API 范围](https://v2.tauri.app/security/capabilities/)
- [Tauri 2.12.5 WebviewWindowBuilder](https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindowBuilder.html)
- [tauri-build 2.7.4](https://docs.rs/tauri-build/2.7.4/tauri_build/)
- [Tauri 图标格式要求](https://v2.tauri.app/develop/icons/)
