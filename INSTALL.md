# C-SSH 0.9.9 安装与数据保护

[下载](README.md#downloads) · [English](INSTALL_EN.md)

本版本会提示新版本，请自行前往下载页下载安装；不强制更新，不在背景下载或安装。Android构建号9009000，iOS继续暂停。

## 安装前

- 从官网或suiyuebaobao/C-SSH的v0.9.9取得安装包，按首页SHA256核对。
- 退出C-SSH，保护重要数据。Cloud仅手动同步加密配置，不包含AI对话与长期记忆；保留数据保护密码。

## Windows x64

本次仅发布 Portable ZIP，暂停客户端安装版及新ARM64包。需要已有WebView2 Runtime。

- 新安装：解压后运行`C-SSH/C-SSH.exe`，不要在压缩包内启动。
- 从0.9.8升级：先正常退出，保留相邻`data`及原系统密钥，使用新EXE替换原EXE。已知0.9.7/0.9.8管理Setup会安全换版，不必手工删除；未知文件仍明确报错，不静默覆盖。
- 不在程序运行时替换，不清空资料。旧正式安装包保持原发布记录；本次不提供新NSIS/MSI。

## M系列Mac

- 打开C-SSH_0.9.9_macOS-arm64.dmg，将C-SSH.app拖到Applications；替换应用时保留Application Support数据和钥匙串条目。
- 仅Apple Silicon，最低系统目标macOS13；ad-hoc签名、未经公证。首次打开的系统许可参见[Apple官方说明](https://support.apple.com/zh-cn/102445)，不要把系统密码发到反馈中。
- 本版本只提示并引导手动下载，不执行自动安装。

## Android arm64

- 用系统安装器安装C-SSH_0.9.9_android-arm64.apk；仅arm64，最低系统目标Android7/API24。
- 沿用既有正式签名；兼容且同签名的旧版可保留数据覆盖，无需卸载。
- 更早旧签名不能直接覆盖；卸载会清除本地数据，尤其AI对话和记忆不在Cloud同步范围。未确认数据保护前不要卸载或清库。

## Linux x86_64／ARM64

Linux本次不更新；以下为0.9.8现有下载的使用说明。

- Debian／Ubuntu按CPU选择amd64或arm64的deb，使用系统包管理器安装。
- AppImage下载后增加可执行权限再启动，正常挂载启动需要FUSE；本轮包体验证范围见下方。包名aarch64对应ARM64。
- 当前验证环境为Ubuntu24.04.5／X11，需要可用的桌面Secret Service（例如GNOME Keyring）；Wayland及其它发行版尚未验收。
- 升级时保留原数据目录和系统密钥环，不从其它设备直接搬运加密库替代正常迁移。Linux本次提供手动下载。

## Windows服务器管理组件

在任一发布客户端添加Windows主机并选择DVC管理时保存内置Setup ZIP；Android可通过系统分享交付文件。目标服务器解压并手动运行C-SSH-Windows-Setup.exe，完成正常系统UAC；客户端不执行EXE，不带主机／账号／密码。已有组件可直接连接，纯RDP不要求Setup。

地址直接填IPv4或域名，客户端自动判断，端口单独填写；自建云见[接入说明](SELF_HOSTING.md)。iOS本次无公开下载。


## 数据兼容性

0.9.9使用SQLite28。请保护完整数据及原系统钥匙圈／密钥库，让程序沿正常前向迁移接续；不要删除数据或密钥处理启动问题。新格式数据库不可交给较旧程序开启。Cloud不包含AI对话与记忆；回退需要相配的旧程序及升级前完整备份。实际验证范围见[更新列表](CHANGELOG.md)。
