**中文** | [English](CHANGELOG_EN.md)

# 更新列表

完整安装包请前往 [GitHub Releases](../../releases)。每个 Release 都包含对应版本的安装包、更新说明和验证信息。

## v0.6.7 - AI 助手弹窗修复与发布前真实验证

### 下载
- Windows 安装版: `Creation-SSH_0.6.7_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.7_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.7_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.7_android-arm64.apk`
- Android AAB: `C-SSH_0.6.7_android-arm64.aab`

### 新增
- 固化发布前真实功能验证门禁:公开 Release / 新 tag / 上传安装包前,必须运行即将发布的正式桌面程序与移动测试包,逐项真实操作本次改动涉及功能。
- 桌面 AI 弹窗回归检查补充 Windows WebView 创建方式约束,避免后续再次把动态窗口做成同步 command。

### 修复
- 修复桌面 AI 助手独立弹窗在 Windows 正式包中可能打开为白色空窗口的问题;后端窗口创建改为 Tauri 推荐的 async command。
- 修复 AI 助手弹窗按钮、历史入口和 agent 性能档位在部分语言下显示乱码或错误 key 的问题。
- 修复主窗口退出时 AI 子窗口可能残留的问题,主窗口关闭会同步清理 `ai-*` 弹窗。

### 验证
- 桌面 `npm run test:ai-window` 通过,覆盖 AI 弹窗 URL、`ai-*` 权限、async command、历史入口文案和多语言文案。
- Rust `cargo fmt --check`、桌面 Tauri workspace `cargo fmt --check`、`cargo check` 通过;桌面 `npm run build` 与 `npm run tauri build` 通过。
- 正式 Windows 程序重新运行后,AI 助手独立弹窗渲染正常(用户现场复测确认),不再出现白色空窗口。
- Android arm64 release APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging`,包名 `com.creationssh.mobile`,versionName `0.6.7`,versionCode `6007`,ABI 仅 `arm64-v8a`。
- Android x86_64 debug APK 仅作测试包,已安装到 MuMu 模拟器并启动;系统包信息显示 versionName `0.6.7`、ABI `x86_64`。该包不上传公开 Release。
- 发布资产重新计算 SHA256,公开仓说明与 Release notes 已脱敏复核。

### SHA256
- `Creation-SSH_0.6.7_portable-Windows-x64.zip`: `B80BC866177D5D9C82034E21BEB41C6B5100A6A0BD62039A5E7D31F8C8A0983F`
- `Creation-SSH_0.6.7_x64_en-US.msi`: `4EE2AF5FFA7CDEF55A5C44144D12B5B6CE30C0D5C28FC1BB06DD803AA4CC84E1`
- `Creation-SSH_0.6.7_x64-setup.exe`: `0447447C4DFFB35DFF48925134368B197AF8B7954033293567CE59A17C2B6D1E`
- `C-SSH_0.6.7_android-arm64.aab`: `0DE4EA9BF5D6B021E90CD8E9C889E73D6DF3E64CF775606B14247E0E4487153E`
- `C-SSH_0.6.7_android-arm64.apk`: `4EA626D63E709F6B1BF0D1A014C951C76153AA2BB141243A250F550DC5BFB402`

## v0.6.6 - Agent 桥接兜底、并发保护与密钥主机

### 下载
- Windows 安装版: `Creation-SSH_0.6.6_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.6_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.6_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.6_android-arm64.apk`
- Android AAB: `C-SSH_0.6.6_android-arm64.aab`

### 新增
- agent 新增 `--stdio-bridge` 桥接模式:当服务器 sshd 拒绝 `direct-streamlocal` 时,客户端会自动退到桥接通道继续访问本机 unix socket,CentOS 7.9 等老系统也能使用 agent 能力。
- agent 短请求新增低/中/高/超级性能档位,对应 Stable / Balanced / Fast / Ultra,并把每台主机的选择持久化到本地 SQLite。
- 桌面端 AI 助手新增独立弹窗入口,可把当前 AI 助手弹出为单独窗口,为多 AI 助手窗口并行操作预留骨架。
- 桌面端与移动端主机添加流程补齐 OpenSSH 私钥模式,支持粘贴私钥后保存到本地加密仓库,后续连接自动复用仓库凭据。
- 主机管理补齐分组编辑、删除确认、认证失败提示与多语言文案,公开版本号同步到 `0.6.6`。

### 修复
- 修复 CentOS 7.9/OpenSSH 7.4 系列环境中 `direct-streamlocal` 被 sshd 返回 `AdministrativelyProhibited` 后 agent 能力不可用的问题;现在会自动启用桥接兜底,无需用户重复输入 SSH 密码。
- 修复高并发工具调用下短 agent 请求没有统一限流的问题;所有非长流短请求统一经过 agent request guard,并在 SSH/channel/streamlocal 传输类错误后自动临时降级到 1 并发。
- 修复移动端和桌面端部分 deploy / files / monitor / appcenter / sysmgmt 路径绕过统一 agent 并发保护的问题。
- 修复“桥接不可用”提示容易误导的问题:只有直连与桥接都失败时才提示 agent 传输兜底失败。
- 修复主机凭据保存后部分客户端路径仍可能要求用户重新输入密码的问题,添加成功后优先复用本地仓库凭据。

### 验证
- Rust `cargo fmt --check`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo check -p client`、`cargo check -p agent` 通过。
- 桌面/移动 Tauri workspace `cargo check` 通过;桌面与移动 `npm run build` 通过。
- agent x86_64 与 aarch64 Linux musl release 通过 `cargo zigbuild` 构建,版本同步为 `0.6.6`。
- 桌面 `npm run tauri build` 生成 Windows setup/MSI;主程序文件版本与产品版本均为 `0.6.6`,便携版 zip 已重打。
- Android x86_64 debug APK 已安装到 MuMu 模拟器验证;`aapt dump badging` 确认 `versionName=0.6.6`、`native-code='x86_64'`。该包仅用于测试,不上传公开 Release。
- Android arm64 release APK/AAB 通过构建;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging`,包名 `com.creationssh.mobile`,versionName `0.6.6`,versionCode `6006`,ABI 仅 `arm64-v8a`。
- CentOS 7.9 真机验证:原生 `direct-streamlocal` 仍被 sshd 拒绝,但当前客户端自动走 `stdio-bridge`,并完成 `handshake`、`sysinfo`、`metrics` 与 `mon` 流式订阅。
- 发布前对私有源码/文档、公开仓文案与 release notes 执行脱敏扫描,未发现真实凭据、token、私钥或非示例 IP。

### SHA256
- `Creation-SSH_0.6.6_portable-Windows-x64.zip`: `68773A304D2C74C3BEE922F75FC3D0C5C61F3D077C57083132C4371E8F98EFBE`
- `Creation-SSH_0.6.6_x64_en-US.msi`: `D7A22FB19E46BD4CFD0A17DBB2DF847051E8246961460111180E1FE2228C2989`
- `Creation-SSH_0.6.6_x64-setup.exe`: `1D2EC3D2F9F22A52AAC94227AF68D2EACBAB4C93F7CD350B56F7C0A3E9C2D76C`
- `C-SSH_0.6.6_android-arm64.aab`: `67E0EB455CA9A92F0379F785D31486A002884D3FAED9EED75E6C055AC2BFBEB4`
- `C-SSH_0.6.6_android-arm64.apk`: `78990A2DB4F286E0E46C0FF0AD959A57A0CDE79A652DFF795028AE7AF504C8F7`

## v0.6.5 - AI 工作区、历史入口与敏感附件拦截

### 下载
- Windows 安装版: `Creation-SSH_0.6.5_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.5_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.5_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.5_android-arm64.apk`
- Android AAB: `C-SSH_0.6.5_android-arm64.aab`

### 新增
- 桌面端 AI 助手新增顶部工作区栏,将历史记录入口从固定侧栏移到顶部弹层,释放主对话区空间。
- 移动端 AI 助手同步新增顶部历史入口,更适合窄屏使用,也为后续多 AI 窗口能力预留统一工作区骨架。
- 新增桌面/移动 AI 工作区静态回归脚本,覆盖顶部历史入口、移动端键盘锚点与敏感附件拦截规则。
- 公开发布资产继续强制全量带版本号,本次 Windows、Android、agent 与公开说明全部同步到 `0.6.5`。

### 修复
- 修复 AI 历史侧栏长期占用桌面主界面宽度的问题,现在默认只保留顶部入口,需要时再展开历史列表。
- 修复移动端 AI 输入区在软键盘打开时容易被遮挡、看不清正在输入内容的问题,输入区改为底部键盘锚定布局。
- 修复历史弹层/底部历史页在未选择服务器时仍可能触发无效会话操作的问题,增加禁用态与键盘关闭处理。
- 修复 AI 附件上传过于依赖 MIME/扩展名白名单的问题,现在会先按文件名拒绝 `.env`、私钥、证书密钥等敏感文件。

### 验证
- 桌面 `npm run test:ai-workspace`、`npm run test:locale-system`、`npm run build` 通过。
- 移动端 `npm run test:ai-workspace`、`npm run test:ai-keyboard`、`npm run test:locale-system`、`npm run build` 通过。
- agent x86_64 musl release 通过 `cargo zigbuild -p agent --target x86_64-unknown-linux-musl --release` 构建,日志显示 `agent v0.6.5`。
- 桌面 `npx tauri build` 在 agent 0.6.5 之后重新执行,生成 Windows setup/MSI;Windows 可执行文件与 setup 属性显示 `0.6.5`。
- Android arm64 release APK/AAB 在 agent 0.6.5 之后重新执行 clean build;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging`,包名 `com.creationssh.mobile`,versionName `0.6.5`,versionCode `6005`,ABI 仅 `arm64-v8a`;AAB 只包含 `arm64-v8a` native lib。
- 发布前对私有源码/文档与公开仓文档执行脱敏扫描,未发现私钥、token、真实凭据或非示例 IP。

