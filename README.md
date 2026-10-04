**中文** | [English](README_EN.md)

# C-SSH 0.9.4

面向 **Windows、M系列 Mac、Android 和 Linux** 的服务器运维工作台：主机管理、持久终端、RDP、监控、文件和三作用域 AI。无需登录 Cloud 也能使用本地功能。

> **0.9.4 只通过公告告知，请自行下载并安装；不弹升级提醒、不强制更新。**
>
> Android 沿用 0.9.0 安装签名，可覆盖 0.9.0／0.9.1／0.9.2／0.9.3；更早的旧正式签名不能直接覆盖，卸载会清除本地数据。请先阅读[安装与数据保全说明](INSTALL.md)，尤其注意 AI 对话和记忆不参与 Cloud 同步。Mac 仅支持 M 系列，DMG 采用 ad-hoc 签名且未公证。

## 下载

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.4_android-arm64.apk | 39,707,218 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_android-arm64.apk) |
| C-SSH_0.9.4_aarch64.AppImage | 116,402,696 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_aarch64.AppImage) |
| C-SSH_0.9.4_amd64.AppImage | 117,410,296 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_amd64.AppImage) |
| C-SSH_0.9.4_amd64.deb | 43,882,432 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_amd64.deb) |
| C-SSH_0.9.4_arm64.deb | 43,049,976 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_arm64.deb) |
| C-SSH_0.9.4_macOS-arm64.dmg | 42,342,556 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_macOS-arm64.dmg) |
| C-SSH_0.9.4_arm64-setup.exe | 45,720,976 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_arm64-setup.exe) |
| C-SSH_0.9.4_portable-Windows-arm64.zip | 41,482,505 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_portable-Windows-arm64.zip) |
| C-SSH_0.9.4_portable-Windows-x64.zip | 42,794,746 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_portable-Windows-x64.zip) |
| C-SSH_0.9.4_x64-setup.exe | 47,565,127 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_x64-setup.exe) |

