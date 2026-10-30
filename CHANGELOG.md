**简体中文** | [English](CHANGELOG_EN.md)

# 更新日志

## 0.9.8 — 主机接入、云服务与稳定性更新

2026-10-30

本次更新面向 Windows、macOS、Linux 和 Android，集中处理主机接入、第三方云连接、Android 会话恢复，以及删除和诊断记录相关问题。客户端与自研 Agent 同步更新至 0.9.8。

### 1. 主机连接失败，提示更清楚

新增主机时，连接错误改用 C-SSH 自身样式的弹窗显示，并按实际失败阶段区分地址解析、网络连接、SSH 握手、身份验证和代理认证问题，方便判断应该检查哪一项。

关闭错误提示后保留已填写的连接资料，便于修改后重试。身份验证被服务器拒绝时，不再笼统地判定为“密码错误”；用户名、密钥和服务器认证策略也可能影响结果。

### 2. 相同 IP、不同 SSH 端口可以分别管理

修正相同地址使用不同 SSH 端口时，新增或同步主机可能被误认为重复的问题。现在按照地址和 SSH 端口识别连接端点，各自主机资料、凭据、信任记录与云同步映射保持独立。

已有主机的身份保持，不要求删除后重新添加，也不会因为新增另一个端口而覆盖原来的凭据。

### 3. 第三方云地址自动识别

在“设置 → 云服务 → 自建云”中，可以直接填写 IP、域名或带端口的地址，点击“测试连接”后查看识别结果，再保存并切换；不必每次手动输入协议前缀。

首次未指定协议时优先检测 HTTPS。经明确配置的可信内网云可使用 HTTP，不需要在每台访问设备上导入证书。界面会清楚显示实际协议，并提示 HTTP 传输未加密，仅适合可信内网。

明确指定或已经保存的 HTTPS 地址、HTTPS 证书验证失败，以及公开网络地址，都不会悄悄降级为 HTTP。连接检测不会携带原账号的密码或会话信息。官方云继续使用 HTTPS；自建服务端需采用支持此模式的新版本并明确启用内网 HTTP。

### 4. Android 锁屏、断网后的 AI 会话恢复

改善 Android 在锁屏、网络中断、重新联网或返回前台后，AI 会话无法继续发送、错误提示无法识别，以及保存状态未及时恢复的问题。

恢复时会核对原任务和本机保存状态，保留已收到的内容，并允许在原会话继续交流；不会自动重发模型请求或远端修改命令。修正相关保存回执的处理时序，减少中断后聊天内容与页面状态不一致的情况。

### 5. 减少删除主机时的不必要等待

修正删除带有服务器浏览器环境的主机时，清理流程重复等待自身资源锁，可能造成长时间停留在“正在删除”的问题。

删除完成后正确释放资源，再执行浏览器环境收尾；远端短时卸载尝试仍有时间限制，失败不会无限阻挡本机删除。

### 6. 降低 Agent 诊断记录的开销

诊断关闭时更早跳过记录处理；Linux Agent 对正常、短时的监控操作采用汇总记录，异常、慢操作和需要追踪的业务仍保留必要细节。

日志维护改为有界的周期任务，减少在请求处理中进行较重清理的机会。Windows Agent 同步调整相关维护调度。此优化针对诊断记录和维护开销，不降低实际监控采样频率，也不改动服务器其他程序的日志设置。

### 7. 云端后台账号删除修复

修正删除包含主机、AI 配置、代理或同步历史的账号时，可能被设备关联约束阻挡的问题。

相关同步依赖与账号在同一笔数据库事务中清理；出现错误时完整回滚，其他账号的数据保持不变。管理员保护、全局历史记录和审计要求继续保留。官方云已同步此项后端修正，自建云需更新服务端才能采用。

### 版本与更新方式

