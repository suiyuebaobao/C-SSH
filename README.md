**中文** | [English](README_EN.md)

<div align="center">

# Creation-SSH（C-SSH）

### 手机上也能接着运维：持久化终端、常驻监控、文件管理与 AI 助手

[![Android](https://img.shields.io/badge/下载-Android-3DDC84?logo=android&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.apk)
[![Windows](https://img.shields.io/badge/下载-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)
[![Linux](https://img.shields.io/badge/下载-Linux-FCC624?logo=linux&logoColor=black)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)
[![Stable](https://img.shields.io/badge/stable-v0.6.13-2ea44f)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)

</div>

Creation-SSH 是一套跨平台 SSH 运维客户端。Android 不是只读遥控器：它可以直接管理主机、恢复服务端 tmux 持久化会话、查看监控、处理文件、调用 AI 助手和进入系统管理；Windows 与 Linux 桌面端负责更完整的日常运维工作流。

核心能力由 Linux 服务器上的常驻 agent 结构化提供，普通终端和端口映射仍保留纯 SSH 路径。当前公开稳定版为 **`v0.6.13`**；`v0.6.11` 是已保留的预发布历史版本，不建议安装。

## v0.6.13 更新重点

- Windows/Linux 主机列表使用紧凑自适应指标网格，显示真实系统、CPU、内存、磁盘、负载、运行时长和正常/暂停/失败三态。
- Windows、Linux 与 Android 切换菜单后继续保留普通终端和持久化终端的连接、画面与输入状态，重新附加不再出现重复提示符。
- 修复 agent 成功后立即执行一次真实监控采集；附加系统信息失败不再丢弃已成功取得的动态指标。
- 不可达主机允许在明确警告远端残留后删除本地记录；可用主机仍执行严格归属审计与安全清理。
- Windows/Linux 的 AI 助手会禁用明确离线主机的输入与发送，Windows 开屏重新按完整窗口居中。

## 先看 Android

同一套主机和 tmux 会话可以在桌面与手机之间继续使用。Android `v0.6.13` 已交付 arm64 APK/AAB，并完成真实应用流程验证；公开 Release 不提供 x86_64 模拟器测试包。

## 下载

| 平台 | 推荐下载 | 其他正式资产 |
| --- | --- | --- |
| Android arm64 | [APK](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.apk) | [AAB](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.aab)，用于应用商店分发 |
| Windows x64 | [安装版 EXE](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_x64-setup.exe) | [MSI](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_x64_en-US.msi) · [便携版 ZIP](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_portable-Windows-x64.zip) |
| Linux x86_64 | [AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_linux-x86_64.AppImage) | [Debian/Ubuntu deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_linux-amd64.deb) |

版本说明和 SHA256 见 [v0.6.13 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)，历史记录见 [CHANGELOG.md](CHANGELOG.md)。

## 已交付平台

| 平台 | `v0.6.13` 已交付范围 |
| --- | --- |
| Android | 主机管理、agent 部署入口、持久化/普通终端、文件、实时监控、AI、系统管理、本地登录门与“我的”设置 |
| Windows | 完整桌面工作流；提供 EXE、MSI 与便携 ZIP |
| Linux 桌面 | 独立 AppImage/deb；公开验证覆盖持久化终端重开、监控、系统/进程、文件、AI 与失效重连 |
| Linux agent | x86_64 musl 静态二进制，由客户端经 SSH 部署；服务器不需要开放额外 agent 端口 |
| iOS / macOS | **尚未发布**，不属于 `v0.6.13` 已交付范围 |

## 主要页面

### Android

| 页面 | 能做什么 |
| --- | --- |
| 主机 | 新增、编辑和删除主机，部署/修复 agent，进入终端、监控和系统管理 |
| 终端 | 在可重连的 tmux 持久化终端与普通 SSH PTY 间切换；支持窗口、字体、尺寸、滚动、复制和移动快捷键 |
| 文件 | 浏览、编辑、新建、重命名和删除远端文件；通过 Android SAF 选择下载位置，保留断点续传与完整性校验 |
| 监控 | 查看 CPU、内存、磁盘、网络、磁盘 I/O 和 Top 进程；后台跨主机采集设置保存在本地 SQLite |
| AI 助手 | 选择主机、模型与权限档，查看历史和上下文；工具执行受权限与确认控制 |
| 系统管理 | 查看系统信息、进程和防火墙端口，执行杀进程与修改 SSH 密码等需确认操作 |
| 我的 / 登录门 | 语言、主题、版本、更新与本地安全设置；设置登录密码后先解锁本地保险库 |

### Android 真实截图（v0.6.13）

以下截图来自 Android 真实运行流程，并已在公开前完成脱敏核对。

<div align="center">
<img width="180" src="screenshots/mobile-hosts.png" alt="Android 主机列表" />
<img width="180" src="screenshots/mobile-terminal.png" alt="Android 持久化终端" />
<img width="180" src="screenshots/mobile-files.png" alt="Android 文件管理" />
<img width="180" src="screenshots/mobile-monitor.png" alt="Android 实时监控" />
<img width="180" src="screenshots/mobile-ai.png" alt="Android AI 真实响应" />
</div>

### Windows 与 Linux 桌面

Windows 提供下列完整桌面入口。Linux 已交付独立桌面客户端；`v0.6.13` 公开验证明确覆盖主机连接、终端、监控、文件、AI 和系统/进程核心链路。

| 页面 | 能做什么 |
| --- | --- |
| 主机管理 | 分组、收藏、搜索、凭据选择，以及 agent 部署、修复和状态查看 |
| AI 助手 | 结合已授权的主机上下文读取指标、日志和文件并执行工具；桌面支持独立 AI 窗口 |
| 终端 | tmux 持久化终端与普通 SSH PTY 双模式，断线或换设备后可恢复持久化窗口 |
| 监控 | 跨主机健康概览、单机实时详情和历史范围查询 |
| 文件 | 远端文件管理、在线编辑、分块传输、断点续传和完整性校验 |
| 端口映射 | SSH 本地转发；默认监听 `127.0.0.1`，可保存、启动和停止映射 |
| 命令片段 | 保存常用命令并对多台主机执行，结果按主机分组 |
| 系统管理 | 系统信息、进程、防火墙端口和 SSH 密码管理 |
| 应用中心 | 安装 Docker，部署 Nginx/Redis 等应用，管理容器、镜像与 systemd 服务 |
| 访问授权 | 查看本地保险库、SSH key、一次性授权和 AI 审计记录 |
| 设置 | AI provider、语言、外观、本地登录、监控采集与更新检查 |

<div align="center">
<img width="400" src="screenshots/hosts.png" alt="桌面主机管理" />
<img width="400" src="screenshots/terminal.png" alt="桌面终端" />
<img width="400" src="screenshots/monitor-list.png" alt="桌面监控入口" />
<img width="400" src="screenshots/monitor.png" alt="桌面监控" />
<img width="400" src="screenshots/files.png" alt="桌面文件管理" />
<img width="400" src="screenshots/ai.png" alt="桌面 AI 助手" />
</div>

## 安全边界

- 私钥和密码只保存在当前设备的本地加密保险库，不上传服务器或 C-SSH 云端；本项目没有托管凭据云服务。
- agent 通过 SSH 隧道访问并只监听服务器本机 Unix socket，不额外暴露公网端口；agent 以当前 SSH 登录身份执行，不自行提权。
- 主机密钥异常会停止连接，破坏性操作需要明确确认；无法证明属于 C-SSH 的服务、会话、socket、数据或公钥不会自动删除。
- 端口映射默认绑定 `127.0.0.1`。如手动改为其他监听地址，局域网暴露风险由用户自行评估。
- AI 工具受权限档和执行确认约束；使用第三方 AI provider 时，用户选定的对话与上下文会按该 provider 的服务条款处理。

## 免费、语言与开源计划

Creation-SSH 当前永久免费，不设订阅、会员或付费功能锁；界面内置简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어。

**当前版本尚未开源。** 本仓库只用于项目介绍、截图与 Release 资产分发。计划是在 iOS 与 macOS 正式版发布后公开源代码；这是后续计划，不代表当前仓库已包含源码，也不承诺具体日期。

## 联系

- 微信：`suiyue_creation`
- QQ 群【AI 创新社区】：[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)
