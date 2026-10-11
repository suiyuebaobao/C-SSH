**中文** | [English](CHANGELOG_EN.md)

# 更新列表


## 0.9.6 — 双界面、AI聊天与主机切换修正

2026-10-11

本次仅公告、手动下载，不推送升级提醒，不强制更新；iOS不上传，旧版本保留。SQLite仍为22，Android／iOS构建号9006000，wire与签名身份保持。

### Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,881,563 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,128,433 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,030,473 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,816,423 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,668,120 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,849,374 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,197,684 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,733,880 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,350,070 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,730,376 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_aarch64.AppImage) |

### Added

- 经典界面与新工作台并存；三桌面提供三档同款短卡，尺寸偏好保存。
- 三桌面工作台AI聊天提供语义Markdown、工具摘要、代码高亮与原文复制、阅读位置、引用及有证据的文件／片段差异。
- Windows／Mac支持AI运行中浏览主机与工具卡片；运行与确认仍属于原会话。Linux／手机未宣称完成这两项页面适配。

### Fixed

- 文件、监控等适用页面可在只读加载中切换主机；旧结果和认证回覆保留原请求归属，写入离页／返回仍保留必要保护。
- 删除当前主机后清空相关读取页的失效目标，隐藏监控页不再跳转或自动读取另一台主机。
- 日志显示安全设置的实际新值，减少例行操作与成功自动保存刷屏；修正重复图标、主机卡片及隐藏更新窗口的闲置动画。
- Mac退出等待保存；Linux Agent诊断使用共享DTO转换，拒绝客户端专用偏好字段，wire不变。

### Verified

- 十项正式包合计583,386,396字节；实际制品与本轮来源及平台收据绑定。已有Mac／Windows真实本地VM的SSH／SFTP、监控加载切换、离页返回与删除定向回归复用，没有重跑同输入PASS。
- Windows x64／ARM64四包完成各架构包级、安装、0.9.4→0.9.6升级、卸载与资料保持，环境为Windows11 ARM VM；x64 updater epoch3真签名／验签通过，不是Authenticode。
- Mac普通arm64 Release与DMG通过ad-hoc严格签章和包级检查，未公证。Android沿用原release signer，实际Manifest／DEX为0.9.6／9006000，16KiB对齐、API36 arm64模拟器同签名升级及设置／21张受保护表保持通过。
- Linux两架构deb／AppImage通过包级、内嵌、各自Ubuntu24.04 X11隔离启动、日志入口与偏好保持；AppImage采用提取入口，未新增FUSE挂载或全远端业务回归。
- iPhone只完成Simulator构建与身份检查，不上传。物理设备、最低系统实际运行、其它发行版／Wayland、弱网与未测远端流程不由本轮结果推定。

### SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `0d87b5a42e4b7196877fd726170aead75d7b7714c9393efc4c1a9770e95d884b` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `ab15668c497e2009fcfa8a5efe2696546b866577a1855afaf5f7cbd44ce9ca20` |
| C-SSH_0.9.6_arm64-setup.exe | `538993bfa604d8e00af58b28273c8f23c56efe85431633dafbb93cd5279ac0b8` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `fa1b4287038c435083c4c13e12eb4194180fbe9f05ea40b24b26b3b6b4db38b8` |
| C-SSH_0.9.6_macOS-arm64.dmg | `c0fd388fcc7b65693ebe5c71c59efaa45ea1798b72791a61e67480961ec21dd9` |
| C-SSH_0.9.6_android-arm64.apk | `423be8b7227cfb026b87413422bc822daeff81527bb1f3588482b81f3a98aac7` |
| C-SSH_0.9.6_amd64.deb | `608baecb6e9106060e2aca7f5af424c6c7f2629136bd27a789c49f473f529980` |
| C-SSH_0.9.6_amd64.AppImage | `e0bd964461331bb790f41f5e6683ee3bfe4e6774f5698e682d6cff2bbb2d8a13` |
| C-SSH_0.9.6_arm64.deb | `fa17a74b7d21e4bb895ac1296c4c96c282e4e58dbd296eadc60c25a078b7677d` |
| C-SSH_0.9.6_aarch64.AppImage | `08a00b83da19789ff30ea94cc7aa8ddb87a0dfba59e1061ef661f914512eab79` |

## 0.9.4 — 导入、首次引导、自动运维Skills与逐行日志

2026-10-04

0.9.4只通过公告告知，请手动下载并安装；不弹升级提醒、不强制更新。Mac仅M系列arm64、最低目标macOS13，采用ad-hoc签名且未公证；Windows安装器没有Authenticode签名。Android沿用0.9.0以后的正式signer；更早不同签名版本不能直接覆盖。AI内容不参与Cloud同步，请先阅读安装与资料保全说明。

### Downloads

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

### Added

- **批量导入主机**：支持CSV和OpenSSH配置。先预览、筛选与确认，再建立“待接入”项；补充凭据并通过真实认证后才成为正式主机。导入预览不等于已连接，密码与私钥不写入明文待接入队列。
- **可跳过的首次引导**：首次使用可了解接入、导入与运维入口，也可直接跳过进入工作台；选择在冷启动后保持，设置中可以重新打开。已有资料升级不强制重新引导。
- **自动AI运维Skills**：工具实际执行且验收成功的经验才自动学习；当前环境匹配后在后续任务中自动参考。无需手动保存或采纳，普通界面只显示一次“参考过往经验”的轻提示，原有权限和操作确认继续生效。Skills、AI对话、原始记录和长期记忆均保留在本机，不参与Cloud同步。
- **易读的逐行操作日志**：一次业务操作对应一行，直接查看时间、级别、本机主机名称和可信结果。采集默认关闭，可按需开启、查询，切换技术视图及导出支持材料；日志不会把子步骤或传输成功外推为整次成功，未知或不完整结果明确显示。
- **版本与资料升级**：四个平台发布0.9.4，客户端SQLite为22；iOS同步维护0.9.4／9004000，本次不上传iOS安装包。