- Windows、macOS、Linux、Android 客户端及自研 Agent：0.9.8。
- 本次通过公告提示更新，请前往官网下载并手动安装；不强制更新，不自动下载安装。
- 使用原安装身份覆盖更新，保留既有本机资料；请勿为更新而卸载、清空数据或手动删除密钥。
- macOS 继续提供 Apple Silicon 版本；当前未取得 Apple Developer ID 公证，安装边界与上一版相同。
- iOS 开发及发布继续暂停；移动端服务器浏览器入口仍保持隐藏。

## 2026-10-30 — Cloud 后台账号删除修复

修复删除包含主机、AI 配置、代理及同步历史的账号时，被设备关联约束阻挡的问题。同步依赖与账号在同一事务中清理；发生错误时完整回滚，其他账号数据保持不变。

已通过隔离数据库的账号隔离、回滚与并发验证，以及内网实例的真实删除 API 验证；官方云已部署同一修复。此为服务端维护更新，不改变客户端版本、HTTPS 设置或数据库结构，既有安装包保持。

## 0.9.7 — 麒麟适配、AI、服务器浏览器、监控与同步改进

2026-10-28

本次更新集中改善服务器接入、AI 助手、服务器浏览器、监控采集与云端同步，并同步更新 Windows、macOS、Linux 和 Android 客户端。

### 1. 麒麟与 ARM 服务器适配

完善麒麟高级服务器操作系统 V10 ARM64 适配，支持其 64KiB 内存页环境。改善 Agent 接入、监控采集、文件操作、持久终端与命令执行的兼容性。

修正特定 SSH 环境下首次建立 Agent 通道失败，以及日志量较大时兼容性检测失败的问题。ARM Agent 同步更新至当前版本。

### 2. Agent 接入与多设备使用

更新 Linux x64、Linux ARM64 及 Windows Server Agent。

改善多台电脑、手机接入同一服务器的授权与复用流程，修正部分公钥授权冲突。完善已有 Agent 的识别、能力检测与更新流程，并修正云端同步后主机部署状态未及时刷新的问题。

### 3. 桌面服务器浏览器

Windows、macOS、Linux 支持通过服务器 Agent 使用本机 Chrome 浏览网站。

同一台服务器可建立多个浏览器环境，每个环境独立保存 Cookie、登录状态、浏览数据与固定指纹种子，并可设置显示系统、时区、语言，以及定位、麦克风和摄像头权限。

改善启动速度与首次打开成功率，修正部分网站、安全验证页导致浏览器意外关闭的问题，完善异常退出后的环境恢复，并增加未安装 Chrome 时的提示。

### 4. AI 并发执行与查询效率

增加可设置的 AI 任务并发与排队机制，支持多台主机，以及同一台主机上的独立只读任务同时执行。子 Agent 沿用主 Agent 本次设置的上下文窗口。

优化服务器状态检查流程，减少重复读文件、反复查询及不必要的工具步骤。有依赖关系或涉及修改的操作仍依序处理。

### 5. AI 即时回答、进度与会话恢复

回答随模型返回逐步显示，子任务进度、结果与错误及时更新。

聊天主界面改用精简的进度与结果摘要，完整工具操作可通过“查看详情”展开。

修正插话内容未显示、暂停后无法继续发送，以及中断、异常退出后部分会话无法正常恢复的问题。改善历史保存冲突判断，修正部分 Mac AI 崩溃问题。

### 6. AI 上下文、长会话与用量统计

上下文整理改为依据模型实际窗口，不再采用固定 256K 门槛；工具结果不再一律截取固定字数，大结果可按需分页读取。

长会话采用分块保存与加载，处理原先整份会话 256MiB 限制造成的阻断，减少重新打开长聊天与准备上下文的等待。

新增真实用量统计，显示 **累计使用 Token、上下文用量与缓存命中率**，统一处理不同模型接口的用量字段，并计入子任务。执行期间继续显示上次已确认数据，供应商未提供的缓存数据不再误作零命中。

“累计使用 Token”表示会话历次请求的累计用量；“上下文用量”表示主助手单次请求的输入量，两者分开说明。

### 7. 监控采集与数据传输优化