[安装与数据保全](INSTALL.md) · [官网](https://c-ssh.com) · [Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.4)

0.9.4只通过公告告知，请手动下载并安装；不弹升级提醒、不强制更新。Mac仅M系列arm64、最低目标macOS13，采用ad-hoc签名且未公证；Windows安装器没有Authenticode签名。Android沿用0.9.0以后的正式signer；更早不同签名版本不能直接覆盖。AI内容不参与Cloud同步，请先阅读安装与资料保全说明。

## 这次更新了什么

### Added

- **批量导入主机**：支持CSV和OpenSSH配置。先预览、筛选与确认，再建立“待接入”项；补充凭据并通过真实认证后才成为正式主机。导入预览不等于已连接，密码与私钥不写入明文待接入队列。
- **可跳过的首次引导**：首次使用可了解接入、导入与运维入口，也可直接跳过进入工作台；选择在冷启动后保持，设置中可以重新打开。已有资料升级不强制重新引导。
- **自动AI运维Skills**：工具实际执行且验收成功的经验才自动学习；当前环境匹配后在后续任务中自动参考。无需手动保存或采纳，普通界面只显示一次“参考过往经验”的轻提示，原有权限和操作确认继续生效。Skills、AI对话、原始记录和长期记忆均保留在本机，不参与Cloud同步。
- **易读的逐行操作日志**：一次业务操作对应一行，直接查看时间、级别、本机主机名称和可信结果。采集默认关闭，可按需开启、查询，切换技术视图及导出支持材料；日志不会把子步骤或传输成功外推为整次成功，未知或不完整结果明确显示。
- **版本与资料升级**：四个平台发布0.9.4，客户端SQLite为22；iOS同步维护0.9.4／9004000，本次不上传iOS安装包。

### Fixed

- **霜白主题按钮可读性**：修正Windows、Mac和Linux主按钮的底色、文字、悬停、禁用与键盘焦点状态。
- **Android新装冷启动**：修正先前0.9.4候选包在冷启动时被错误拒绝的问题；本次正式APK已在新的arm64 Android SDK Emulator完成冷启动与正常界面验证，沿用原正式安装签名。旧候选的失败记录保留。

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

[0.9.1 新增入口](screenshots/v0.9.1/README.md)（公开于2026-09-30）与保留的 0.9.0 完整图集（公开于2026-09-29）均使用真实 Vue 界面及合成示例数据，在后台浏览器中渲染。它展示界面，不冒充原生程序、真机或真实服务器验收；个人联系方式在公开截图中隐藏。

![Windows 新增主机与安装包导出](screenshots/v0.9.1/windows-add-host-setup.png)
![Mac 监控](screenshots/v0.9.0/macos-monitor-detail.png)

## 验证范围 / Verified

- 本次十项正式包的文件名、大小及SHA256与平台收据逐一绑定，合计580,358,528字节。Windows双架构为当前Release与包级门禁，Portable仅有`C-SSH/C-SSH.exe`；本轮未重复安装生命周期或普通用户资料启动。
- Mac为普通arm64 Release与DMG，ad-hoc签名检查通过，未公证；复用同源Mac业务验证，不启动用户普通资料。Android正式APK验证0.9.4／9004000、原signer、16KB对齐、fresh SDK Emulator冷启动及正常操作日志行。
- Linux双架构deb／AppImage完成包级和隔离最小启动；AppImage使用提取入口。没有新增FUSE挂载、升级、其它发行版或物理机器验证。
- 导入、引导和自动Skills复用10-02／10-03专项原生证据：Mac arm64；Windows ARM64及ARM VM中的x64；Linux ARM64，x64仅其原生差异；Android SDK Emulator及iPhone Simulator。SSH／Windows管理与真实provider学习／冷启复用按各自范围成立，不宣称每个架构都重跑完整矩阵。
- 逐行操作日志的Mac同源实际业务证据及Android正常行已完成；其它平台采用当前共享实现、前端／包级与对应最小启动证据，不宣称全平台所有业务日志都已复测。
- 不宣称Android或iPhone真机、全部输入法、全部Linux发行版／Wayland、所有断网恢复或本轮未实测功能完成验收。

## 数据与源码

Cloud 不转发客户端到服务器的 SSH 数据。主机凭据和模型 Key 在客户端加密后才进入手动同步；AI 对话和记忆留在本机。使用模型服务时，选定上下文会发送给该服务商。删除本地主机只处理本机数据，不卸载远端 Agent 或清理远端会话。

本公开仓提供推广资料、截图、安装包及筛选后的 Creation Cloud 生产源码镜像；镜像包含现行Rust API、Nuxt前端及生产迁移；客户端、共享核心、AI 与 Agent 源码未公开。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.4_android-arm64.apk | `8ee9649e69ed73660f1bdc42c6c56de9964de64f1ecfe9d72bb52d2ec26fd76c` |
| C-SSH_0.9.4_aarch64.AppImage | `a30eeadb2052a3a7702b4ac4de20ab0544ccf6a1552cd8b4fcfdccab3a744402` |
| C-SSH_0.9.4_amd64.AppImage | `6e536a4c070539d7ff1195ebfb92b6654402450fdf5d82d5a86de3a47ff91ad5` |
| C-SSH_0.9.4_amd64.deb | `8bd0330364baf7fe5f5fd6bc71c3c64cb54c4351184dc2a578ccbaf58dd98edc` |
| C-SSH_0.9.4_arm64.deb | `fb12a27ae9fc2e93729aeff6ea826e3b45cc01ca5f168f2283ccc4a8d752d281` |
| C-SSH_0.9.4_macOS-arm64.dmg | `b6c472e64e98965f9859e65cc738de2f2895dbebfb1d5a080ba4c9f553cfe401` |
| C-SSH_0.9.4_arm64-setup.exe | `463ae846ea6d2129f9fd1e2a25055f02a78a9dcffb9e80abf0672e9477ec8bb6` |
| C-SSH_0.9.4_portable-Windows-arm64.zip | `1dd2abe077602c94fb976b849648e5432f984ad482de1da91828040520321627` |
| C-SSH_0.9.4_portable-Windows-x64.zip | `32689bfaf190bab66ae5197d12ece09270358bc139f40c1a30ced15563f72841` |
| C-SSH_0.9.4_x64-setup.exe | `280b1144366a283f20fd75f224c800239db072d5dd1c0fdd162b71c4ea21ff54` |

## 反馈与联系

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [官网反馈](https://c-ssh.com/feedback)
- QQ 群【AI 创新社区】：[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)，群号 1041937161
- 支持简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어。

旧版 Release、安装包及摘要保持原样，见[历史发布](https://github.com/suiyuebaobao/C-SSH/releases)。
