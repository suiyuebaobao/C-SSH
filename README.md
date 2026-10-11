# C-SSH 0.9.6

[English](README_EN.md) · [官网](https://c-ssh.com) · [安装与数据保全](INSTALL.md)

面向Windows、M系列Mac、Android和Linux的服务器运维客户端，提供主机、SSH／RDP、持久终端、监控、文件与三作用域AI。本地功能不要求Cloud登录。

本次仅公告、手动下载，不推送升级提醒，不强制更新；iOS不上传，旧版本保留。SQLite仍为22，Android／iOS构建号9006000，wire与签名身份保持。

## Downloads

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

## Added

- 经典界面与新工作台并存；三桌面提供三档同款短卡，尺寸偏好保存。
- 三桌面工作台AI聊天提供语义Markdown、工具摘要、代码高亮与原文复制、阅读位置、引用及有证据的文件／片段差异。
- Windows／Mac支持AI运行中浏览主机与工具卡片；运行与确认仍属于原会话。Linux／手机未宣称完成这两项页面适配。

## Fixed

- 文件、监控等适用页面可在只读加载中切换主机；旧结果和认证回覆保留原请求归属，写入离页／返回仍保留必要保护。
- 删除当前主机后清空相关读取页的失效目标，隐藏监控页不再跳转或自动读取另一台主机。
- 日志显示安全设置的实际新值，减少例行操作与成功自动保存刷屏；修正重复图标、主机卡片及隐藏更新窗口的闲置动画。
- Mac退出等待保存；Linux Agent诊断使用共享DTO转换，拒绝客户端专用偏好字段，wire不变。

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


## Verified

- 十项正式包合计583,386,396字节；实际制品与本轮来源及平台收据绑定。已有Mac／Windows真实本地VM的SSH／SFTP、监控加载切换、离页返回与删除定向回归复用，没有重跑同输入PASS。
- Windows x64／ARM64四包完成各架构包级、安装、0.9.4→0.9.6升级、卸载与资料保持，环境为Windows11 ARM VM；x64 updater epoch3真签名／验签通过，不是Authenticode。
- Mac普通arm64 Release与DMG通过ad-hoc严格签章和包级检查，未公证。Android沿用原release signer，实际Manifest／DEX为0.9.6／9006000，16KiB对齐、API36 arm64模拟器同签名升级及设置／21张受保护表保持通过。
- Linux两架构deb／AppImage通过包级、内嵌、各自Ubuntu24.04 X11隔离启动、日志入口与偏好保持；AppImage采用提取入口，未新增FUSE挂载或全远端业务回归。
- iPhone只完成Simulator构建与身份检查，不上传。物理设备、最低系统实际运行、其它发行版／Wayland、弱网与未测远端流程不由本轮结果推定。

## 数据与源码

Mac仅arm64、ad-hoc且未公证；Windows安装器无Authenticode。Android沿用原正式签名；本轮直接验证0.9.4覆盖升级。不要卸载或清库来尝试解决签名差异。当前schema22不能打开更高schema的测试版资料，需保留原测试版和独立数据。

Cloud只接收允许手动同步的客户端不透明密文；AI对话、原始记录、Skills和长期记忆不上传Cloud。所选模型上下文会发给对应服务商。本公开仓只提供说明、安装包及白名单Cloud生产源码，客户端、共享核心、AI与Agent源码不公开。

[历史界面图集](screenshots/README.md)使用合成示例数据；保留原版本与日期，不作为0.9.6截图或原生验收。

## SHA256

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

[GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Release archive](https://github.com/suiyuebaobao/C-SSH/releases)