采集并发上限提高至可设置的 **1,000**，完善排队与调度，改善启动后首轮数据更新及慢主机影响其他主机的情况。

优化本机数据库读写，减少重复操作；较大的监控历史响应采用选择性压缩，降低传输量，小数据维持轻量传输。

### 8. 云端同步与密码验证

同一次程序运行期间，完成一次数据保护密码验证后，切页、检查、上传与下载可沿用验证，减少重复输入。

同步界面增加全选与默认采用云端选项，改善项目排列和操作提示。

修正 Android 首次登录后确认下载失败的问题；已删除且本机也不存在的 AI 账号，不再反复出现在待同步列表中。

### 9. Android 界面与操作修复

同步共享层的 AI、上下文、记忆与用量统计改动，使用底部用量摘要及简洁的三项详情。

修正底部大面积空白、键盘与安全区间距问题。修正从主机功能卡片进入终端时未选中对应主机，以及连接等待期间无法切换主机的问题。

修正 AI 历史对话无法选取、历史模型干扰当前设置，以及切换模型窗口后用量未刷新的问题。

### 10. 桌面界面统一与细节改善

Windows、macOS、Linux 的共用设置与 AI 界面统一调整。设置首页整合为四个分类卡片，子项可直接进入对应设置。

确认弹窗改用 C-SSH 自身样式，改善 Mac 窗口控制、圆角，以及部分深色弹窗文字不清楚的问题。

**更新方式：**本次版本只提供更新提示，请前往官网下载并自行安装；不强制更新，也不自动下载安装。

## 0.9.6 — 修訂3：Agent檢測復用與就緒進度

2026-10-12


只公告與手動下載，不提醒升級、不強制更新。產品版本0.9.6、SQLite26及構建號9006000保持；修訂3使用獨立tag `v0.9.6-r3`，四平台十項，iOS不上传，原0.9.6、修訂2及歷史下載保留。

本修訂產品版本仍為0.9.6、SQLite26。以對應平台套件於原位置更新並保留既有資料及系統鑰匙圈／密鑰庫；Mac為ad-hoc簽章、未公證。只公告與手動下載，不推送升级提醒或強制更新。

### Downloads

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

### Added

- 獨立修訂3下載記錄，原0.9.6、修訂2及所有歷史下載保留。

### Fixed

- 新增 Agent 主機時先確認既有 Agent 的相容性；相容者直接復用，不重新上傳、安裝或啟動。缺失才安裝；已確認舊版沿安全升級；未知或未來版本的探測不觸發覆蓋。
- 將身份採納、READY 確認及憑據保存顯示為各自的真實階段；就緒階段失敗不再誤報為初始 SSH 連線失敗，且不自動重放安装。
- 部署進度按當次呼叫、主機與操作隔離，拒絕終態後或離頁後的遲到事件。
- Windows／Mac工作台右上角關閉键與相鄰視窗控件對齊外框、背景及尺寸，保留危险懸停與原退出路徑；Linux繼續使用正常系統標題欄。
- Windows／Mac／Linux的經典及工作台搜尋入口預設收成放大鏡圖示；點擊開啟原搜尋面板，原快捷鍵保留。窄視窗的終端工具列自動換行，避免搜尋與連線操作重疊。

### Verified

- Windows／Mac／Linux 經典與工作台的搜尋按鈕、原面板、快捷鍵及窄視窗排版完成 170 項直接前端檢查、38 組布局；三端前端構建通過。
- 十項正式包已完成對應的來源與包體檢查；九項桌面新包確認包含最終搜尋介面，Windows 四包取得新的實際安裝身份與有效簽名。Mac 為 arm64、ad-hoc 簽章，未公證。
- Agent 實測涵蓋 Mac 三台 Linux 節點、Windows 11 ARM64 的原生 ARM64／相容 x64，以及 Linux 兩架構 VM；本次搜尋修正沿用未變更 Rust 流程的原始證據，不外推為新包全功能重測或 Windows x64 實體機驗收。
- Android arm64 正式 APK 沿用原簽名，已在 SDK Emulator 經正常介面完成既有 Agent 復用、就緒、憑據保存與監控載入；不外推為真機或一般主機刪除驗收。iOS 僅同步源碼與前端，不上傳。

