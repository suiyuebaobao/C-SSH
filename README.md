# C-SSH 0.9.6 — 修訂3

[English](README_EN.md) · [官網](https://c-ssh.com) · [安裝與資料保全](INSTALL.md)


面向Windows、M系列Mac、Android及Linux的伺服器運維客戶端。本地SSH／Agent功能不要求Cloud登入。

只公告與手動下載，不提醒升級、不強制更新。產品版本0.9.6、SQLite26及構建號9006000保持；修訂3使用獨立tag `v0.9.6-r3`，四平台十項，iOS不上传，原0.9.6、修訂2及歷史下載保留。

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,912,113 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,180,565 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,045,545 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,861,101 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,822,796 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,899,610 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,271,530 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,750,264 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,422,970 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,779,528 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_aarch64.AppImage) |

## Added

- 獨立修訂3下載記錄，原0.9.6、修訂2及所有歷史下載保留。

## Fixed

- 新增 Agent 主機時先確認既有 Agent 的相容性；相容者直接復用，不重新上傳、安裝或啟動。缺失才安裝；已確認舊版沿安全升級；未知或未來版本的探測不觸發覆蓋。
- 將身份採納、READY 確認及憑據保存顯示為各自的真實階段；就緒階段失敗不再誤報為初始 SSH 連線失敗，且不自動重放安装。
- 部署進度按當次呼叫、主機與操作隔離，拒絕終態後或離頁後的遲到事件。
- Windows／Mac工作台右上角關閉键與相鄰視窗控件對齊外框、背景及尺寸，保留危险懸停與原退出路徑；Linux繼續使用正常系統標題欄。
- Windows／Mac／Linux的經典及工作台搜尋入口預設收成放大鏡圖示；點擊開啟原搜尋面板，原快捷鍵保留。窄視窗的終端工具列自動換行，避免搜尋與連線操作重疊。

## Verified

- Windows／Mac／Linux 經典與工作台的搜尋按鈕、原面板、快捷鍵及窄視窗排版完成 170 項直接前端檢查、38 組布局；三端前端構建通過。
- 十項正式包已完成對應的來源與包體檢查；九項桌面新包確認包含最終搜尋介面，Windows 四包取得新的實際安裝身份與有效簽名。Mac 為 arm64、ad-hoc 簽章，未公證。
- Agent 實測涵蓋 Mac 三台 Linux 節點、Windows 11 ARM64 的原生 ARM64／相容 x64，以及 Linux 兩架構 VM；本次搜尋修正沿用未變更 Rust 流程的原始證據，不外推為新包全功能重測或 Windows x64 實體機驗收。
- Android arm64 正式 APK 沿用原簽名，已在 SDK Emulator 經正常介面完成既有 Agent 復用、就緒、憑據保存與監控載入；不外推為真機或一般主機刪除驗收。iOS 僅同步源碼與前端，不上傳。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `68c108d74ce246513b87d66a10fb7bd88c2bc729d150394f09b1a1cfdad41de8` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `67e46eae24524e4478c014aaea660e8ad938dafe5269b9059cc8ed73327c95f2` |
| C-SSH_0.9.6_arm64-setup.exe | `b3287cbcb86daac82963015d5abedb2786541c2c1f3e60bac6b2c125831eff0c` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `ffee1c505cbd0883a20babeb51d9cada3173510c48f3247b897b26bbd85f13f5` |
| C-SSH_0.9.6_macOS-arm64.dmg | `28d6962f39104739030dc192d513ae00f088e1bf32930a979ec887d5c05bf000` |
| C-SSH_0.9.6_android-arm64.apk | `a0245a283b45ed9f1a5b4c35091ff419e1cc9bdf3054d4cef3f2488357c5d239` |
| C-SSH_0.9.6_amd64.deb | `a482d1d7c9e452533c4980790b02f88a6d71bf350bd0a231e47216eeaff76a22` |
| C-SSH_0.9.6_amd64.AppImage | `60ef49c4a3e9ac2abada6ad5f2baa0a6a43cb4a513ed5cc3f472fd8b2e42f218` |
| C-SSH_0.9.6_arm64.deb | `117b9915c92af9eecbac46c9c51b37d5414a751fcfe85339f5f32e6be30c4794` |
| C-SSH_0.9.6_aarch64.AppImage | `6528806c24497e6e696a1a038c0025819295c5cc68b9db959d1376d5f9abff14` |

## 資料與源碼

本修訂產品版本仍為0.9.6、SQLite26。以對應平台套件於原位置更新並保留既有資料及系統鑰匙圈／密鑰庫；Mac為ad-hoc簽章、未公證。只公告與手動下載，不推送升级提醒或強制更新。

AI對話、原始紀錄、Skills與長期記憶保留在本機，不參與Cloud同步；所選模型上下文會發給對應服務商。公開倉保存說明、安裝包與既有Cloud白名單源码，客戶端／共用核心／AI／Agent源码不公開。
