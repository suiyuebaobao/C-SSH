**中文** | [English](README_EN.md)

<div align="center">

# Creation-SSH（C-SSH）

### 跨平台 SSH 运维新体验 —— 原生客户端 × 服务端 tmux 持久化 × 常驻监控 × 内置 AI 助手

[![下载 Windows](https://img.shields.io/badge/下载-Windows-0078D6?logo=windows&logoColor=white)](../../releases/latest)
[![下载 Android](https://img.shields.io/badge/下载-Android-3DDC84?logo=android&logoColor=white)](../../releases/latest)
[![支持全球](https://img.shields.io/badge/支持全球-Global-2ea44f)](../../releases/latest)
[![永久免费](https://img.shields.io/badge/永久免费-Free%20Forever-ff69b4)](../../releases/latest)
[![即将开源](https://img.shields.io/badge/开源计划-满500★或iOS完成即开源-success)](../../releases/latest)

</div>

---

## 这是什么

Creation-SSH 不是又一个网页运维面板,也不是普通的 SSH 终端。它把 **Xshell 级的原生客户端体验**、**服务端常驻 agent 的结构化能力**、**tmux 级的会话持久化**三者合一:客户端体验流畅原生,重活由服务器上的常驻 agent 结构化承担,终端会话即使断网、关机、换设备也永不丢失。

一句话:**原生客户端 × 常驻结构化 agent × 持久化会话**,三合一的现代 SSH 运维工具。

---

## 功能一览

<div align="center">

### 主机管理与轻量大盘
<img width="820" src="screenshots/hosts.png" alt="主机管理" />

</div>

集中管理所有服务器,列表内嵌轻量资源大盘,一眼看清每台机器的在线状态与负载概览。分组、搜索、快速连接,凭据本地加密存储、零上传。

<div align="center">

### 双模式终端(持久化 tmux + 直连)
<img width="820" src="screenshots/terminal.png" alt="双模式终端" />

</div>

**持久化模式**由 agent 直驱 tmux,断线、关机、换设备后重连,通过 `capture-pane` 恢复完整屏幕内容,正在跑的任务一行不丢;**直连模式**为纯原生 PTY,未装 agent 也能当普通终端用。两种模式随手切换,兼顾护城河与兜底。

<div align="center">

### 常驻监控(实时六卡 + 历史)
<img width="820" src="screenshots/monitor.png" alt="常驻监控" />

</div>

常驻 agent 持续采集 CPU、内存、磁盘、网络、磁盘 IO、Top 进程六大维度,实时六卡呈现,历史数据落 redb 时序库,可回溯任意时间范围。无需自己搭监控栈,连上即用。

<div align="center">

### 文件管理器(CRUD + 编辑器 + 断点续传)
<img width="820" src="screenshots/files.png" alt="文件管理器" />

</div>

图形化浏览远端文件系统,支持增删改查、在线编辑、权限查看;上传下载分块传输、断点续传,大文件也稳。由 agent 结构化提供文件能力,不靠客户端拼 shell。

<div align="center">

### 应用中心(Docker / systemd 一键装)
<img width="820" src="screenshots/appcenter.png" alt="应用中心" />

</div>

内置应用商城:一键安装 Docker 本身,一键部署 Nginx、Redis 等常用容器应用;结构化管理 Docker 容器与镜像、systemd 服务(启停、查日志)。破坏性操作二次确认,以 SSH 登录身份执行、绝不额外提权。

<div align="center">

### 内置 AI 运维 / 编程助手
<img width="820" src="screenshots/ai.png" alt="AI 助手" />

</div>

内置 AI 助手,能读监控、看日志、写文件、改配置、跑命令,帮你诊断故障、编写脚本。**五档权限模式 + 执行前二次确认**,写入与执行动作全程可控可审计;同时兼容 **OpenAI 兼容接口**与 **Anthropic** 两套 API,模型自选。

---

## 📱 移动端伴侣(Android)

桌面端能力,装进口袋。同一套 tmux 持久化会话、常驻监控与内置 AI 助手,随时随地在手机上继续运维。

<div align="center">
<img width="200" src="screenshots/mobile-hosts.png" alt="主机" />
<img width="200" src="screenshots/mobile-terminal.png" alt="终端" />
<img width="200" src="screenshots/mobile-ai.png" alt="AI 助手" />
<img width="200" src="screenshots/mobile-me.png" alt="设置 · 多语言" />
</div>

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
| iOS | 🚧 开发中 | 客户端开发中,敬请期待 |

---

## 🌍 支持全球 · 💛 永久免费

Creation-SSH **面向全球用户**,界面内置 **9 种语言**(简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어),无论你在哪里都能顺畅使用。

本产品**永久免费**,不设订阅、不卖会员、不锁功能。💛

---

## 🔓 开源承诺

**满 500 个 GitHub Star,或 iOS 客户端开发完成后,项目将全部开源;任一条件先达成即开源。** 我们希望把一款真正好用的原生 SSH 运维工具带给社区,并以开源方式长期维护、接受贡献。

> 想更早开源的话,点一个 ⭐ Star 是最直接的推动。

---

## 📥 下载

前往 [**Releases**](../../releases/latest) 获取最新安装包:

- **Windows**:下载 `Creation-SSH_x.y.z_x64-setup.exe`(推荐)或 `.msi` 安装。
- **免安装便携版(推荐,无需安装)**:下载 `Creation-SSH-portable-Windows-x64.zip`,解压即运行,零安装;内含 agent 与静态 tmux 资源,请保持整个文件夹在一起。
  - 首次运行若出现 SmartScreen 提示,点击「更多信息 → 仍要运行」即可。
- **Android**:下载 `C-SSH-android-arm64.apk` 安装。
  - 首次安装需在系统设置中允许「安装未知来源应用」。

> 示例配置一律使用 `example.com` 等占位地址,请替换为你自己的服务器信息。

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


有问题、建议、想催 iOS / 开源进度,欢迎来撩~

---


---

<div align="center">

本仓库仅用于项目简介、截图与安装包分发推广,源代码暂不在此仓(iOS 完成后全部开源)。

</div>