### SHA256

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

## 0.9.6 — 修訂2：啟動、資料接續與中斷上下文

2026-10-11

本次僅公告、手動下載，不提醒升級、不強制更新。產品版本仍為0.9.6，修訂2使用新tag `v0.9.6-r2`；四平台十項，iOS不上传，所有舊Release與原0.9.6資產保持。Android／iOS構建號9006000，wire及簽名身份不變。

本修訂使用SQLite26。請退出程式、保全完整資料並保留現有系統鑰匙圈／密鑰庫條目，再於原位置替換程式；不要手動搬移、刪除或匯出原始密鑰，也不要清庫或以卸載處理啟動／簽名問題。正常22→26及已驗證的已知25資料接續已實測；其它23–25只接受嚴格已知形狀與信任無損的前滾。資料一旦採用26，不要用舊schema22的原0.9.6包打開新庫；回退必須使用相配的舊程式與升級前完整備份。

### Downloads

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

### Added

- 未明確選擇介面的資料與新安裝預設使用工作台，外觀跟隨系統；已保存的介面與外觀選擇繼續保留。
- 提供獨立修訂2下載記錄，保留原0.9.6與所有歷史下載。

### Fixed

- 修正Mac啟動初始化中止；已驗證的舊schema25資料可前滾到26，既有Cloud會話頁正常載入。
- 修正Mac右上角最大化／還原、最小化與關閉按鈕。
- 通知中心只顯示公告與版本訊息，移除帳號安全類的可見入口及相關未讀提示。
- AI同會話中斷後續問保留已記錄的原文、助手說明及有完整唯一結果的工具紀錄；缺失结果不補造，未完整涵蓋的紀錄不提前標為已覆蓋。
- SQLite前滾到26；已知23–25遺留資料只在嚴格形狀與信任無損檢查成立時接續，不恢復插件執行。

### Verified

- 十項制品共583,805,859位元組，大小與SHA256直接採用本輪制品及傳輸收據。未改的Mac r26／Windows r11 SSH、SFTP、監控切換、離頁返回與UI-R57刪除實鏈按原範圍復用。
- Mac arm64：同Store首候選的實際25→26遷移證據復用；最終r2在已採用26的資料上以普通身份啟動，既有Cloud頁、窗口按鈕與關閉通過。真模型純文字續問保留前文；停止發生在provider context之前且tool=0，未驗真模型的不完整工具輪。Mac為ad-hoc且未公證。
- Windows11 ARM VM：ARM64原生、x64仿真；release原生程序的22→26資料保持、工作台及同會話停止／續問HTTP fixture通過。新普通使用者的installer-only、原公開0.9.6同版本覆蓋、實際EXE身份、data及登記保持通過；四包epoch3真簽名／驗簽通過，沒有Authenticode。新身份Session0的close／exit_app自然收尾門禁失敗保留；ARM64 raw另於Session1的close／destroyed通過，不稱全套formal通過。
- Android arm64正式APK在真正Android SDK Emulator完成同簽名／同9006000覆蓋、22→26、51張Required表與偏好／包裹資料保持，以及fresh工作台／system外觀。AI只驗入口；未驗真手機或一般CleanShutdown。
- Linux兩架構Ubuntu VM：22→26保留57表及12條舊資料；deb後直接FUSE AppImage冷啟與實際業務畫面通過。無獨立模型憑據，未做Linux原生AI續問。
- 不完整工具輪、唯一完整結果與精確覆蓋水位：共享3项集成測試／4個實際本地HTTP場景通過；Windows原生使用受控HTTP fixture。這些不是全平台真模型／遠端工具結果。
- iPhone Simulator實測版本／入口、fresh工作台／system及通知兩類，不公開上传。真機、最低系統實際運行、其它發行版／Wayland、弱網及未測遠端流程均未外推。

### SHA256

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
