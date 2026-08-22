**中文** | [English](README_EN.md)

<div align="center">

# Creation-SSH（C-SSH）

### 手机上也能接着运维：持久化终端、常驻监控、文件管理与 AI 助手

[![Android](https://img.shields.io/badge/下载-Android-3DDC84?logo=android&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.3/C-SSH_0.8.3_android-arm64.apk)
[![Windows](https://img.shields.io/badge/下载-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.3)
[![Stable](https://img.shields.io/badge/stable-v0.8.3-2ea44f)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.3)

</div>

Creation-SSH 是一套面向 Windows PC 与 Android 的 SSH 运维客户端。Android 不是只读遥控器：它可以直接管理主机、恢复服务端 tmux 持久化会话、查看监控、处理文件、调用 AI 助手和进入系统管理；Windows 端负责更完整的日常运维工作流。

当前提供 Agent 与原生 SSH 两种主机模式：Agent 模式负责 tmux 持久终端与服务端持续监控；SSH 模式无需安装 agent，可使用终端、端口转发、SFTP 文件管理、在线监控、系统管理、应用中心和 SSH AI 工具。当前公开稳定版为 **`v0.8.3`**；旧版本继续保留为历史记录。

> Windows 安全提示：本版 NSIS 与 MSI 尚未使用 Authenticode 签名，Windows SmartScreen 可能显示“未知发布者”或额外确认。请只从本仓库 Release 下载，并用下方 SHA256 核对文件。

> 数据升级提示：`v0.8.3` 使用 Copy-on-Migrate 从 schema 12 前滚到 schema 15。迁移只在数据库副本上执行；主机、凭据、known_hosts、分组、片段、AI/Cloud 数据与必要设置必须保持一致，失败时原数据库不变。

## v0.8.3 更新重点

- 客户端启动时通过 Creation Cloud 统一取得公告、版本、下载和更新策略；公告按普通、重要、紧急三档以整体弹窗显示。
- 管理端可停用指定版本或强制其更新到指定稳定版，并可选择只验证版本号或同时核对当前安装身份 SHA-256。
- 新增 Creation Cloud 自动更新。Windows 自动下载、验签、安全退出、替换并重启；Android 通过系统安装器更新，系统要求时只需完成系统确认。
- Windows 与 Android 统一为 SQLite schema 15。Copy-on-Migrate 在副本上前滚旧数据，验证通过后才切换，失败保留原库。
- 新增独立 Creation-Log 诊断数据库。诊断日志默认关闭，只有用户手动开启后才采集；关闭时不建立或打开诊断库。
- 正式资产恰好为 Windows NSIS、MSI、便携 ZIP 与 Android arm64 APK 四项。Linux 客户端保持冻结；Android 不生成或上传 AAB。

## 当前主要能力

- Windows 与 Android 的本机密钥均由平台设备能力无感包裹；Cloud 数据保护密码不参与普通启动、SSH、AI 或日常使用。
- Creation Cloud 提供账号、登录设备与手动云同步；主机凭据以及 AI provider 的 Key/Token、API 地址和模型绑定仅由账号数据密钥加密，Cloud 不接触明文或使用这些秘密。
- AI 支持本地多 provider 账户和模型绑定；对话与五层记忆只保存在设备本地，不参与云同步。
- AI 权限收敛为查看、编辑、全权三档，并与 Global、Project、Host 三种作用域相互独立；canonical 原始记录与五层记忆在切换模型后继续保留。
- 普通三作用域总结直接使用共享认知快照；只有显式精确取证才按权限读取完整原始对话，认知召回本身不授予远程执行权限。
- AI 瞬时失败重试由共享 Rust 层统一编排，并先持久化每次重试排程；停止操作可以取消等待。
- 群发执行支持 Agent 与原生 SSH 主机混选，并统一逐机结果、失败隔离和确认流程。
- Windows 安装版、便携版和直接运行都使用程序同目录 `data`，文件拖出和下载恢复也受同一隔离数据根约束。
- Windows 的 Frost 主题、紧凑主机状态列、真实延迟和图片消息，以及 Android 的三作用域与图片消息，继续包含在 `v0.8.3`。
- Windows 终端右键菜单只接管现有选区的复制，无选区时不改变远端终端的右键交互。
- Windows 与 Android 提供“联系我们”，可查看微信、QQ 群和 WhatsApp 二维码；移动端 AI 工具栏使用紧凑图标布局。
- 修复数据保护冷启动与忘记密码重置、空云端下载预览、模型配置即时刷新和主机监控状态投影问题。
- 修复 Android 新设备验证弹窗被上层透明提醒遮罩截获触摸，以及数据保护说明出现问号乱码的问题。

## 先看 Android

同一套主机和 tmux 会话可以在桌面与手机之间继续使用。Android `v0.8.3` 只发布 arm64 APK；本版不生成或上传 AAB，公开 Release 也不提供 x86_64 模拟器测试包。

## 下载

| 平台 | 推荐下载 | 其他正式资产 |
| --- | --- | --- |
| Android arm64 | [APK](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.3/C-SSH_0.8.3_android-arm64.apk) | 本版不提供 AAB |
| Windows x64 | [安装版 EXE](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.3/Creation-SSH_0.8.3_x64-setup.exe) | [MSI](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.3/Creation-SSH_0.8.3_x64_en-US.msi) · [便携版 ZIP](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.3/Creation-SSH_0.8.3_portable-Windows-x64.zip) |

### SHA256

- `87DC4A7F07DD39B2D4764BE00DA1726BD8F50437F9ED8F172E7E931AAC254BE5`  `Creation-SSH_0.8.3_x64-setup.exe`
- `0D1BC658622B456F1AFC32066C2B6CA1D8D34813896989203637C64C8D03A8AA`  `Creation-SSH_0.8.3_x64_en-US.msi`
- `333DB9499D851450561995ADA2A982502434027865F66867A524B2D4C20610A8`  `Creation-SSH_0.8.3_portable-Windows-x64.zip`
- `77D383471B1D122100B37C24F4FDA5DC3732FFC9A412C994C12B944C115651CE`  `C-SSH_0.8.3_android-arm64.apk`

### 发布验证

- Windows NSIS、MSI 与 Portable 已分别通过正式安装／替换／重启、数据保留和清场门禁。
- Android 已通过既有真 App、真 SSH、Agent、UI、SQLite 与 arm64 制品门禁；不把 x86_64 模拟器无法运行 arm64 引导包的结果冒充为 Android 自动更新通过。

Linux 客户端已停止开发并冻结；历史源码和历史 Release 仅作为既有记录保留。

版本说明和下载见 [v0.8.3 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.3)，历史记录见 [CHANGELOG.md](CHANGELOG.md)。

## 已交付平台

| 平台 | `v0.8.3` 已交付范围 |
| --- | --- |
| Android | 主机管理、agent 安装与更新/修复、持久化/普通终端、文件上传下载、实时监控、AI、系统管理、本地登录门与“我的”设置 |
| Windows | 完整桌面工作流；提供 EXE、MSI 与便携 ZIP |
| iOS / macOS | **尚未发布**，不属于 `v0.8.3` 已交付范围 |

Linux 客户端不再开发、测试、构建或发布；服务器侧 Linux agent 不属于 Linux 客户端。

## 主要页面

### Android

| 页面 | 能做什么 |
| --- | --- |
| 主机 | 真实 SSH 认证成功后新增主机；编辑主机；只清本机关联状态并明确远端保留；部署/修复 agent，进入终端、监控和系统管理 |
| 终端 | 在可重连的 tmux 持久化终端与普通 SSH PTY 间切换；支持窗口、字体、尺寸、滚动、复制和移动快捷键 |
| 文件 | 浏览、编辑、新建、重命名和删除远端文件；通过 Android SAF 选择单个文件上传或选择下载位置，保留分块传输、断点续传与完整性校验 |
| 监控 | 查看 CPU、内存、磁盘、网络、磁盘 I/O 和 Top 进程；后台跨主机采集设置保存在本地 SQLite |
| AI 助手 | 选择主机、模型与权限档，查看历史和上下文；工具执行受权限与确认控制 |
| 系统管理 | 查看系统信息、进程和防火墙端口，执行杀进程与修改 SSH 密码等需确认操作 |
| 我的 / 设置 | 语言、主题、版本、更新、本机无感保护与可选 Creation Cloud 账号设置 |

### Android 产品截图

以下截图来自 `v0.7.5` Android 测试界面，只使用 RFC 5737、`example.com` 与明确标注的离线模拟数据；未连接真实主机、Cloud 或 AI provider，截图证据不等同于物理 arm64 设备验收或完整人工 GUI 覆盖。

#### 主机管理

<div align="center">
<img width="360" src="screenshots/mobile-hosts.png" alt="Android 主机管理" />
</div>

集中查看主机连接状态和 agent 部署信息，也可以安装、更新/修复 agent，或新增、编辑和删除主机。新增须先通过真实 SSH 认证；删除只结束本机生命周期并明确保留远端 agent、公钥、tmux 和数据。之后即使使用相同 ID 或地址重新添加，也会作为全新主机开始。

#### 持久化与普通终端

<div align="center">
<img width="360" src="screenshots/mobile-terminal.png" alt="Android 持久化与普通终端" />
</div>

在可重连的 tmux 持久化终端与普通 SSH 终端之间切换，并管理当前窗口。持久化会话支持重新附加，方便在手机上继续之前的命令行工作。

#### 文件管理

<div align="center">
<img width="360" src="screenshots/mobile-files.png" alt="Android 文件管理" />
</div>

使用紧凑两行工具栏浏览远端目录、折叠深层路径、新建文件或文件夹并切换隐藏文件。上传通过 Android 系统文件选择器选取单个本地文件，下载同样由用户选择保存位置。

#### 实时监控

<div align="center">
<img width="360" src="screenshots/mobile-monitor.png" alt="Android 实时监控" />
</div>

实时查看 CPU、内存、负载、网络、磁盘、磁盘 I/O 和 Top 进程。页面同时显示监控状态与运行时长，便于在移动端快速判断主机健康情况。

#### AI 助手

<div align="center">
<img width="360" src="screenshots/mobile-ai.png" alt="Android AI 助手模拟演示" />
</div>

选择目标主机、模型和权限档后与 AI 对话，并管理上下文、历史和设置。截图展示明确标注为模拟数据的离线时间线；实际工具执行仍受权限与确认控制。

### Windows 桌面

Windows 提供下列完整桌面入口，并遵循主机硬删除与生命周期隔离语义。

| 页面 | 能做什么 |
| --- | --- |
| 主机管理 | 分组、收藏、搜索、凭据选择，以及 agent 部署、修复和状态查看 |
| AI 助手 | 结合已授权的主机上下文读取指标、日志和文件并执行工具；桌面支持独立 AI 窗口 |
| 终端 | tmux 持久化终端与普通 SSH PTY 双模式，断线或换设备后可恢复持久化窗口；支持打开多个独立终端窗口并行操作 |
| 监控 | 跨主机健康概览、单机实时详情和历史范围查询 |
| 文件 | 远端文件管理、在线编辑、分块传输、断点续传和完整性校验 |
| 端口映射 | SSH 本地转发；默认监听 `127.0.0.1`，可保存、启动和停止映射 |
| 命令片段 | 保存常用命令并对多台主机执行，结果按主机分组 |
| 系统管理 | 系统信息、进程、防火墙端口和 SSH 密码管理 |
| 应用中心 | 安装 Docker，部署 Nginx/Redis 等应用，管理容器、镜像与 systemd 服务 |
| 访问授权 | 查看本地保险库、SSH key、一次性授权和 AI 审计记录 |
| 设置 | AI provider、语言、外观、本地登录、监控采集与更新检查 |

### 桌面产品截图

以下截图来自 `v0.7.5` Windows 候选程序界面，只使用 RFC 5737、`example.com` 与明确标注的离线模拟数据；未连接真实主机、Cloud 或 AI provider，8 张页面截图不代表完整人工 GUI 验收。

#### 主机管理

<div align="center">
<img width="920" src="screenshots/hosts.png" alt="桌面主机管理" />
</div>

通过分组、收藏和搜索统一管理 SSH 主机，并查看 agent 部署状态与运行指标。删除操作会清除该主机可归属的本地凭据、历史会话、窗口持久化和监控缓存，而不让后续新建主机继承旧数据。

#### 持久化与普通终端

<div align="center">
<img width="920" src="screenshots/terminal.png" alt="桌面持久化与普通终端" />
</div>

选择主机后，可在 tmux 持久化会话与普通 SSH PTY 之间切换。普通终端在菜单往返期间保留现场，持久化窗口则可在断线或更换设备后重新附加；右键只在当前确有选区时显示“复制”，无选区时继续交给远端 TUI 或鼠标模式。

#### 跨主机监控概览

<div align="center">
<img width="920" src="screenshots/monitor-list.png" alt="桌面跨主机监控概览" />
</div>

在统一列表中比较多台主机的 CPU、内存与实时状态，并控制主动采集。点击任意主机即可进入对应的监控详情。

#### 单机监控详情

<div align="center">
<img width="920" src="screenshots/monitor.png" alt="桌面单机监控详情" />
</div>

查看单台主机的 CPU、内存、磁盘、Swap、负载、网络和磁盘 I/O。趋势图用于观察近期指标变化，Top 进程帮助定位资源占用来源。

#### 文件管理

<div align="center">
<img width="920" src="screenshots/files.png" alt="桌面文件管理" />
</div>

浏览和搜索远端目录，显示隐藏文件，并执行新建、上传、下载、编辑和刷新。文件拖出与下载支持中断恢复和完整性核对，文件列表提供大小、修改时间以及逐项操作入口。

#### AI 助手

<div align="center">
<img width="920" src="screenshots/ai.png" alt="桌面 AI 助手" />
</div>

选择主机、模型和权限档，让 AI 在明确授权范围内读取指标与系统信息并给出结果。可证明的瞬时只读失败按固定 5 秒间隔最多重试 5 次；可能产生副作用的请求不会自动重放。

#### 群发执行

选择多台主机，输入命令或选择 UTF-8 `.sh` 文件，冻结确认后通过 agent `RunCommand` 协议执行。结果按主机独立收集并默认折叠，可按需显式请求脱敏后的 AI 总结。

#### 访问授权

<div align="center">
<img width="920" src="screenshots/grants.png" alt="桌面访问授权" />
</div>

为指定主机生成独立的临时 SSH 访问密钥，私钥只在创建时展示给接收方。授权记录可随时吊销，避免共享主机长期凭据。

#### 设置

<div align="center">
<img width="920" src="screenshots/settings.png" alt="桌面设置" />
</div>

集中配置跟随系统语言、可选 Creation Cloud 账号、AI provider、外观和监控采集。关于页面提供当前版本与更新检查；Windows 业务状态进入程序同目录 `data`，本机密钥由平台设备能力无感包裹。

## 安全边界

- 私钥和密码先保存在当前设备的本地加密保险库；只有用户明确执行手动同步时，才以账号数据密钥加密后上传不透明密文，Creation Cloud 无法读取或使用这些秘密。
- agent 通过 SSH 隧道访问并只监听服务器本机 Unix socket，不额外暴露公网端口；agent 以当前 SSH 登录身份执行，不自行提权。
- 主机密钥异常会停止连接；新增主机只有在 SSH 认证成功后才写入。删除需要明确确认，但只清本机状态，不连接或清理远端；远端卸载须由用户另行发起。
- 端口映射默认绑定 `127.0.0.1`。如手动改为其他监听地址，局域网暴露风险由用户自行评估。
- AI 工具受权限档和执行确认约束；使用第三方 AI provider 时，用户选定的对话与上下文会按该 provider 的服务条款处理。

## 免费、语言与开源计划

Creation-SSH 当前永久免费，不设订阅、会员或付费功能锁；界面内置简体中文、繁體中文、English、Español、Français、Deutsch、Português、Русский、한국어。

**当前版本尚未开源。** 本仓库只用于项目介绍、截图与 Release 资产分发。计划是在 iOS 与 macOS 正式版发布后公开源代码；这是后续计划，不代表当前仓库已包含源码，也不承诺具体日期。

## 联系

- 微信：`suiyue_creation`
- QQ 群【AI 创新社区】：[点击加入](https://qm.qq.com/q/OWYQ9hwFWy)

### QQ 群【AI 创新社区】

<div align="center">
<img width="300" src="screenshots/qq-group-qr.png" alt="QQ 群二维码 · AI 创新社区" />
</div>

扫描二维码或点击上方链接加入交流群，群号：`1041937161`。这里用于交流使用体验、问题反馈和后续版本计划。
