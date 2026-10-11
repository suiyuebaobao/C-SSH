# C-SSH 0.9.6 — 修訂2

[English](README_EN.md) · [官網](https://c-ssh.com) · [安裝與資料保全](INSTALL.md)

面向Windows、M系列Mac、Android及Linux的伺服器運維客戶端。本地SSH／Agent功能不要求Cloud登入。

本次僅公告、手動下載，不提醒升級、不強制更新。產品版本仍為0.9.6，修訂2使用新tag `v0.9.6-r2`；四平台十項，iOS不上传，所有舊Release與原0.9.6資產保持。Android／iOS構建號9006000，wire及簽名身份不變。

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,898,408 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,155,548 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,037,776 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,842,326 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,801,567 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,881,210 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,255,362 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,774,840 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,403,870 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,754,952 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_aarch64.AppImage) |

## Added

- 未明確選擇介面的資料與新安裝預設使用工作台，外觀跟隨系統；已保存的介面與外觀選擇繼續保留。
- 提供獨立修訂2下載記錄，保留原0.9.6與所有歷史下載。

## Fixed

- 修正Mac啟動初始化中止；已驗證的舊schema25資料可前滾到26，既有Cloud會話頁正常載入。
- 修正Mac右上角最大化／還原、最小化與關閉按鈕。
- 通知中心只顯示公告與版本訊息，移除帳號安全類的可見入口及相關未讀提示。
- AI同會話中斷後續問保留已記錄的原文、助手說明及有完整唯一結果的工具紀錄；缺失结果不補造，未完整涵蓋的紀錄不提前標為已覆蓋。
- SQLite前滾到26；已知23–25遺留資料只在嚴格形狀與信任無損檢查成立時接續，不恢復插件執行。

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

- 十項制品共583,805,859位元組，大小與SHA256直接採用本輪制品及傳輸收據。未改的Mac r26／Windows r11 SSH、SFTP、監控切換、離頁返回與UI-R57刪除實鏈按原範圍復用。
- Mac arm64：同Store首候選的實際25→26遷移證據復用；最終r2在已採用26的資料上以普通身份啟動，既有Cloud頁、窗口按鈕與關閉通過。真模型純文字續問保留前文；停止發生在provider context之前且tool=0，未驗真模型的不完整工具輪。Mac為ad-hoc且未公證。
- Windows11 ARM VM：ARM64原生、x64仿真；release原生程序的22→26資料保持、工作台及同會話停止／續問HTTP fixture通過。新普通使用者的installer-only、原公開0.9.6同版本覆蓋、實際EXE身份、data及登記保持通過；四包epoch3真簽名／驗簽通過，沒有Authenticode。新身份Session0的close／exit_app自然收尾門禁失敗保留；ARM64 raw另於Session1的close／destroyed通過，不稱全套formal通過。
- Android arm64正式APK在真正Android SDK Emulator完成同簽名／同9006000覆蓋、22→26、51張Required表與偏好／包裹資料保持，以及fresh工作台／system外觀。AI只驗入口；未驗真手機或一般CleanShutdown。
- Linux兩架構Ubuntu VM：22→26保留57表及12條舊資料；deb後直接FUSE AppImage冷啟與實際業務畫面通過。無獨立模型憑據，未做Linux原生AI續問。
- 不完整工具輪、唯一完整結果與精確覆蓋水位：共享3项集成測試／4個實際本地HTTP場景通過；Windows原生使用受控HTTP fixture。這些不是全平台真模型／遠端工具結果。
- iPhone Simulator實測版本／入口、fresh工作台／system及通知兩類，不公開上传。真機、最低系統實際運行、其它發行版／Wayland、弱網及未測遠端流程均未外推。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `78e084620bf3d6b610de983986d8d6dfb13c38bdb557febdf8ddc8c594edbd8d` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `141ccd8f5f4c900c36ec7e6a71621d75b8f2680f7a6dfbf6d722453ead0e454e` |
| C-SSH_0.9.6_arm64-setup.exe | `a08b6ae653d1741c4d192aa8b13a151ca5b7906bc009b210964703f50352e46b` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `a44878b3726ea7317d2ae0de3443d8d8a4135235a23139f46483b1232e3d9b03` |
| C-SSH_0.9.6_macOS-arm64.dmg | `42e4154303c150d5f31b88d969eb75d3bdcb304c603e0275088b208668038756` |
| C-SSH_0.9.6_android-arm64.apk | `efacd601d0299ec61f122394e5be666b43fd731f9a8ed21ba3b45bfd8e60b88f` |
| C-SSH_0.9.6_amd64.deb | `604f8100e1397bb0ea9dc7a89c74efa4072dc59143da3356e4170cda31efcc37` |
| C-SSH_0.9.6_amd64.AppImage | `bd13eee859d5ac91af86232fcf3f9ed277eb9582a2b58d36bb4b9d82f5b94ea4` |
| C-SSH_0.9.6_arm64.deb | `6db75f6b7ff9dc3d7923524fd2b9de3ab3ea885bdc1281be16e22eb19728789c` |
| C-SSH_0.9.6_aarch64.AppImage | `65a92848bd5c7e34340d9462eeb95436300badbc3c399026e86a7982e4465762` |

## 資料與源碼

本修訂使用SQLite26。請退出程式、保全完整資料並保留現有系統鑰匙圈／密鑰庫條目，再於原位置替換程式；不要手動搬移、刪除或匯出原始密鑰，也不要清庫或以卸載處理啟動／簽名問題。正常22→26及已驗證的已知25資料接續已實測；其它23–25只接受嚴格已知形狀與信任無損的前滾。資料一旦採用26，不要用舊schema22的原0.9.6包打開新庫；回退必須使用相配的舊程式與升級前完整備份。

AI對話、原始紀錄、Skills與長期記憶保留在本機，不參與Cloud同步；所選模型上下文會發給對應服務商。本公開倉保存說明、安裝包與Cloud白名單鏡像，客戶端／共用核心／AI／Agent源碼不公開。