### Fixed

- **霜白主题按钮可读性**：修正Windows、Mac和Linux主按钮的底色、文字、悬停、禁用与键盘焦点状态。
- **Android新装冷启动**：修正先前0.9.4候选包在冷启动时被错误拒绝的问题；本次正式APK已在新的arm64 Android SDK Emulator完成冷启动与正常界面验证，沿用原正式安装签名。旧候选的失败记录保留。

### Verified

- 本次十项正式包的文件名、大小及SHA256与平台收据逐一绑定，合计580,358,528字节。Windows双架构为当前Release与包级门禁，Portable仅有`C-SSH/C-SSH.exe`；本轮未重复安装生命周期或普通用户资料启动。
- Mac为普通arm64 Release与DMG，ad-hoc签名检查通过，未公证；复用同源Mac业务验证，不启动用户普通资料。Android正式APK验证0.9.4／9004000、原signer、16KB对齐、fresh SDK Emulator冷启动及正常操作日志行。
- Linux双架构deb／AppImage完成包级和隔离最小启动；AppImage使用提取入口。没有新增FUSE挂载、升级、其它发行版或物理机器验证。
- 导入、引导和自动Skills复用10-02／10-03专项原生证据：Mac arm64；Windows ARM64及ARM VM中的x64；Linux ARM64，x64仅其原生差异；Android SDK Emulator及iPhone Simulator。SSH／Windows管理与真实provider学习／冷启复用按各自范围成立，不宣称每个架构都重跑完整矩阵。
- 逐行操作日志的Mac同源实际业务证据及Android正常行已完成；其它平台采用当前共享实现、前端／包级与对应最小启动证据，不宣称全平台所有业务日志都已复测。
- 不宣称Android或iPhone真机、全部输入法、全部Linux发行版／Wayland、所有断网恢复或本轮未实测功能完成验收。

### SHA256

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

## 0.9.3 — 终端键盘与表单操作修复

2026-10-02.

- **终端键盘避让**：Android 自适应终端随键盘可见区域调整，修正游标和输入焦点定位；快捷键保持在键盘上方，展开时为输入行留出空间。
- **更易关闭表单**：新增／编辑主机表单增加右上关闭按钮，长表单滚动时仍可操作；点击表单外侧不会丢失输入。
- **移动终端字号**：Android／iOS 默认字号为 6，保留已经保存的自定义字号。
- **版本同步**：iOS 同步维护 0.9.3，但本次不上传 iOS 安装包。
- **手动安装**：本次只发公告，请自行下载安装；不弹升级提醒、不强制更新。

## 0.9.2 — Linux与Windows ARM64

2026-10-02。四个平台、十项安装包；iOS不上传。

- **Linux 桌面客户端**：新增 x86_64／ARM64 的 deb 和 AppImage，复用桌面主机、SSH／Agent、持久终端、RDP／DVC、监控、文件、AI、代理、端口映射和群发能力。
- **Windows ARM64 原生版**：提供 NSIS 和 Portable，已在 Windows 11 ARM64 的普通用户环境验证；远端 Windows 管理组件继续使用 x64。
- **Windows 旧系统终端**：配套 psmux 增加 WinPTY 后端，供缺少 ConPTY 的 Server 2016 使用；现代 Windows 保持 ConPTY。管理组件使用静态 CRT，减少干净系统上的运行库依赖。
- **稳定性修复**：主机编辑／删除等待、Windows 域名接入、文件覆盖权限收尾及 Linux 旧库迁移。
- **公告更简洁**：移除公告界面的同步消息列表，云同步功能保留。
- **手动下载安装**：本次通过公告告知，不弹升级提醒、不强制更新。

## 0.9.1 — 地址自动识别与Windows安装包导出

2026-09-30。Windows x64 NSIS／Portable、M系列Mac arm64 DMG、Android arm64 APK；仅公告告知手动下载安装，不弹升级提醒、不强制更新。iOS不上传，Linux客户端冻结。

新增：
- 直接输入IPv4或域名并自动识别，移除地址类型下拉选择；端口独立，既有IPv6主机兼容保持。
- 三端内置Windows服务器管理Setup ZIP，DVC管理、组件缺失与维护入口可离线导出；Android另有系统分享。纯RDP隐藏，不带主机或凭据，不在客户端运行EXE。
- 组件版本0.9.1，ZIP13,308,604字节；Setup与Agent／psmux维护继续同源。

修复与验证：
- Windows新旧Setup冲突继续失败关闭，Portable升级说明明确备份旧EXE／Setup并保留data；NSIS与Portable普通用户手动升级及数据保持通过。
- Mac完成arm64、包版本、ad-hoc签名、DMG挂载与唯一内嵌资源核对；未公证，无自动更新，不外推最低系统运行或全部M系列。
- Android沿用0.9.0签名，覆盖安装、启动、设置保持及系统保存／分享在SDK Emulator定向验证；不宣称物理手机或收件人已收到。
- 更早Android正式签名仍可能不能覆盖；卸载会清除本地AI历史和记忆，需先保全。0.9.0用户无需卸载。

[安装说明](INSTALL.md) · [新增入口示例](screenshots/v0.9.1/README.md) · [v0.9.1下载](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.1)


[Historical changelog 1](CHANGELOG_HISTORY_1.md)

[Historical changelog 2](CHANGELOG_HISTORY_2.md)