### SHA256
- `Creation-SSH_0.6.5_portable-Windows-x64.zip`: `96150F4C5128A33FFE558289FB2261BADF31A73B4508E1243783952C2401B36A`
- `Creation-SSH_0.6.5_x64_en-US.msi`: `47E0817CC3F69E8ADA7B9F364FCDE14B876E99E3425856D2D5C00FEBB808C153`
- `Creation-SSH_0.6.5_x64-setup.exe`: `FD532382E110769DCC3DA40362A6102008822005F4FD67E5ED215D21A88C889A`
- `C-SSH_0.6.5_android-arm64.aab`: `C217DA6561ED0DE2A79E684B6C95B6111F74CB96AE784B40379658C543217EBB`
- `C-SSH_0.6.5_android-arm64.apk`: `4F279E03184F0942C276F4B915129844856FAEDFFB5A72B9522E608540238F79`

## v0.6.4 - AI 运行恢复、系统语言与移动端输入修复

### 下载
- Windows 安装版: `Creation-SSH_0.6.4_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.4_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.4_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.4_android-arm64.apk`
- Android AAB: `C-SSH_0.6.4_android-arm64.aab`

### 新增
- 新增 AI 运行状态持久化与页面恢复:切换页面或重新进入 AI 助手后,正在运行/刚结束的回合可以恢复事件与状态。
- 新增 ACP/Hermes 基础协议骨架,先提供能力描述与适配入口,后续继续扩展完整 Hermes 体验。
- 新增桌面与移动端 GitHub 更新检测入口,提示用户到 GitHub Releases 下载,不做静默自动安装。
- 新增系统语言跟随模式:默认跟随系统语言,也可手动选择 9 种界面语言;偏好继续落 SQLite。
- 移动端 AI 可见图标改为组件化 SVG 图标,减少 emoji 在不同系统上的不一致显示。

