# C-SSH 0.9.6 安装与数据保全

[下载](README.md#下载) · [English](INSTALL_EN.md)

0.9.6只发公告，不弹升级提醒、不强制更新；请自行下载并安装。

## 安装前

- 从官网或suiyuebaobao/C-SSH的v0.9.6取得安装包，按首页SHA256核对。
- 退出C-SSH，保全重要数据。Cloud仅手动同步加密配置，不包含AI对话与长期记忆；保留数据保护密码。

## Windows x64／ARM64

- 按本机CPU选x64或ARM64，ARM64首轮验收为Windows11；不要混用两种架构。安装版运行对应setup.exe，沿原安装目录覆盖安装；需要已有WebView2 Runtime。安装器没有Authenticode签名。
- 新便携安装解压后运行C-SSH/C-SSH.exe。
- 从0.9.0及以后版本升级便携版：退出程序，备份旧C-SSH.exe及相邻Windows目录中的旧Setup，原data保留原位；放入新EXE后让它释放新同源Setup。保留旧Setup时，新版会拒绝摘要冲突，不静默覆盖。
- 不在程序运行时替换EXE，不覆盖或删除data；历史MSI须按原安装方式处理，勿直接混用。

## M系列Mac

- 打开C-SSH_0.9.6_macOS-arm64.dmg，将C-SSH.app拖到Applications；替换应用时保留Application Support数据和钥匙串条目。
- 仅Apple Silicon，最低系统目标macOS13；ad-hoc签名、未经公证。首次打开的系统许可参见[Apple官方说明](https://support.apple.com/zh-cn/102445)，不要把系统密码发到反馈中。
- 当前没有Mac自动更新，后续单独开发。

## Android arm64

- 用系统安装器安装C-SSH_0.9.6_android-arm64.apk；仅arm64，最低系统目标Android7/API24。
- 沿用0.9.0正式签名，可覆盖0.9.0／0.9.1／0.9.2／0.9.3保留数据，无需卸载。
- 更早旧签名不能直接覆盖；卸载会清除本地数据，尤其AI对话和记忆不在Cloud同步范围。未确认数据保全前不要卸载或清库。

## Linux x86_64／ARM64

- Debian／Ubuntu按CPU选择amd64或arm64的deb，使用系统包管理器安装。
- AppImage下载后增加可执行权限再启动，普通挂载启动通常需要FUSE；本轮验证采用AppImage提取入口，不宣称FUSE挂载已验收。包名aarch64对应ARM64。
- 当前验证环境为Ubuntu24.04.5／X11，需要可用的桌面Secret Service（例如GNOME Keyring）；Wayland及其它发行版尚未验收。
- 升级时保留原数据目录和系统密钥环，不从其它设备直接搬运加密库替代正常迁移。Linux本次提供手动下载。

## Windows服务器管理组件

在任一发布客户端添加Windows主机并选择DVC管理时保存内置Setup ZIP；Android可通过系统分享交付文件。目标服务器解压并手动运行C-SSH-Windows-Setup.exe，完成正常系统UAC；客户端不执行EXE，不带主机／账号／密码。已有组件可直接连接，纯RDP不要求Setup。

地址直接填IPv4或域名，客户端自动判断，端口单独填写；自建云见[接入说明](SELF_HOSTING.md)。iOS本次无公开下载。


## 本轮数据与验证

Mac仅arm64、ad-hoc且未公证；Windows安装器无Authenticode。Android沿用原正式签名；本轮直接验证0.9.4覆盖升级。不要卸载或清库来尝试解决签名差异。当前schema22不能打开更高schema的测试版资料，需保留原测试版和独立数据。

Cloud只接收允许手动同步的客户端不透明密文；AI对话、原始记录、Skills和长期记忆不上传Cloud。所选模型上下文会发给对应服务商。本公开仓只提供说明、安装包及白名单Cloud生产源码，客户端、共享核心、AI与Agent源码不公开。

- 十项正式包合计583,386,396字节；实际制品与本轮来源及平台收据绑定。已有Mac／Windows真实本地VM的SSH／SFTP、监控加载切换、离页返回与删除定向回归复用，没有重跑同输入PASS。
- Windows x64／ARM64四包完成各架构包级、安装、0.9.4→0.9.6升级、卸载与资料保持，环境为Windows11 ARM VM；x64 updater epoch3真签名／验签通过，不是Authenticode。
- Mac普通arm64 Release与DMG通过ad-hoc严格签章和包级检查，未公证。Android沿用原release signer，实际Manifest／DEX为0.9.6／9006000，16KiB对齐、API36 arm64模拟器同签名升级及设置／21张受保护表保持通过。
- Linux两架构deb／AppImage通过包级、内嵌、各自Ubuntu24.04 X11隔离启动、日志入口与偏好保持；AppImage采用提取入口，未新增FUSE挂载或全远端业务回归。
- iPhone只完成Simulator构建与身份检查，不上传。物理设备、最低系统实际运行、其它发行版／Wayland、弱网与未测远端流程不由本轮结果推定。
