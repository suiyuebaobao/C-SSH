**中文** | [English](README_EN.md)

# C-SSH 0.9.1

面向 **Windows、M系列 Mac 和 Android** 的服务器运维工作台：主机管理、持久终端、RDP、监控、文件和三作用域 AI。无需登录 Cloud 也能使用本地功能。

> **0.9.1 只通过公告告知，请自行下载并安装；不弹升级提醒、不强制更新。**
>
> Android 沿用 0.9.0 安装签名，可覆盖 0.9.0；更早的旧正式签名不能直接覆盖，卸载会清除本地数据。请先阅读[安装与数据保全说明](INSTALL.md)，尤其注意 AI 对话和记忆不参与 Cloud 同步。Mac 仅支持 M 系列，DMG 采用 ad-hoc 签名且未公证。

## 下载

| 平台 | 0.9.1 官方下载 |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.1_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.1_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.1_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.1_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_android-arm64.apk) |

[官网](https://c-ssh.com) · [官网下载页](https://c-ssh.com/downloads) · [本次 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.1) · [安装说明](INSTALL.md)

iOS 本次不上传；不提供 Intel Mac、Linux 客户端、MSI、AAB 或 x86 Android 发布包。Windows 需要可用的 WebView2 Runtime；Mac 最低系统目标为 macOS 13，Android 最低系统目标为 Android 7.0/API 24。最低目标不等于所有系统都已实测。

## 这次更新了什么

- **地址自动识别**：直接输入 IPv4 或域名，不再要求下拉选择；地址与端口分别填写，已有 IPv6 主机兼容保持。
- **内置 Windows 管理安装包**：Windows、Mac、Android 添加 Windows 主机的 DVC 管理模式可直接保存标准 Setup ZIP；Android 另可通过系统分享，纯 RDP 模式不显示。无需先下载 Windows 客户端。
- **离线取得安装包**：当前同源组件版本 0.9.1，ZIP 为 13,308,604 字节；导出不带主机、账号或凭据，不连接远端，也不在客户端运行 EXE。
- **按公告手动升级**：不投放升级提醒或强制更新；Mac 自动更新延期，继续 DMG 安装。

自建云、Windows RDP/DVC 与 Terminal2/psmux、代理和三作用域 AI 等既有能力保持，详见[更新记录](CHANGELOG.md)。

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
| 桌面工具 | Windows/Mac 的端口映射、命令库、群发执行、逐机结果及访问授权 |
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

- Windows 测试虚拟机完成同源 NSIS/Portable 检查、安装/启动/卸载、0.9.0 到 0.9.1 的**手动**安装升级及数据保留。
- Mac 完成 arm64 构建、DMG 挂载、应用版本/架构及 ad-hoc 签名完整性检查；未公证，不能将此视为最低系统或所有 Mac 型号验收。
- Android 完成包名/版本/ABI/签名/16KiB 对齐/部署资源检查，并在 Android SDK Emulator 验证0.9.0 同签名覆盖、启动、设置保持及系统保存／分享入口；更早旧签名安装限制仍在，不是物理手机验证。
- 主机地址、三端表单、自建云隔离及共享层定向测试已完成；公网 IPv6、全部系统组合及所有弱网场景不在本次已验证范围。

## 数据与源码

Cloud 不转发客户端到服务器的 SSH 数据。主机凭据和模型 Key 在客户端加密后才进入手动同步；AI 对话和记忆留在本机。使用模型服务时，选定上下文会发送给该服务商。删除本地主机只处理本机数据，不卸载远端 Agent 或清理远端会话。

本公开仓提供推广资料、截图、安装包及筛选后的 Creation Cloud 生产源码镜像；客户端、共享核心、AI 与 Agent 源码未公开。

## SHA256

- `87738a053666b9a44503cfba10f70db2a075d80c4f703fd857d6c7933974d49f`  `C-SSH_0.9.1_x64-setup.exe`
- `a19dd017d41e79fbd8cdfd1bdb159828eb4368c131b381fcdf537fdc543b3eba`  `C-SSH_0.9.1_portable-Windows-x64.zip`
- `bc0aa6ffb0459e66ea7efc8536c1c6fe8076b3cf1a4dec7c9bcc96a0b91f0a74`  `C-SSH_0.9.1_macOS-arm64.dmg`
- `9cf10f2049a6824ff9bb5cd5af95f3576a3964d3700ee0ccc77ccff63c74a20e`  `C-SSH_0.9.1_android-arm64.apk`

## 反馈与联系

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [官网反馈](https://c-ssh.com/feedback)
- QQ 群【AI 创新社区】：[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)，群号 1041937161
- 支持简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어。

旧版 Release、安装包及摘要保持原样，见[历史发布](https://github.com/suiyuebaobao/C-SSH/releases)。