### 修复
- 修复高并发工具调用场景下 AI 执行链路容易崩溃的问题:同一模型回合内的多个工具调用改为受控串行执行,并补充运行态 guard。
- 修复移动端 AI 输入框被软键盘遮挡的问题:Android Activity 使用 `adjustResize`,前端按 `visualViewport` 调整底部空间。
- 修复移动端 AI 权限模式、执行档、工具次数等偏好在退出/切换页面后回到默认值的问题。
- 补齐 PC 端工具调用次数自定义入口,与移动端一样走持久化配置。
- 本版继续包含终端尺寸/状态栏、默认透明度 80%、agent 版本同步、正式资产全带版本号等此前修复。

### 验证
- Rust `cargo fmt --check`、`cargo build --workspace`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings` 全部通过。
- 桌面/移动前端 `npm run build` 通过;系统语言回归、移动端 AI 键盘避让回归、启动动画回归脚本通过。
- 桌面 `npm run tauri build` 生成 Windows setup/MSI;Windows 可执行文件与 setup 属性显示 `0.6.4`;便携版 zip 已重新制作。
- Android arm64 release APK/AAB 已生成;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging`,包名 `com.creationssh.mobile`,versionName `0.6.4`,versionCode `6004`,ABI 仅 `arm64-v8a`;AAB 抽查只包含 `arm64-v8a` native lib。
- 发布前对源码/文档与公开文案执行脱敏扫描,未发现私钥、token、真实凭据或非示例 IP。

