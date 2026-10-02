**中文** | [English](README_EN.md)

# C-SSH 0.9.3

面向 **Windows、M系列 Mac、Android 和 Linux** 的服务器运维工作台：主机管理、持久终端、RDP、监控、文件和三作用域 AI。无需登录 Cloud 也能使用本地功能。

> **0.9.3 只通过公告告知，请自行下载并安装；不弹升级提醒、不强制更新。**
>
> Android 沿用 0.9.0 安装签名，可覆盖 0.9.0／0.9.1／0.9.2；更早的旧正式签名不能直接覆盖，卸载会清除本地数据。请先阅读[安装与数据保全说明](INSTALL.md)，尤其注意 AI 对话和记忆不参与 Cloud 同步。Mac 仅支持 M 系列，DMG 采用 ad-hoc 签名且未公证。

## 下载

| 平台 | 0.9.3 官方下载 |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.3_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.3_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.3_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.3_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_android-arm64.apk) |
| Windows 11 ARM64 · NSIS | [C-SSH_0.9.3_arm64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_arm64-setup.exe) |
| Windows 11 ARM64 · Portable | [C-SSH_0.9.3_portable-Windows-arm64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_portable-Windows-arm64.zip) |
| Linux x86_64 · deb | [C-SSH_0.9.3_amd64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_amd64.deb) |
| Linux x86_64 · AppImage | [C-SSH_0.9.3_amd64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_amd64.AppImage) |
| Linux ARM64 · deb | [C-SSH_0.9.3_arm64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_arm64.deb) |
| Linux ARM64 · AppImage | [C-SSH_0.9.3_aarch64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_aarch64.AppImage) |

