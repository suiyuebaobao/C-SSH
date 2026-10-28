# C-SSH 0.9.7

[English](README_EN.md) · [官網](https://c-ssh.com) · [安裝與資料保全](INSTALL.md)

這次更新整合了近期的 AI 助手、伺服器瀏覽器、監控採集、雲同步與四端介面改進，並集中修正實際使用中發現的中斷、等待、資料顯示與連線問題。

**本版本只提示新版本，請前往下載頁自行下載安裝。沒有強制更新，也不會在背景自動下載或安裝。** Windows、macOS、Linux、Android 均採這個方式；iOS 暫停開發及發布。


## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.7_x64-setup.exe | 49,731,494 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_x64-setup.exe) |
| C-SSH_0.9.7_portable-Windows-x64.zip | 45,010,208 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_portable-Windows-x64.zip) |
| C-SSH_0.9.7_arm64-setup.exe | 47,816,201 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_arm64-setup.exe) |
| C-SSH_0.9.7_portable-Windows-arm64.zip | 43,592,452 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_portable-Windows-arm64.zip) |
| C-SSH_0.9.7_macOS-arm64.dmg | 47,725,501 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_macOS-arm64.dmg) |
| C-SSH_0.9.7_android-arm64.apk | 45,779,461 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_android-arm64.apk) |
| C-SSH_0.9.7_arm64.deb | 46,732,072 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_arm64.deb) |
| C-SSH_0.9.7_aarch64.AppImage | 119,740,936 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_aarch64.AppImage) |
| C-SSH_0.9.7_amd64.deb | 47,644,200 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_amd64.deb) |
| C-SSH_0.9.7_amd64.AppImage | 120,789,496 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.7/C-SSH_0.9.7_amd64.AppImage) |

## Added

### AI 任務並行與執行設定

- 可設定 AI 同時執行的獨立唯讀任務數，超出的工作排隊處理；同一台主機上的獨立任務也可並行。
- 子任務沿用主助手的模型上下文配置，改善委派時使用不一致窗口造成的超限。
- 權限模式、工具循環和執行設定集中管理；有依賴的工作及需要確認的修改仍遵守原有執行順序與權限。

### 更清楚的 AI 進度與結果

- AI 回覆支援逐步顯示，子任務的動作和已取得結果會持續更新，減少長時間只有「正在處理」的情況。
- 聊天主畫面保留簡潔的執行摘要，需要查看命令、檔案與個別操作時可展開詳情，避免大量工具行淹沒對話。
- 插話會出現在對話中，進行中的任務可接收補充說明；暫停、停止與重新接續的狀態更明確。

### 會話用量與快取資訊

- 提供累計使用 Token、上下文用量和快取命中率；Android 使用簡潔的三項面板，桌面保留詳細統計。
- 「累計使用 Token」包含主助手和子任務歷次請求的使用量；上下文另附單次輸入說明，避免把累計用量誤認成模型窗口占用。
- 新一輪處理或統計刷新期間保留上次已確認數值。供應商沒有回報的快取資料顯示未知，不當作零命中。
- 兼容不同模型接口的用量欄位，按實際可觀測輸入計算整體快取命中率，避免重複計數。

### 桌面伺服器瀏覽器

- Windows、macOS、Linux 可透過主機 Agent 使用伺服器出口瀏覽網站；桌面版本要求本機安裝 Google Chrome，缺少時會提醒。
- 同一台伺服器可建立多個瀏覽器環境，每個環境分別保存瀏覽資料、固定指紋種子及設定。
- 增加系統、時區、語言與權限等設定；定位、麥克風及攝影機依設定正常詢問權限。
- 改善環境重開及程式異常退出後的恢復，保留各環境既有資料和固定身份。
- Android 的伺服器瀏覽器入口目前隱藏，相關驗證暫停；不將桌面能力標為手機已支援。

## Fixed

### AI 回應速度、上下文與歷史

- 改善簡單主機檢查反覆讀取目錄、重複查詢及不必要工具循環的情況。
- 優化上下文準備、穩定前綴和歷史載入，依模型實際窗口處理上下文，避免固定小窗口限制影響大窗口模型。
- 調整工具結果處理與長會話儲存，在保留原始記錄的前提下減少一次載入、傳輸和渲染的負擔。
- 修正暫停或中斷後無法再送訊息、會話狀態誤判為由其他視窗接手，以及重開後看不到既有內容等問題。
- 修正部分 AI 執行路徑造成的 Mac 程式崩潰。
- 修正 Android 歷史對話選擇受其他會話更新干擾、恢復歷史時覆蓋當前模型，以及修改模型窗口後統計未更新的問題。

### 監控採集與資料庫

- 監控採集併發設定上限提高至 1000，搭配排隊和獨立主機調度，減少慢主機拖延整輪刷新。
- 優化本機資料庫的讀寫與高頻採集路徑，減少重複開庫及不必要的資料處理。
- 為適合的通訊資料採用選擇性壓縮；小回應維持直接傳輸，減少為壓縮小封包付出的額外成本。
- 修正雲同步後 Agent 狀態未及時更新、需要重開程式才顯示正確，以及初次採集狀態容易造成誤解的情況。
- 改善 Windows Server Agent 的監控採樣成本與 CPU 使用情況。

