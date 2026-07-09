**中文** | [English](README_EN.md)

<div align="center">

# Creation-SSH（C-SSH）

### 跨平台 SSH 运维新体验 —— 原生客户端 × 服务端 tmux 持久化 × 常驻监控 × 内置 AI 助手

[![下载 Windows](https://img.shields.io/badge/下载-Windows-0078D6?logo=windows&logoColor=white)](../../releases/latest)
[![下载 Android](https://img.shields.io/badge/下载-Android-3DDC84?logo=android&logoColor=white)](../../releases/latest)
[![支持全球](https://img.shields.io/badge/支持全球-Global-2ea44f)](../../releases/latest)
[![永久免费](https://img.shields.io/badge/永久免费-Free%20Forever-ff69b4)](../../releases/latest)
[![开源计划](https://img.shields.io/badge/开源计划-iOS和macOS正式版发布后开源-success)](../../releases/latest)

</div>

---

## 这是什么

Creation-SSH 不是又一个网页运维面板,也不是普通的 SSH 终端。它把 **Xshell 级的原生客户端体验**、**服务端常驻 agent 的结构化能力**、**tmux 级的会话持久化**三者合一:客户端体验流畅原生,重活由服务器上的常驻 agent 结构化承担,终端会话即使断网、关机、换设备也永不丢失。

一句话:**原生客户端 × 常驻结构化 agent × 持久化会话**,三合一的现代 SSH 运维工具。

---

## 桌面端页面导览

> 以下截图使用 `example.com` 等脱敏演示数据,不包含真实服务器或凭据。

<div align="center">

### 主机管理
<img width="820" src="screenshots/hosts.png" alt="主机管理" />

</div>

首页集中管理 SSH 主机、分组、收藏、搜索、agent 部署与修复。新增主机支持密码或 OpenSSH 私钥,凭据只进入本地加密保险库;部署过程会展示连接、上传、启动、握手等步骤。

<div align="center">

### AI 助手
<img width="820" src="screenshots/ai.png" alt="AI 助手" />

</div>

AI 助手可以带着主机上下文读监控、看日志、改文件、跑命令。顶部工作区收纳历史、主机、权限、上下文和性能档位;桌面端支持独立弹窗,方便同时操作多个 AI 助手窗口。

<div align="center">

### 终端
<img width="820" src="screenshots/terminal.png" alt="双模式终端" />

</div>

**持久化模式**由 agent 直驱 tmux,断线、关机、换设备后重连,通过 `capture-pane` 恢复完整屏幕内容,正在跑的任务一行不丢;**直连模式**为纯原生 PTY,未装 agent 也能当普通终端用。两种模式随手切换,兼顾护城河与兜底。

<div align="center">

### 监控入口
<img width="820" src="screenshots/monitor-list.png" alt="监控入口" />

</div>

监控入口先展示所有主机健康概览,可以快速发现离线、异常负载和待处理主机,再进入单机详情。适合日常巡检,不用逐台打开终端敲命令。

<div align="center">

### 监控详情
<img width="820" src="screenshots/monitor.png" alt="常驻监控" />

</div>

常驻 agent 持续采集 CPU、内存、磁盘、网络、磁盘 IO、Top 进程六大维度,实时六卡呈现,历史数据落 redb 时序库,可回溯任意时间范围。无需自己搭监控栈,连上即用。

<div align="center">

### 文件管理
<img width="820" src="screenshots/files.png" alt="文件管理器" />

</div>

图形化浏览远端文件系统,支持增删改查、在线编辑、权限查看;上传下载分块传输、断点续传,大文件也稳。由 agent 结构化提供文件能力,不靠客户端拼 shell。

<div align="center">

### 端口映射
<img width="820" src="screenshots/ports.png" alt="端口映射" />

</div>

端口映射基于 SSH 本地转发,把远端内网服务安全映射到本机。默认绑定 `127.0.0.1`,避免误暴露到局域网;已保存的映射可以重建、停止和移除。

<div align="center">

### 命令片段
<img width="820" src="screenshots/snippets.png" alt="命令片段" />

</div>

命令片段把常用运维命令整理成库,可勾选多台主机批量执行,结果按主机分组展示。适合巡检、快速排障和重复性操作。

<div align="center">

### 系统管理
<img width="820" src="screenshots/sysmgmt.png" alt="系统管理" />

</div>

系统管理提供只读系统信息、进程管理、防火墙端口和 SSH 密码修改。终止进程、改密等高风险动作需要二次确认,并以 SSH 登录身份执行,不额外提权。

<div align="center">

### 应用中心
<img width="820" src="screenshots/appcenter.png" alt="应用中心" />

</div>

内置应用商城:一键安装 Docker 本身,一键部署 Nginx、Redis 等常用容器应用;结构化管理 Docker 容器与镜像、systemd 服务(启停、查日志)。破坏性操作二次确认,以 SSH 登录身份执行。

<div align="center">

### 访问授权
<img width="820" src="screenshots/grants.png" alt="访问授权" />

</div>

访问授权集中管理本地保险库、已建 SSH key、一次性授权与 AI 审计记录。凭据永远只留在本机,不会上传服务器或云端。

<div align="center">

### 设置
<img width="820" src="screenshots/settings.png" alt="设置" />

</div>

设置页集中配置 AI provider、上下文窗口、工具循环次数、语言跟随系统、登录密码、外观透明度、监控采集节奏和 GitHub 更新检测。

---

## 移动端页面导览(Android)

桌面端能力装进口袋。同一套 tmux 持久化会话、常驻监控、文件管理与 AI 助手,随时随地在手机上继续运维。

<div align="center">
<img width="180" src="screenshots/mobile-login.png" alt="移动端登录" />
<img width="180" src="screenshots/mobile-hosts.png" alt="移动端主机" />
<img width="180" src="screenshots/mobile-terminal.png" alt="移动端终端" />
<img width="180" src="screenshots/mobile-files.png" alt="移动端文件" />
<img width="180" src="screenshots/mobile-monitor.png" alt="移动端监控" />
<img width="180" src="screenshots/mobile-ai.png" alt="移动端 AI 助手" />
<img width="180" src="screenshots/mobile-sysmgmt.png" alt="移动端系统管理" />
<img width="180" src="screenshots/mobile-me.png" alt="移动端我的" />
</div>

### 移动端主机
主机页用卡片管理服务器,支持新增、编辑、删除、部署 agent,也可以从卡片直接跳转终端、监控、系统管理。

### 移动端终端
终端页同样支持持久化 tmux 与普通 PTY 双模式,并提供 Ctrl、Esc、Tab、方向键等移动快捷键栏。

### 移动端文件
文件页支持目录浏览、在线编辑、下载到 App 私有目录、新建、重命名、删除和隐藏文件切换。

### 移动端监控
监控页可订阅实时指标流,用手机查看 CPU、内存、磁盘、网络和 Top 进程。

### 移动端 AI 助手
AI 页保留主机选择、权限模式、上下文、历史和配置入口;软键盘打开时输入区自动避让,不会挡住正在输入的内容。

### 移动端系统管理
系统管理作为内页从主机入口进入,覆盖系统信息、防火墙端口、杀进程和改 SSH 密码。

### 我的 / 登录门
“我的”页包含语言、检查更新、版本、登录密码和本地安全设置。设置过登录密码后,启动时会先进入本地登录门解锁保险库。

## 核心优点 · Why C-SSH

- **原生客户端体验** —— 全栈 Rust + Tauri 2,启动快、占用低,不是套壳网页面板,操作如 Xshell 般顺手。
- **持久化不丢会话** —— agent 直驱 tmux,断网/关机/换设备重连即恢复,长任务永不中断。
- **常驻结构化 agent** —— 监控、文件、应用、系统管理都由服务端常驻 agent 结构化提供,高效、可复用,而非在客户端裸拼 shell。
- **内置 AI 双接口** —— 同时支持 OpenAI 兼容与 Anthropic,五档权限 + 执行确认,能力强又安全可控。
- **凭据本地加密、零上传** —— 私钥与密码仅存本地加密保险库,绝不上传服务器或任何云端。
- **多语言全球化** —— 界面内置 9 种语言,面向全球用户。
- **跨桌面与移动** —— Windows 桌面 + Android 移动伴侣,一套体验随身带走。

---

## 支持平台

| 平台 | 状态 | 说明 |
| --- | --- | --- |
| Windows | ✅ 已支持 | 桌面客户端(setup.exe / msi) |
| Android | ✅ 已支持 | 移动伴侣(arm64 APK) |
| 服务端 agent（Linux） | ✅ 已支持 | x86_64 / ARM64,musl 静态二进制,单文件部署 |
| macOS | 🚧 计划中 | 正式版发布后与 iOS 一起推进开源计划 |
| iOS | 🚧 开发中 | 正式版发布后与 macOS 一起推进开源计划 |

---

## 🌍 支持全球 · 💛 永久免费

Creation-SSH **面向全球用户**,界面内置 **9 种语言**(简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어),无论你在哪里都能顺畅使用。

本产品**永久免费**,不设订阅、不卖会员、不锁功能。💛

---

## 🔓 开源承诺

**iOS 和 macOS 正式版发布后,项目将进行开源。** 我们希望把一款真正好用的原生 SSH 运维工具带给社区,并以开源方式长期维护、接受贡献。

> 想关注开源进度,可以 Star 本仓库或加入交流群。

---

## 📥 下载

前往 [**Releases**](../../releases/latest) 获取最新安装包:

**当前最新版本**: `v0.6.7`。

- **Windows**:下载 `Creation-SSH_0.6.7_x64-setup.exe`(推荐)或 `Creation-SSH_0.6.7_x64_en-US.msi` 安装。
- **免安装便携版(推荐,无需安装)**:下载 `Creation-SSH_0.6.7_portable-Windows-x64.zip`,解压即运行,零安装;内含 agent 与静态 tmux 资源,请保持整个文件夹在一起。
  - 首次运行若出现 SmartScreen 提示,点击「更多信息 → 仍要运行」即可。
- **Android**:下载 `C-SSH_0.6.7_android-arm64.apk` 安装。
  - 首次安装需在系统设置中允许「安装未知来源应用」。

> 示例配置一律使用 `example.com` 等占位地址,请替换为你自己的服务器信息。

### v0.6.7 更新重点

- 修复桌面 AI 助手独立弹窗在 Windows 正式包中可能打开白色空窗口的问题。
- 修复 AI 助手弹窗按钮、历史入口和 agent 性能档位的多语言文案显示异常。
- 发布规则新增「发布前真实功能验证」硬门禁:正式包必须实际打开并验证本次改动功能,失败即暂停发布。
- Windows、Android、agent 与公开资产文件名全部同步到 `0.6.7`,并完成签名、版本号、ABI、模拟器安装与 SHA256 校验。

## 🧾 更新列表

- 最新安装包与完整发布说明见 [GitHub Releases](../../releases/latest)。
- 历史更新记录见 [CHANGELOG.md](CHANGELOG.md)。
- Release 说明保持中英双语,包含「新增 / 修复 / 验证 / 下载」。

## 💬 联系 · 交流群

- 微信:**`suiyue_creation`**
- QQ 群【AI 创新社区】:**[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)**

<div align="center">
<img src="screenshots/qq-group-qr.png" width="260" alt="QQ 群二维码 · AI创新社区" />
<br/><sub>扫码加入 QQ 群【AI 创新社区】· 群号 1041937161</sub>
</div>


有问题、建议、想了解 iOS / macOS / 开源进度,欢迎来撩~

---


---

<div align="center">

本仓库仅用于项目简介、截图与安装包分发推广,源代码暂不在此仓(iOS 和 macOS 正式版发布后开源)。

</div>
