# C-SSH 0.9.9

[English](README_EN.md) · [官网](https://c-ssh.com) · [安装说明](INSTALL.md) · [功能介绍](FEATURES.md) · [自建云](SELF_HOSTING.md) · [更新日志](CHANGELOG.md)

C-SSH 将主机管理、终端、文件、监控和 AI 助手放在同一个客户端中。本次补充 Linux x86_64 AppImage，按 Mac 0.9.9 的界面与使用方式对齐。

## 这次更新

- 改善多个 AI 会话同时运行时的切换、进度显示和任务控制。
- 改善云账号登录、页面加载和启动体验。
- 修正部分工具结果与搜索内容的显示问题。

Windows、Mac 和 Android 的 0.9.9 文件保持不变，已使用这些版本的用户无需重复下载。本次由用户手动下载安装，不自动或强制升级。

## 主要功能

- 管理主机、终端、文件和监控，支持 Agent 与普通 SSH 连接。
- 在全局、项目或单台主机中使用 AI，查看任务进度和结果。
- 手动管理与同步云端配置，也可不登录使用本机功能。
- 桌面服务器浏览器使用 Google Chrome；手机端暂不提供该入口。

功能与平台差异见[功能介绍](FEATURES.md)。本次验证了 Linux 的相关界面、AI 会话及升级后资料保留；不代表已验证所有发行版和网络环境。

## Downloads

| Platform | File | Bytes | Download |
|---|---|---:|---|
| Windows x64 | C-SSH_0.9.9_portable-Windows-x64.zip | 46,581,351 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9-r2/C-SSH_0.9.9_portable-Windows-x64.zip) |
| Mac Apple Silicon | C-SSH_0.9.9_macOS-arm64.dmg | 48,915,525 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9-r2/C-SSH_0.9.9_macOS-arm64.dmg) |
| Linux x86_64 | C-SSH_0.9.9_amd64.AppImage | 122,038,776 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9-r2/C-SSH_0.9.9_amd64.AppImage) |
| Android arm64 | C-SSH_0.9.9_android-arm64.apk | 46,778,053 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9-r2/C-SSH_0.9.9_android-arm64.apk) |

安装步骤与数据保留要求见[安装说明](INSTALL.md)。Linux ARM64 与 deb 等旧包保留在[历史发布](https://github.com/suiyuebaobao/C-SSH/releases)，不属于本次新包；iOS 暂停。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.9_portable-Windows-x64.zip | `f8e7b58ae96a6fcb603bd16af9d30090d0ce9d8defdd7f369ff4e9f157c93c34` |
| C-SSH_0.9.9_macOS-arm64.dmg | `6d6c5b49a1e3d052c41c63345ec874699b340073ba4035b4b148bedf7b6dc8ad` |
| C-SSH_0.9.9_amd64.AppImage | `f045637980ced45a0b0d9ebd25c3a6a3dc41cff16e3b249e162875e33cf051cf` |
| C-SSH_0.9.9_android-arm64.apk | `960bba3cd79935f7bf46410d8ddd402dc70650d521c378616b8677c688d6bd02` |