### 瀏覽器啟動與穩定性

- 改善首次啟動偶發失敗、參數準備耗時與重試操作。
- 修正部分驗證頁、頁面事件及導航等待造成瀏覽器意外關閉的情況。
- 瀏覽器環境之間分開保存 Cookie 與其他資料，固定身份不再因每次開啟而重新生成。
- 網站是否顯示驗證碼仍由網站及出口網路狀況決定，本更新不保證所有網站免驗證，也不宣稱完全隱藏所有本機指紋。

### Agent 接入與 ARM 相容性

- 改善已有 Agent 的跨設備接入與專用公鑰處理，減少新設備連線時出現授權衝突。
- 改善 Agent 檢測、復用、更新與就緒階段的提示，讓使用者能看出目前正在處理哪一步。
- 更新 Linux x86_64／ARM64 Agent，改善獨立通道相容性，並整理統信 UOS、麒麟 ARM 的實際適配結果。
- 保留 Agent／SSH 模式邊界，純 SSH 主機不因更新而自動安裝 Agent。

### 手動雲同步與資料保護

- 同一程式進程內完成資料保護驗證後，切頁、檢查及後續同步可復用有效驗證，不反覆要求輸入相同密碼；退出重開後重新驗證。
- 增加全選及預設使用雲端的選擇，改善同步列表布局和操作確認。
- 修正 Android 首次登入缺少模型目錄快取時，點「確認下載」造成整批下載失敗的問題；下載會自行取得所需目錄。
- 本機已沒有對應 AI 帳戶的刪除記錄不再顯示為待下載配置，也不阻擋同批主機下載；內部仍保留必要的同步確認，避免舊設備重新帶回已刪除資料。
- 已有本機資料的刪除衝突仍保留明確選擇；不按名稱猜測或刪除其他帳戶。雲同步繼續由使用者手動發起，不自動上傳 AI 對話和記憶。

### 介面、終端與 Android

- 設定入口整合為四組卡片，保留原功能，子項可直接進入對應設定。
- 改善三桌面的自繪視窗控制、Mac 圓角、主題及彈窗一致性，修正部分深色介面文字對比不足的問題。
- Android 移除重複的上下文區域，修正底部安全區重複留白及鍵盤相關布局。
- 修正從主機功能卡片進入終端時選錯主機，以及第一條連線等待時無法切換主機／終端的問題。
- 切換後拒絕舊連線遲到的錯誤與認證提示，減少它們干擾目前選中的終端。

## Verified

上述改動沿用各自直接範圍的共享層、介面及原生驗證結果，本次發布另外檢查新版本制品的版本、架構、內嵌資源、簽名與封裝。沒有將既有局部結果稱為所有平台、所有發行版或所有實體手機的全量重新驗收。

Android 首次登入雲下載修正已用授權測試帳戶，在正常 Android SDK Emulator 客戶端完成主機與 AI 配置下載及後續檢查；不依賴先進 AI 設定或重開程式預熱。

macOS 僅提供 Apple Silicon 版本；目前仍為 ad-hoc 簽章，未取得 Developer ID 公證。Android 保持既有 APK 簽名及資料相容性。請在更新前保留重要資料，資料格式升級後不要使用較舊的程式開啟新格式資料庫。

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.7_x64-setup.exe | `5d372e069ec70ec43d04e4e7219bb0aae7990c6620a3ec9bc3a996c8321ed269` |
| C-SSH_0.9.7_portable-Windows-x64.zip | `2d6b8418195887ef62ba3bd7e09ca8886d9bea7126e33a02df9d8bb9ebac6be0` |
| C-SSH_0.9.7_arm64-setup.exe | `63db71cae80b4f2772ee9c7e8fcc5c96edcc1a982b5a797002b3e342edd69bc6` |
| C-SSH_0.9.7_portable-Windows-arm64.zip | `834fc184cc0c234f22f5b2acbb8d7bb7d885b3e806da31d79ca83378250a7567` |
| C-SSH_0.9.7_macOS-arm64.dmg | `8315451283855bf352b786161e4645a01671f8914db73d7c60026708d6f856c9` |
| C-SSH_0.9.7_android-arm64.apk | `f425c11e4d60f134fc9cef1f099a73c9f8f57bd30e412d9024200f7698b5edae` |
| C-SSH_0.9.7_arm64.deb | `15285fd9df875da5ce5538e63941840289b7004a2be3cf3313420fa2909181da` |
| C-SSH_0.9.7_aarch64.AppImage | `15042627ac7499241c49b8ebe56c07690d88973aabf4128e1891475bad044fab` |
| C-SSH_0.9.7_amd64.deb | `e0f9638ba7123fb07b4a158fd6b3090c4fd07bd6ee930320743b83e8fea01a8a` |
| C-SSH_0.9.7_amd64.AppImage | `e8f9626789395baf6bccd2d00995f4d91326b6812b2321037184bd0350c4b733` |

## 資料與源碼

AI 對話和記憶保存在本機；Cloud 僅同步使用者選擇的加密配置。本倉庫提供公開說明、下載及 Creation Cloud 生產源碼鏡像，客戶端與 Agent 源碼不在公開範圍。
