# C-SSH 0.9.2 安装与数据保全

[下载](README.md#下载) · [English](INSTALL_EN.md)

0.9.2只发公告，不弹升级提醒、不强制更新；请自行下载并安装。

## 安装前

- 从官网或suiyuebaobao/C-SSH的v0.9.2取得安装包，按首页SHA256核对。
- 退出C-SSH，保全重要数据。Cloud仅手动同步加密配置，不包含AI对话与长期记忆；保留数据保护密码。

## Windows x64／ARM64

- 按本机CPU选x64或ARM64，ARM64首轮验收为Windows11；不要混用两种架构。安装版运行对应setup.exe，沿原安装目录覆盖安装；需要已有WebView2 Runtime。安装器没有Authenticode签名。
- 新便携安装解压后运行C-SSH/C-SSH.exe。
- 从0.9.0／0.9.1升级便携版：退出程序，备份旧C-SSH.exe及相邻Windows目录中的旧Setup，原data保留原位；放入新EXE后让它释放新同源Setup。保留旧Setup时，新版会拒绝摘要冲突，不静默覆盖。
- 不在程序运行时替换EXE，不覆盖或删除data；历史MSI须按原安装方式处理，勿直接混用。

## M系列Mac

- 打开C-SSH_0.9.2_macOS-arm64.dmg，将C-SSH.app拖到Applications；替换应用时保留Application Support数据和钥匙串条目。
- 仅Apple Silicon，最低系统目标macOS13；ad-hoc签名、未经公证。首次打开的系统许可参见[Apple官方说明](https://support.apple.com/zh-cn/102445)，不要把系统密码发到反馈中。
- 当前没有Mac自动更新，后续单独开发。

## Android arm64

- 用系统安装器安装C-SSH_0.9.2_android-arm64.apk；仅arm64，最低系统目标Android7/API24。
- 沿用0.9.0正式签名，可覆盖0.9.0／0.9.1保留数据，无需卸载。
- 更早旧签名不能直接覆盖；卸载会清除本地数据，尤其AI对话和记忆不在Cloud同步范围。未确认数据保全前不要卸载或清库。

## Linux x86_64／ARM64

- Debian／Ubuntu按CPU选择amd64或arm64的deb，使用系统包管理器安装。
- AppImage下载后增加可执行权限再启动，需要FUSE；包名aarch64对应ARM64。
- 当前验证环境为Ubuntu24.04.5／X11，需要可用的桌面Secret Service（例如GNOME Keyring）；Wayland及其它发行版尚未验收。
- 升级时保留原数据目录和系统密钥环，不从其它设备直接搬运加密库替代正常迁移。Linux本次提供手动下载。

## Windows服务器管理组件

在任一发布客户端添加Windows主机并选择DVC管理时保存内置Setup ZIP；Android可通过系统分享交付文件。目标服务器解压并手动运行C-SSH-Windows-Setup.exe，完成正常系统UAC；客户端不执行EXE，不带主机／账号／密码。已有组件可直接连接，纯RDP不要求Setup。

地址直接填IPv4或域名，客户端自动判断，端口单独填写；自建云见[接入说明](SELF_HOSTING.md)。iOS本次无公开下载。