[官网](https://c-ssh.com) · [官网下载页](https://c-ssh.com/downloads) · [本次 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.3) · [安装说明](INSTALL.md)

iOS 本次不上传；不提供 Intel Mac、MSI、AAB 或 x86 Android 发布包。Windows 需要可用的 WebView2 Runtime；Mac 最低系统目标为 macOS 13，Android 最低系统目标为 Android 7.0/API 24。最低目标不等于所有系统都已实测。

## 这次更新了什么

- **终端键盘避让**：Android 自适应终端随键盘可见区域调整，修正游标和输入焦点定位；快捷键保持在键盘上方，展开时为输入行留出空间。
- **更易关闭表单**：新增／编辑主机表单增加右上关闭按钮，长表单滚动时仍可操作；点击表单外侧不会丢失输入。
- **移动终端字号**：Android／iOS 默认字号为 6，保留已经保存的自定义字号。
- **版本同步**：iOS 同步维护 0.9.3，但本次不上传 iOS 安装包。
- **手动安装**：本次只发公告，请自行下载安装；不弹升级提醒、不强制更新。

## 完整能力

| 模块 | 能做什么 |
|---|---|
| 主机与项目 | 新增、编辑、搜索、收藏、分组；IPv4/域名、独立端口、密码/私钥和显式代理线路 |
| SSH 与 Agent | 按主机选择普通 SSH 或 Agent 模式；身份、能力及错误由共享运行时处理 |
| 终端 | Linux tmux、Windows Terminal2/psmux 持久终端及普通 SSH；桌面多标签、移动端输入与会话恢复 |
| Windows RDP | 远程桌面及 RDP 内的 DVC 管理通道；证书与账号验证，登录完成后才开放管理连接 |
| 监控 | CPU、内存、磁盘、负载、网络和磁盘 IO、实时/历史曲线、Top 进程与采集设置 |
| 文件 | 浏览、上传、下载、创建、重命名、删除、文本编辑和摘要校验；手机系统文件选择器 |
| AI | 全局/项目/主机作用域，多模型账户与绑定、权限模式、确认、停止、历史及本地记忆 |
| 系统与应用 | 系统信息、进程、防火墙、服务；桌面应用中心按服务器能力管理 Docker/systemd/Windows 服务 |
| 桌面工具 | Windows/Mac/Linux 的端口映射、命令库、群发执行、逐机结果及访问授权 |
| 代理 | 直连、保存的 SOCKS5/HTTP CONNECT 线路及显式官方代理；代理失败不自动直连 |
| Cloud | 可选登录、设备管理、主机/代理/模型账户的手动端到端加密同步；官方或自建服务 |
| 偏好与诊断 | 九种语言、外观、采集参数、数据保护设置；诊断日志默认关闭 |

[逐项功能与平台差异](FEATURES.md) · [自建云接入与更新](SELF_HOSTING.md) · [完整截图目录](screenshots/README.md)

## 界面图集

[Windows](screenshots/WINDOWS.md) · [Mac](screenshots/MACOS.md) · [Android](screenshots/ANDROID.md)

[0.9.1 新增入口](screenshots/v0.9.1/README.md)与保留的 0.9.0 完整图集均使用真实 Vue 界面及合成示例数据，在后台浏览器中渲染。它展示界面，不冒充原生程序、真机或真实服务器验收；个人联系方式在公开截图中隐藏。

![Windows 新增主机与安装包导出](screenshots/v0.9.1/windows-add-host-setup.png)
![Mac 监控](screenshots/v0.9.0/macos-monitor-detail.png)

## 验证范围

本次仅复测变化相关流程及新包：五端表单后台浏览器验证、Android SDK Emulator 键盘与升级、Windows 两架构安装生命周期、Mac DMG／arm64／ad-hoc 签名、Linux Ubuntu 24.04／X11 包与表单。既有功能证据保留原版本边界；不宣称荣耀真机、所有输入法、所有 Linux 发行版或 iPhone 真机已验收。

## 数据与源码

Cloud 不转发客户端到服务器的 SSH 数据。主机凭据和模型 Key 在客户端加密后才进入手动同步；AI 对话和记忆留在本机。使用模型服务时，选定上下文会发送给该服务商。删除本地主机只处理本机数据，不卸载远端 Agent 或清理远端会话。

本公开仓提供推广资料、截图、安装包及筛选后的 Creation Cloud 生产源码镜像；客户端、共享核心、AI 与 Agent 源码未公开。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.3_android-arm64.apk | `261f028ed76462577f0bd369404a5c4e5a47cf6b7b1ff8422851b1e0401612e9` |
| C-SSH_0.9.3_aarch64.AppImage | `842c1ca1b01650a8e19a27f43cd4aef839e7e0255e34d0aa923591e86ac291a4` |
| C-SSH_0.9.3_amd64.AppImage | `26249275cc7d2e9f9e85cf4d47e89d11ca9740590fc3f6626328274cd214145b` |
| C-SSH_0.9.3_amd64.deb | `8686242edc6c470f039dbb6e710143e964c75da4e18750a18b6e57c408b88292` |
| C-SSH_0.9.3_arm64.deb | `d6b92a1b0d2f33144f6ab063703483f491a94092f69d7aeb1d22ba59ebcfe17f` |
| C-SSH_0.9.3_macOS-arm64.dmg | `d86c2cdf8563f53602f74e1d0200d95b79dd35b89bc08f6f55f236b88ba09785` |
| C-SSH_0.9.3_arm64-setup.exe | `98ee1a090ec8143a0fd47336319c86a9dd03877cd5b62c302dae8da294310091` |
| C-SSH_0.9.3_portable-Windows-arm64.zip | `8c419270eff33874887e2848c5167c77c05ca873d6b6ebdfcf2e45e431b4f35e` |
| C-SSH_0.9.3_portable-Windows-x64.zip | `bf290d13963885b8f5c6eac8da576b8aac731d26d20df0de204e39ad190aeb54` |
| C-SSH_0.9.3_x64-setup.exe | `0defc336bfe744b5f6b76917d3954c53dd5e0ccfa0c9b74b8973deb8f78a1b09` |

## 反馈与联系

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [官网反馈](https://c-ssh.com/feedback)
- QQ 群【AI 创新社区】：[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)，群号 1041937161
- 支持简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어。

旧版 Release、安装包及摘要保持原样，见[历史发布](https://github.com/suiyuebaobao/C-SSH/releases)。