### SHA256
- `Creation-SSH_0.6.4_portable-Windows-x64.zip`: `CFC6D71575E26C1F2E84539505AF1AC5DB72B0B63F182EF3DA2C6191AE3AE799`
- `Creation-SSH_0.6.4_x64_en-US.msi`: `1B3D8364CED5BA4A6E53FD28F4024A53322C286D904C4AC95527A0827AF7BAF6`
- `Creation-SSH_0.6.4_x64-setup.exe`: `48A0318A3AFDC7679301C00F46DCA0C6BCC329BD3FF2A8184A90B1B6298A6131`
- `C-SSH_0.6.4_android-arm64.aab`: `891AC7731D7B592122294B28E508030DBEDB1792253F6DF8C09F296AAEF25001`
- `C-SSH_0.6.4_android-arm64.apk`: `5F8099C77DFD547DCF969937D02AA07C184F3C05B1A5A107EE6BFA43F26CA664`
## v0.6.3 - 版本同步、AI 布局与发布包修复

### 下载
- Windows 安装版: `Creation-SSH_0.6.3_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.3_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.3_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.3_android-arm64.apk`
- Android AAB: `C-SSH_0.6.3_android-arm64.aab`

### 新增
- agent 正式版本同步到 `0.6.3`,客户端懒部署现在同时检查协议号与 agent 包版本;协议不变但 agent 版本落后时也会自动升级。
- 固化版本同步规则:每次正式发布必须同步源码版本、Tauri 配置、npm 包版本、Android versionName/versionCode、关于页、agent 版本、公开 Release 文件名与文档。
- 桌面端 AI 助手在窗口最大化后改为全宽铺满布局,和其他菜单页保持一致。

### 修复
- 修复 Windows 安装包、APK 应用信息、关于页面仍显示 `0.1` / `0.1.0` 的问题;本版统一显示 `0.6.3`。
- 修复 agent 长期显示 `0.0.1` 的问题;agent 运行时版本现在来自包版本,并随发布更新。
- 修复桌面透明度默认值没有按要求生效的问题;默认值改为 80%,旧的 0 值会迁移到 80%。
- 修复 AI 助手执行档选择不会像权限模式一样持久化的问题;重新打开后保留上次选择。
- 继续补齐 AI 权限模式、自定义上下文窗口、工具循环自定义次数等公开说明,避免用户找不到入口。

