**中文** | [English](CHANGELOG_EN.md)

# 更新列表

完整安装包请前往 [GitHub Releases](../../releases)。每个 Release 都包含对应版本的安装包、更新说明和验证信息。

## v0.6.1 - SSH 兼容与 agent 降级修复

### 下载
- Windows 安装版: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows 便携版: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### 新增
- 增加 CentOS/RHEL/FIPS/老 OpenSSH 的登录密钥兼容策略:首选 Ed25519,必要时自动尝试 RSA/ECDSA P-256。
- 增加 agent `direct-streamlocal` 通道被服务器策略拒绝时的稳定识别标签,避免误判为 SSH 密码错误。
- 桌面端与移动端部署流程拆分凭据/资源 helper,文档和代码索引同步到当前结构。

### 修复
- 修复 CentOS 7.9 等环境下 agent 私钥显示错误、密钥登录无法稳定建立的问题。
- 修复服务器拒绝 agent unix socket 转发时反复要求用户输入 SSH 密码的问题;终端会临时降级到普通 SSH PTY。
- 修复已保存 SSH 密码时,懒部署/建钥仍可能再次打断用户要求输入密码的问题。

### 验证
- Rust `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 全部通过。
- 桌面 `npm run tauri build` 已生成 Windows setup/MSI,并重新制作 portable zip。
- Android arm64 release APK/AAB 已生成;APK 通过 `apksigner verify --verbose --print-certs` 与 `aapt dump badging` 检查,包名 `com.creationssh.mobile`,SDK 24/36,ABI 仅 `arm64-v8a`。
- 脱敏真机验证覆盖 CentOS 7.9 与 Ubuntu 24:CentOS 密钥建立成功并在 agent 通道被策略拒绝时降级 SSH,Ubuntu agent 握手与命令执行正常。

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
