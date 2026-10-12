# C-SSH 0.9.6 修訂3安裝與資料保全


[下载](README.md#downloads) · [English](INSTALL_EN.md)

0.9.6只发公告，不弹升级提醒、不强制更新；请自行下载并安装。

本次僅公告、手動下載，不提醒升級、不強制更新。產品版本仍為0.9.6，修訂3使用新tag `v0.9.6-r3`；四平台十項，iOS不上传，所有舊Release與原0.9.6資產保持。Android／iOS構建號9006000，wire及簽名身份不變。

## 安装前

- 从官网或suiyuebaobao/C-SSH的v0.9.6-r3取得安装包，按首页SHA256核对。
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
- 沿用原正式簽名；修訂3最終APK沿用原正式簽名；相容同簽名安裝可保留資料覆蓋。相容且同簽名的舊版可保留資料覆蓋，無需卸載。
- 更早旧签名不能直接覆盖；卸载会清除本地数据，尤其AI对话和记忆不在Cloud同步范围。未确认数据保全前不要卸载或清库。

## Linux x86_64／ARM64

- Debian／Ubuntu按CPU选择amd64或arm64的deb，使用系统包管理器安装。
- AppImage下载后增加可执行权限再启动，正常掛載啟動需要FUSE；本輪套件驗證範圍見下方；直接AppImage啟動需要FUSE。包名aarch64对应ARM64。
- 当前验证环境为Ubuntu24.04.5／X11，需要可用的桌面Secret Service（例如GNOME Keyring）；Wayland及其它发行版尚未验收。
- 升级时保留原数据目录和系统密钥环，不从其它设备直接搬运加密库替代正常迁移。Linux本次提供手动下载。

## Windows服务器管理组件

在任一发布客户端添加Windows主机并选择DVC管理时保存内置Setup ZIP；Android可通过系统分享交付文件。目标服务器解压并手动运行C-SSH-Windows-Setup.exe，完成正常系统UAC；客户端不执行EXE，不带主机／账号／密码。已有组件可直接连接，纯RDP不要求Setup。

地址直接填IPv4或域名，客户端自动判断，端口单独填写；自建云见[接入说明](SELF_HOSTING.md)。iOS本次无公开下载。


## 本輪資料與驗證

本修訂產品版本仍為0.9.6、SQLite26。以對應平台套件於原位置更新並保留既有資料及系統鑰匙圈／密鑰庫；Mac為ad-hoc簽章、未公證。只公告與手動下載，不推送升级提醒或強制更新。

- Windows／Mac／Linux 經典與工作台的搜尋按鈕、原面板、快捷鍵及窄視窗排版完成 170 項直接前端檢查、38 組布局；三端前端構建通過。
- 十項正式包已完成對應的來源與包體檢查；九項桌面新包確認包含最終搜尋介面，Windows 四包取得新的實際安裝身份與有效簽名。Mac 為 arm64、ad-hoc 簽章，未公證。
- Agent 實測涵蓋 Mac 三台 Linux 節點、Windows 11 ARM64 的原生 ARM64／相容 x64，以及 Linux 兩架構 VM；本次搜尋修正沿用未變更 Rust 流程的原始證據，不外推為新包全功能重測或 Windows x64 實體機驗收。
- Android arm64 正式 APK 沿用原簽名，已在 SDK Emulator 經正常介面完成既有 Agent 復用、就緒、憑據保存與監控載入；不外推為真機或一般主機刪除驗收。iOS 僅同步源碼與前端，不上傳。