### 验证
- `cargo fmt --check`、`cargo check -p agent` 通过,agent 构建显示 `0.6.3`。
- 桌面/移动 `npm run build` 通过;桌面/移动 Tauri workspace `cargo check` 通过。
- 桌面 `npm run tauri build` 已生成 `0.6.3` Windows setup/MSI;Windows 可执行文件版本为 `0.6.3`。
- Android x86_64 debug 包已安装到 MuMu 12 模拟器,系统包信息 `versionName=0.6.3`,关于页面显示 `0.6.3`。
- Android arm64 release APK/AAB 已生成;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging`,包名 `com.creationssh.mobile`,SDK 24/36,ABI 仅 `arm64-v8a`。

## v0.6.2 - AI 设置持久化与自定义模型体验修复

### 下载
- Windows 安装版: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows 便携版: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### 新增
- 桌面 AI 助手页顶栏新增「工具循环」数字框,PC 端不用再进设置页深处寻找自定义次数入口。
- 自定义 AI provider 增加上下文窗口设置,支持 4096 到 2,000,000 tokens,默认 128k;自定义模型的上下文剩余条会使用用户配置。
- AI 配置持久化命令拆分为独立 `ai_config` 模块,后续扩展配置字段不会继续撑大执行循环文件。

### 修复
- 修复 AI 助手权限模式在离开页面再切回时又变回第一个「只读」的问题;现在会先用本会话缓存恢复选择,同时继续写入 SQLite。
- 修复权限模式切换后立刻去做其他事情时保存不够及时的问题;前端改为同步 flush 触发保存。
- 修复自定义 AI 保存配置时无法传递上下文窗口的问题,桌面端与移动端保持一致。
- 修复桌面 clippy 因部署注释列表缩进触发 `doc_lazy_continuation` 的阻断。

### 验证
- 桌面/移动 `npm run build` 通过。
- 桌面/移动 Tauri workspace `cargo check`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 通过。
- 9 种语言的 AI 文案 JSON 全部通过解析。
- 桌面 `npm run tauri build` 已生成 Windows setup/MSI,并重新制作 portable zip。
- Android arm64 release APK/AAB 已生成;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging` 检查,包名 `com.creationssh.mobile`,SDK 24/36,ABI 仅 `arm64-v8a`。

## v0.6.1 - 终端体验与 agent 兼容修复

### 下载
- Windows 安装版: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows 便携版: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### 新增
- AI 工具循环默认上限从 16 次调整为 30 次,并支持在设置里自定义次数,同时保留安全范围约束。
- AI 助手五档权限模式改为持久化保存,重新打开应用后会保留上次选择的权限模式。
- 增加 CentOS/RHEL/FIPS/老 OpenSSH 的登录密钥兼容策略:首选 Ed25519,必要时自动尝试 RSA/ECDSA P-256。
- 增加 agent `direct-streamlocal` 通道被服务器策略拒绝时的稳定识别标签,避免误判为 SSH 密码错误。
- 增加统一认证错误分类工具,文件、监控、系统管理、应用中心等页面可复用同一套错误判断与提示。
- 桌面端与移动端部署流程拆分凭据/资源 helper,文档和代码索引同步到当前结构。

### 修复
- 修复桌面终端 CLI/xterm 区域尺寸偏小的问题:终端页面改为全宽布局,顶部控制区与工具栏在窄屏下更紧凑稳定。
- 修复终端光标和最后一行在底部被裁切的问题:补齐 xterm 容器底部留白和行高约束。
- 修复持久化终端状态栏/模式状态在 agent 通道不可用时显示不准、卡在持久化状态的问题;现在仅本次临时降级到普通 SSH PTY,不改写用户保存的终端模式。
- 修复 CentOS 7.9 等环境下 agent 私钥显示错误、密钥登录无法稳定建立的问题。
- 修复服务器拒绝 agent unix socket 转发时反复要求用户输入 SSH 密码的问题;终端会临时降级到普通 SSH PTY。
- 修复已保存 SSH 密码时,懒部署/建钥仍可能再次打断用户要求输入密码的问题。
- 修复文件、监控、系统管理、应用中心等 agent 页面把 `direct-streamlocal` 被拒误判为 SSH 密码错误的问题,改为清楚提示 agent 通道被服务器策略拦截。

