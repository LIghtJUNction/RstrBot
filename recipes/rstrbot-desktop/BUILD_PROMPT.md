# 桌面启动器

Rust / Tauri 2路线，本地sidecar与远程HTTPS工作区两种模式。依赖 `client` 提供Web资源及其递归service/core、插件与存储；`.deps/client/` 是重建输入。桌面不需要另造业务逻辑，只集成相同的账户/工作区/API。阅读 [DESKTOP_REFERENCE.md](DESKTOP_REFERENCE.md)。

本地模式定位相邻、与平台架构匹配的核心及三个worker和Web资源；生成Tauri所需target triple文件名。系统数据目录或显式便携目录独立于工作目录，重复打开同数据根遵守锁；bootstrap秘密仅本地可信路径可见，不嵌URL。退出须有界停止核心和所属完整进程树，崩溃/升级/卸载保留业务数据。

远程模式只接受明确指定HTTPS服务器，连接不创建本地数据库或核心，也不替服务器升级/停止任务。远程网页不自动获得Tauri宿主能力；原生命令暴露最小化，不把WebView导航限制当内容沙箱。

目标Linux、macOS、Windows；来源仅提供桌面工程与部分静态检查，没有三平台原生安装/签名/升级成功证据。Windows历史本机worker不支持；未补齐协议transport/ACL/进程树前本机模式必须如实不可用。需要GTK/WebKit、Xcode、WebView2等平台构建前提以及真实target产物，不能改名Linux二进制冒充其他平台。

原生签名账号、分发基础设施、自动升级服务属外部前提/待实现工作，性能数字未知。重建后分别验收每个平台；不能用Linux核心检查代表桌面安装完成。
