# C-SSH 0.9.6 修訂2安裝與資料保全

[下载](README.md#downloads) · [English](INSTALL_EN.md)

0.9.6只发公告，不弹升级提醒、不强制更新；请自行下载并安装。

本次僅公告、手動下載，不提醒升級、不強制更新。產品版本仍為0.9.6，修訂2使用新tag `v0.9.6-r2`；四平台十項，iOS不上传，所有舊Release與原0.9.6資產保持。Android／iOS構建號9006000，wire及簽名身份不變。

## 安装前

- 从官网或suiyuebaobao/C-SSH的v0.9.6-r2取得安装包，按首页SHA256核对。
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
- 沿用原正式簽名；本輪直接驗證原0.9.6同版本／同構建號覆蓋。相容且同簽名的舊版可保留資料覆蓋，無需卸載。
- 更早旧签名不能直接覆盖；卸载会清除本地数据，尤其AI对话和记忆不在Cloud同步范围。未确认数据保全前不要卸载或清库。

## Linux x86_64／ARM64

- Debian／Ubuntu按CPU选择amd64或arm64的deb，使用系统包管理器安装。
- AppImage下载后增加可执行权限再启动，正常掛載啟動需要FUSE；本輪已在兩架構Ubuntu VM實測直接FUSE AppImage冷啟。包名aarch64对应ARM64。
- 当前验证环境为Ubuntu24.04.5／X11，需要可用的桌面Secret Service（例如GNOME Keyring）；Wayland及其它发行版尚未验收。
- 升级时保留原数据目录和系统密钥环，不从其它设备直接搬运加密库替代正常迁移。Linux本次提供手动下载。

## Windows服务器管理组件

在任一发布客户端添加Windows主机并选择DVC管理时保存内置Setup ZIP；Android可通过系统分享交付文件。目标服务器解压并手动运行C-SSH-Windows-Setup.exe，完成正常系统UAC；客户端不执行EXE，不带主机／账号／密码。已有组件可直接连接，纯RDP不要求Setup。

地址直接填IPv4或域名，客户端自动判断，端口单独填写；自建云见[接入说明](SELF_HOSTING.md)。iOS本次无公开下载。


## 本輪資料與驗證

本修訂使用SQLite26。請退出程式、保全完整資料並保留現有系統鑰匙圈／密鑰庫條目，再於原位置替換程式；不要手動搬移、刪除或匯出原始密鑰，也不要清庫或以卸載處理啟動／簽名問題。正常22→26及已驗證的已知25資料接續已實測；其它23–25只接受嚴格已知形狀與信任無損的前滾。資料一旦採用26，不要用舊schema22的原0.9.6包打開新庫；回退必須使用相配的舊程式與升級前完整備份。

- 十項制品共583,805,859位元組，大小與SHA256直接採用本輪制品及傳輸收據。未改的Mac r26／Windows r11 SSH、SFTP、監控切換、離頁返回與UI-R57刪除實鏈按原範圍復用。
- Mac arm64：同Store首候選的實際25→26遷移證據復用；最終r2在已採用26的資料上以普通身份啟動，既有Cloud頁、窗口按鈕與關閉通過。真模型純文字續問保留前文；停止發生在provider context之前且tool=0，未驗真模型的不完整工具輪。Mac為ad-hoc且未公證。
- Windows11 ARM VM：ARM64原生、x64仿真；release原生程序的22→26資料保持、工作台及同會話停止／續問HTTP fixture通過。新普通使用者的installer-only、原公開0.9.6同版本覆蓋、實際EXE身份、data及登記保持通過；四包epoch3真簽名／驗簽通過，沒有Authenticode。新身份Session0的close／exit_app自然收尾門禁失敗保留；ARM64 raw另於Session1的close／destroyed通過，不稱全套formal通過。
- Android arm64正式APK在真正Android SDK Emulator完成同簽名／同9006000覆蓋、22→26、51張Required表與偏好／包裹資料保持，以及fresh工作台／system外觀。AI只驗入口；未驗真手機或一般CleanShutdown。
- Linux兩架構Ubuntu VM：22→26保留57表及12條舊資料；deb後直接FUSE AppImage冷啟與實際業務畫面通過。無獨立模型憑據，未做Linux原生AI續問。
- 不完整工具輪、唯一完整結果與精確覆蓋水位：共享3项集成測試／4個實際本地HTTP場景通過；Windows原生使用受控HTTP fixture。這些不是全平台真模型／遠端工具結果。
- iPhone Simulator實測版本／入口、fresh工作台／system及通知兩類，不公開上传。真機、最低系統實際運行、其它發行版／Wayland、弱網及未測遠端流程均未外推。