### 验证
- Rust `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 全部通过。
- 桌面 `npm run tauri build` 已生成 Windows setup/MSI,并重新制作 portable zip。
- Android arm64 release APK/AAB 已生成;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging` 检查,包名 `com.creationssh.mobile`,SDK 24/36,ABI 仅 `arm64-v8a`。
- 脱敏真机验证覆盖 CentOS 7.9 与 Ubuntu 24:CentOS 密钥建立成功并在 agent 通道被策略拒绝时降级 SSH,Ubuntu agent 握手与命令执行正常。
- GitHub Issues #1、#2、#3、#4 已逐条回复修复说明,并指向 v0.6.1。

## v0.6 - 启动动画与图标安全区修复

### 下载
- Windows 安装版: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows 便携版: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### 新增
- 固化公开发布流程:公开仓 README/CHANGELOG/Release notes 保持中英双语,每次正式包发布新建 tag,不覆盖旧版本。
- 新增移动端启动动画时间轴回归测试 `npm run test:splash`,同时约束大 C 桥接文字收拢和 App 壳退场前预渲染。

### 修复
- 修复 Android 启动动画小字汇聚成大 C 时的明显空档:大 C 舞台现在在文字收拢阶段提前挂载并交叠淡入。
- 修复 Splash 退场后主界面冷挂载造成的短暂黑帧:主路由和底部 Tab 现在会在遮罩仍覆盖时预渲染。
- 修复品牌图标在 Android 上 C 过大、切边、边框消失的问题:C 缩小到安全区内,Android launcher/foreground/round 均使用完整主源等比缩放。

### 验证
- agent x86_64 musl release、移动端测试/构建、桌面 Tauri release 构建均通过。
- Android x86_64 debug APK 已 clean 后重建并安装到 MuMu 12 模拟器;3500/3900/4300ms 启动关键帧均有大 C/边框覆盖,5000ms 进入主界面。
- Android arm64 release APK 已通过签名、包名、SDK 与 ABI 检查;ABI 仅 `arm64-v8a`。

## v0.5 - 品牌图标与正式包更新

### 下载
- Windows 安装版: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows 便携版: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### 新增
- 全新品牌 C 图标:深色玻璃底、银色金属 C,桌面端、Android launcher、应用内品牌图统一更新。
- 公开发布说明改为中英双语,Release 列表作为版本更新记录入口。
- 补齐接手开发文档、架构总览、协议参考、部署发布、测试指南、代码地图与代码索引。

### 修复
- 修复 Android release 图标资源同步问题,避免 launcher/foreground/round 图标仍使用旧图。
- 修复 Android 构建资源缺失的 `splash_window_bg` 配置,release/debug 构建均可正确打包。
- 修复 AI 会话入口参数过多导致的 clippy 阻断,改为结构化 `RunRequest`。
- 私有源代码仓与公开推广仓边界写入规则,避免 APK、推广 zip、凭据、截图、日志误入源代码仓。

### 验证
- Rust fmt、clippy、workspace tests 已通过。
- Windows 桌面 release 构建已通过。
- Android arm64 release APK 已通过签名、包名、SDK 与 ABI 检查。
- Android x86_64 debug 包已在模拟器验证冷启动和主界面渲染。
- 发布前执行脱敏检查,不包含真实 IP、凭据、测试机信息。

## v0.4 - 桌面 + 安卓双端

### 新增
- 首次提供 Windows 桌面安装包。
- Android 开屏动画优化,减少白闪和主界面闪现。
- Android 应用图标修正为深色底完整 C。
- AI 助手支持 OpenAI 兼容接口与 Anthropic。

### 下载
- `C-SSH-android-arm64.apk`
- `Creation-SSH-portable-Windows-x64.zip`
- `Creation-SSH_0.1.0_x64-setup.exe`
- `Creation-SSH_0.1.0_x64_en-US.msi`
