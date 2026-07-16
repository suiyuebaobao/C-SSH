**中文** | [English](CHANGELOG_EN.md)

# 更新列表

完整安装包请前往 [GitHub Releases](../../releases)。每个 Release 都包含对应版本的安装包、更新说明和验证信息。

## v0.6.14 - 三端主机硬删除与生命周期隔离

### 下载
- Windows 安装版：`Creation-SSH_0.6.14_x64-setup.exe`
- Windows MSI：`Creation-SSH_0.6.14_x64_en-US.msi`
- Windows 便携版：`Creation-SSH_0.6.14_portable-Windows-x64.zip`
- Android arm64 APK：`C-SSH_0.6.14_android-arm64.apk`
- Android arm64 AAB：`C-SSH_0.6.14_android-arm64.aab`
- Linux AppImage：`Creation-SSH_0.6.14_linux-x86_64.AppImage`
- Linux deb：`Creation-SSH_0.6.14_linux-amd64.deb`

### 新增
- Windows、Linux 与 Android 统一使用主机硬删除语义：删除会结束该主机在当前设备上的本地生命周期，而不是只移除列表项。
- 本地数据库升级到 schema 5，主机与关联状态通过 `ON DELETE CASCADE` 收口，并在迁移中执行一次匿名孤儿记录清理。

### 修复
- 删除主机时同步清除本地主机记录、绑定凭据、历史会话、终端窗口持久化、监控缓存及其他可归属状态，避免残留数据继续影响客户端。
- 后续新增主机始终创建全新生命周期；即使复用已删除主机的 ID 或网络地址，也不会继承旧凭据、会话、窗口或指标数据。
- 不可达主机仅在远端清理尚未开始时允许执行本地硬删除；一旦流程涉及远端服务、会话、socket、数据或公钥，归属或完整性无法确认时即 fail-closed。
- schema 5 迁移一次性清理无法归属到任何主机的匿名孤儿记录，避免历史残留继续游离于生命周期之外。
- 修复 Windows/Linux 主窗口关闭后进程仍在后台残留的问题；`CloseRequested` 与 `Destroyed` 统一进入幂等退出流程。

### 验证
- 公开资料已完成版本号、资产命名、中英双语功能口径、逐图说明、QQ群入口保留与脱敏检查。
- Windows/Linux 正式候选均要求主窗口与调试连接消失后，程序进程自然归零；强制结束进程不计为通过。
- 尚未执行的破坏性真实服务器深度删除 E2E 未列为通过项；本条目不声称该项已经验证。

### SHA256
- `Creation-SSH_0.6.14_x64-setup.exe`: `3332E724EAD76EE32ABFB047DBDBB77C8C372D8D8CAC5A5CF37A315A69F2C1A3`
- `Creation-SSH_0.6.14_x64_en-US.msi`: `B74A9FCA806D83AD09B8FA117D14730A53BFFB67CA842C98A618014D80D42A02`
- `Creation-SSH_0.6.14_portable-Windows-x64.zip`: `04E326D51D380188C9EFA2232448F67CFBC7ED3769E5D1F51C8F78830E7BC7C0`
- `C-SSH_0.6.14_android-arm64.apk`: `349BF316DB92FCB84E4143B9F9C6FE967B82A55BC2828652B09DA39EC601C458`
- `C-SSH_0.6.14_android-arm64.aab`: `E656672C08F5E1D3E9D0C17E5B18CEFB0AD4ABF3A5D8697E231D00B22A8828B2`
- `Creation-SSH_0.6.14_linux-x86_64.AppImage`: `CB09DF5ED82FC9E084145177ED18702959BA33300C7CED71EC407154B5FC863A`
- `Creation-SSH_0.6.14_linux-amd64.deb`: `48DDA463DAF7ED99F77018C1E3AD1B1515B5D731044B381137F1E38413793F15`

## v0.6.13 - 主机监控恢复与客户端韧性

### 下载
- Windows 安装版：`Creation-SSH_0.6.13_x64-setup.exe`
- Windows MSI：`Creation-SSH_0.6.13_x64_en-US.msi`
- Windows 便携版：`Creation-SSH_0.6.13_portable-Windows-x64.zip`
- Android arm64 APK：`C-SSH_0.6.13_android-arm64.apk`
- Android arm64 AAB：`C-SSH_0.6.13_android-arm64.aab`
- Linux AppImage：`Creation-SSH_0.6.13_linux-x86_64.AppImage`
- Linux deb：`Creation-SSH_0.6.13_linux-amd64.deb`

### 新增
- Windows/Linux 主机列表改为紧凑自适应指标网格，展示真实系统、CPU、内存、磁盘、负载、运行时长与正常/暂停/失败三种可见状态。
- 最新指标与系统描述写入共享 SQLite schema 4；Android 同步指标缓存字段，三端删除继续与后台采集互斥。

### 修复
- 修复 Windows、Linux 与 Android 切换菜单会销毁终端现场的问题；普通 PTY 和持久化 tmux 会话现在跨菜单保留连接、画面与输入状态，重新附加不再产生重复提示符。
- 修复监控数据已经成功拉取但状态仍停在“失败”的问题；附加系统信息失败不再丢弃有效动态指标，修复 agent 后立即执行一次真实采集并按结果刷新。
- 修复不可达主机无法删除本地记录的问题；SSH bootstrap 失败时明确跳过远端破坏动作并警告可能残留，已连接后的归属审计仍严格停止。
- Windows/Linux AI 助手对明确离线的当前主机禁用输入、附件与发送；切换到可用主机后恢复。
- 修复 Windows 开屏受应用壳布局影响而偏移的问题，动画继续保持原时序并按完整窗口居中。

### 验证
- Windows 隔离候选、授权 Ubuntu 虚拟机和 Android x86_64 MuMu 测试包均完成普通/持久化终端菜单往返、继续输入、主动断开保持及无重复提示符的真实验证。
- Windows 候选完成真实 `failed→fresh`、系统列、无横向溢出、启动与退出验证。
- Android arm64 正式资产完成版本/包名/ABI/签名检查，x86_64 测试包在 MuMu 完成受影响页面实操。
- Linux AppImage/deb 在授权 Ubuntu 虚拟机本地构建，并在隔离图形与密钥环会话中完成安装、启动、关闭及本轮主机监控验证。

### SHA256
- `Creation-SSH_0.6.13_x64-setup.exe`: `161795BA3CB74E144A3423E50CB3C03D857A4AB448C7AEF7C2046BA0E78D03C1`
- `Creation-SSH_0.6.13_x64_en-US.msi`: `DD62A6C7EC1BAF2CC1ABF8C3F18BFC37424239709247990F78D99E0B94599EF6`
- `Creation-SSH_0.6.13_portable-Windows-x64.zip`: `FEA9D83918138EDD04A1FF839BB92A14A7CD2BFA321AA9187EAF7FB331387057`
- `C-SSH_0.6.13_android-arm64.apk`: `FE2BA564552B072D0997FC0C9DB00391098C802E4E867CB3E99EEA75C2D5FE5D`
- `C-SSH_0.6.13_android-arm64.aab`: `802CA928A94A2F09B8267B7E43BE9F945CA7463627A5B4EF2D8748F87C84D2A4`
- `Creation-SSH_0.6.13_linux-x86_64.AppImage`: `EB56DB36524B4D6781DC5721824A29B70D89108DAAAE47A61A954757BD52D397`
- `Creation-SSH_0.6.13_linux-amd64.deb`: `FC01CA9EFC001345253C140ED0164D25BE2962F7915BC1275D915D6E616F08CA`

## v0.6.12 - 安全卸载收口与三端体验完善

### 下载
- Windows 安装版：`Creation-SSH_0.6.12_x64-setup.exe`
- Windows MSI：`Creation-SSH_0.6.12_x64_en-US.msi`
- Windows 便携版：`Creation-SSH_0.6.12_portable-Windows-x64.zip`
- Android arm64 APK：`C-SSH_0.6.12_android-arm64.apk`
- Android arm64 AAB：`C-SSH_0.6.12_android-arm64.aab`
- Linux AppImage：`Creation-SSH_0.6.12_linux-x86_64.AppImage`
- Linux deb：`Creation-SSH_0.6.12_linux-amd64.deb`

### 新增
- Windows、Linux、Android 复用同一主机已认证 SSH transport；监控、文件、AI、系统管理与终端通过独立 channel 并行，多客户端可同时连接。
- Android 终端采用紧凑双行工具栏，补齐主机 IP、唯一窗口名、`1-24px` 字体、自适应/固定/自定义尺寸、横纵滚动、复制与按需快捷键面板。
- Windows/Linux 使用系统“另存为”，Android 使用 SAF 文档选择器；三端均由用户选择下载目的地。
- Android 新增系统/浅色/深色主题和系统语言自动选择，终端、AI、监控偏好通过 SQLite 持久化。

### 修复
- 修复 `v0.6.11` 真实 Ubuntu 卸载后 tmux socket 路径残留阻断数据清理的问题；只隔离并删除身份连续、已证明归属的对象，任何身份异常继续 fail-closed。
- 修复离线主机旧绿色状态、全新 SQLite 并发初始化锁库、单 channel 失败误断健康 transport、停止状态 firewalld 误报，以及 Windows 主窗关闭后隐藏登录窗残留。
- SSH 的 DNS、TCP、握手、认证阶段分别使用 8 秒硬截止，不再为不可恢复错误重复完整拨号。

### 验证
- 根 workspace 全量测试、`client-core` 132 项定向测试、Clippy、格式与 Shell 语法通过。
- Ubuntu no-mock 覆盖正常 quarantine/guard 清理、初始 stale、事务残留、原路径替换与外部资源不变。
- Windows、Android、Linux 正式资产完成版本、包身份、签名/ABI/payload、真实启动、关闭与 SHA256 复核；Android x86_64 测试包在 MuMu 完成页面实操。
- Linux AppImage/deb 在隔离图形与系统密钥环会话中，以真实服务器和真实 AI Key 完成监控、系统、文件、进程、AI、失效重连和持久化终端重开。

### SHA256
- `Creation-SSH_0.6.12_x64-setup.exe`: `07F1E843DA9AF1122CB6E282343684DE898C18A25F14626EA50FE96C892B92F8`
- `Creation-SSH_0.6.12_x64_en-US.msi`: `3D4A70A0975A2D4A0755934B79220CFB3EA58D500790C37BFC76F21565D61257`
- `Creation-SSH_0.6.12_portable-Windows-x64.zip`: `0FF0672A737689959FE8B5D490F1C232432887AEDDDB40FEA08F724EF6E232F2`
- `C-SSH_0.6.12_android-arm64.apk`: `19B813DBF79A64304961C09DEBBC2268A64FEE48ACBD0ECDB2DB8D80DEB5D789`
- `C-SSH_0.6.12_android-arm64.aab`: `541C6A2BAAA7BC4A9489C55F736D4499CE57B7AF37D9645D7544B434193E0625`
- `Creation-SSH_0.6.12_linux-x86_64.AppImage`: `E4A2AC525ADF04304B642FE6E5C8A0A40AD99AFD09E66043117FEE86D41B7B45`
- `Creation-SSH_0.6.12_linux-amd64.deb`: `DBB46F0D7E1D31D2E97339C89D709B0818D85683BC0B9DC7275C6D983553C325`

## v0.6.11 - 跨端连接复用、移动终端与安全恢复（预发布）

> Ubuntu 真实卸载验证发现 tmux 受控退出后可能残留 socket 路径，完整产品数据清理会按安全策略停止。该版本已降为预发布；修复使用新版本发布，不覆盖本版本资产。

### 下载
- Windows 安装版：`Creation-SSH_0.6.11_x64-setup.exe`
- Windows MSI：`Creation-SSH_0.6.11_x64_en-US.msi`
- Windows 便携版：`Creation-SSH_0.6.11_portable-Windows-x64.zip`
- Android arm64 APK：`C-SSH_0.6.11_android-arm64.apk`
- Android arm64 AAB：`C-SSH_0.6.11_android-arm64.aab`
- Linux AppImage：`Creation-SSH_0.6.11_linux-x86_64.AppImage`
- Linux deb：`Creation-SSH_0.6.11_linux-amd64.deb`

### 新增
- Windows、Linux 和 Android 统一复用同一主机已经认证的 SSH transport。监控、文件、AI、系统管理和终端可在各自通道中并行工作，减少重复登录与等待；单项操作结束不会打断其他正在运行的功能。
- Windows/Linux 桌面端与 Android 可同时连接同一台主机并独立工作；一端退出后，另一端的监控和请求继续运行，持久化终端仍可重连恢复。
- Android 终端采用紧凑双行工具栏，在同一顶部区域显示主机、目标 IP、连接状态、持久化/普通终端切换、窗口选择和常用操作，为终端画布保留更多空间。
- Android 同时支持可断线恢复的持久化终端和关闭即结束的普通终端；持久化窗口默认使用不重复的 `terminal-N` 名称，支持安全重命名并整理旧重复名称。
- Android 终端新增自适应、固定 `80x24` 与自定义尺寸，字体可调范围为 `1-24px`；固定/自定义模式支持横纵浏览，尺寸、字体与滚动偏好会在重启后恢复。
- Android 终端新增按需快捷键覆盖层，提供 Esc、Tab、Ctrl、方向键和 `-`，不再常驻挤占终端空间；复制优先使用选区，无选区时复制当前可见内容，并写入系统剪贴板。
- Android 新增系统、浅色、深色三种主题模式；系统模式随设备主题自动切换，偏好写入本地 SQLite 并在重启后恢复。
- Windows 与 Linux 下载文件或目录时使用系统“另存为”，Android 使用 Storage Access Framework（SAF）系统文档选择器选择保存位置；取消不会先连接或下载，已选目标继续保留断点续传与完整性校验。
- SSH 连接新增分阶段 8 秒失败边界与明确提示：DNS 解析、TCP 连接、SSH 握手和认证分别计时，失败或超时后立即返回，不再因切换凭据重复等待。
- 未显式输入密码且库存私钥被服务器明确拒绝时，可在同一 SSH 会话内尝试本地加密保险库中的库存密码；认证成功后继续修复公钥登录，减少重复输入。

### 修复
- 修复全新本地数据库首次启动时，多个页面或后台任务同时初始化可能出现“database is locked”的问题；AI、文件、监控、主机与偏好数据现在可稳定从同一个 SQLite 数据库打开和恢复。
- 修复已安装但未运行的 firewalld 被误报为查询失败的问题；客户端现在明确显示“未运行”，保持端口操作禁用，也不会擅自启动或安装防火墙。
- 修复主机已离线但列表仍沿用旧绿色状态的问题；在线状态来自当前真实连接结果，历史指标只显示为过期数据。
- 单个功能通道失败不再连带断开仍然健康的共享 SSH transport；只有确认连接已中断时才重新连接，可能已经送达的修改操作不会自动重复执行。
- 主机密钥信任记录读取、解析或保存失败时改为安全停止：不把异常当作“首次连接”继续，不交付当前会话，也不转入其他凭据尝试。
- 删除主机或重装前会先确认相关服务、进程、持久化会话、数据和公钥确属 C-SSH；任何资源无法确认时立即停止并保留现状，不会误删其他服务、会话或密钥。
- systemd 危险指令按合法空白语法审计；无法逐字节证明归属的残留 tmux socket 只提示修复并保留，不再自动删除未知资源。
- 已确认属于 C-SSH 的旧版残留现在可以安全恢复到可重装状态；远端清理未完整成功时保留本地主机和凭据供重试，外部或未知资源始终保持不变。

### 验证
- Windows、Linux、Android 的共享连接复用、跨客户端同连、单端退出后另一端继续工作，以及 AI、文件、监控和两种终端的真实流程均已通过验证。
- Android 紧凑工具栏、主机 IP、持久化/普通终端、唯一窗口名、`1-24px` 字体、尺寸、滚动、复制、快捷键覆盖层及重启恢复均已通过验证。
- Windows/Linux 系统“另存为”、Android SAF 系统文档选择器、取消路径、断点续传和下载完整性校验均已通过验证。
- 全新 SQLite 首次并发打开、SSH 各阶段 8 秒失败提示与同会话凭据恢复均已通过验证。
- 主机密钥异常与安全删除的停止保护、外部资源保持不变，以及已确认旧版残留的重装恢复均已通过验证。
- 根 workspace 门禁与 Windows、Android、Linux 三端构建测试均已通过；CentOS 上已安装但未运行的 firewalld 正确返回 `NotRunning`。
- Windows 正式程序已完成独立启动、关闭、SQLite 与 `0.6.11` 版本检查。
- Android x86_64 测试包已在 MuMu 中真实进入终端、文件、监控和 AI 流程且无崩溃；arm64 APK/AAB 的包名、版本、ABI 与签名检查均已通过。
- Linux deb/AppImage 已在授权虚拟机中完成真实安装、启动、关闭与 SQLite 检查；两种包的 payload 一致，内置 agent 版本为 `0.6.11`。

### SHA256
- `Creation-SSH_0.6.11_x64-setup.exe`: `bf03f3805c28cdaf6d545e6b5bfac3d2ed0ec44265f591569c78be35fceb8c5b`
- `Creation-SSH_0.6.11_x64_en-US.msi`: `647b4b8978433385950b34578588366657206f2746eb38355f2102f01295a911`
- `Creation-SSH_0.6.11_portable-Windows-x64.zip`: `f319942c1710e794a78792b84dcc1e0a1178efb4b2b0d1dab1f205a832aa8b61`
- `C-SSH_0.6.11_android-arm64.apk`: `92246daa0cbcd0283e238bc02d729f497a94407c6c4efc384de7fd3787a061ab`
- `C-SSH_0.6.11_android-arm64.aab`: `3e3394bde08a9c8c96fcea6cc1660475ff509dec1d2ca588fa6032b0eaeee063`
- `Creation-SSH_0.6.11_linux-x86_64.AppImage`: `2567e21b8498b6593d26d899728ad086302647acfbcb5948bbc8766358669fcb`
- `Creation-SSH_0.6.11_linux-amd64.deb`: `cd10a93610caf3153c8ff7f711db84c2cf60576fb6dfce7187bfbfdac36b076f`

## v0.6.10 - Linux 正式版与 agent 部署事务加固

### 下载
- Windows 安装版: `Creation-SSH_0.6.10_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.10_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.10_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.10_android-arm64.apk`
- Android AAB: `C-SSH_0.6.10_android-arm64.aab`
- Linux AppImage: `Creation-SSH_0.6.10_linux-x86_64.AppImage`
- Linux deb: `Creation-SSH_0.6.10_linux-amd64.deb`

### 新增
- 首次提供独立 Linux 桌面正式版 AppImage 与 deb，全部从独立 `linux/` 工程构建和验证。
- CLI、Windows、Android、Linux 统一使用同一套 agent 部署事务；每轮使用唯一暂存/备份路径，并在覆盖前核对字节数与 SHA256。
- 增加跨客户端远端部署锁；已有锁绝不自动接管，活跃锁返回 busy，陈旧锁要求显式修复。

### 修复
- 修复跨平台 SQLite 数据根不一致导致“主机列表存在，修复 agent 却报告主机不存在”的问题；主机、凭据、SSH/repair、监控与 AI 统一使用同一数据库根。
- Unix 默认或显式 `CS_DATA_DIR` 均强制根目录/密钥目录 `0700`、SQLite `0600`，权限收紧失败即停止打开数据库。
- systemd 部署在 stop/覆盖前校验固定 `FragmentPath`、原始与有效 `ExecStart`、活动主进程真实路径，拒绝陌生同名单元和 drop-in 覆盖。
- 既有 unit 保持原 enable 状态；fresh unit 清理失败不再伪报成功。迁移先复核 `KillMode=process`，agent 更新不会连带结束持久化 tmux。
- readiness 与严格版本握手失败均执行两阶段回滚；旧 agent 真正恢复并返回有效协议响应后才清除备份和锁。
- 进程归属改为遍历 `/proc/<pid>/exe` 精确匹配，兼容 CentOS 7.9；Linux gzip payload 与裸 agent 逐字节、架构、版本、SHA256 全量核对，阻止陈旧 agent 入包。
- 2026-07-12 同版本刷新 Windows/Linux 桌面包:AI 助手主机与模型选择框不再显示外框内嵌原生小框,统一由外层绘制边框、焦点环和右侧箭头；版本保持 `0.6.10`,Android 资产不变。

### 验证
- 根 Rust workspace 全量测试、Clippy `-D warnings`、格式、平台边界、版本一致性与 gzip payload 门禁全部通过。
- CentOS 7.9 与 Ubuntu 24 完成真实部署、握手与 `MetricsSnapshot`；Ubuntu 真实 0.6.9 被拒绝后自动恢复 0.6.10。
- 故障注入覆盖 readiness 失败、有效 ExecStart drop-in、disabled unit、活跃/陈旧锁、tmux 存活与零残留。
- 刷新后的 Windows 正式便携包真实启动，主窗口、Tauri、隔离 SQLite、AI 主页面与独立 AI 窗口均正常，退出后任务进程和隔离数据清理完成。
- 最终 Android x86_64 测试包在 MuMu 整卸安装并真实部署 agent 0.6.10，验证 user-systemd、持久化终端、监控、页面切换与强停重启恢复；该测试包不上传。
- Android arm64 APK/AAB 为 `versionName=0.6.10`、SDK 24/36、仅 arm64；APK v2 与 AAB JAR 签名验证通过。
- 刷新后的 Linux AppImage/deb 在 Ubuntu 24 真实桌面会话启动，两者均通过进程存活、agent 结构化 Collector、SQLite `integrity_check=ok`、指标 `+4`、`0700/0700/0600` 权限与零残留验证；Wayland 下未把不可靠的 `xdotool` 可见窗口集合差虚报为通过。

### SHA256
- `Creation-SSH_0.6.10_x64-setup.exe`: `5EA8FC3CD3CE08DA004B062DF28DFA4F86F656275338D84C963C114FD193E82E`
- `Creation-SSH_0.6.10_x64_en-US.msi`: `F1E41543BE522BAF6940073450873A99B2FD709243BD3C6F20673FB4EF57C750`
- `Creation-SSH_0.6.10_portable-Windows-x64.zip`: `DCC71D79C8EE681E1F79A7D53AEAADED251A97CC8AD3C511178692994AA21A66`
- `C-SSH_0.6.10_android-arm64.apk`: `5D347EDC629D09A6C683BF7B82E0F06DC75DA87EFBB43E73DF7663749C100E5C`
- `C-SSH_0.6.10_android-arm64.aab`: `B45101EBBB40BAF66BEC2237BACE4E32AE2B82696A51F91C5F843CD846522E84`
- `Creation-SSH_0.6.10_linux-x86_64.AppImage`: `3E7B299DBD639AB27EC16CC7E5BA34540FD8C696FF9C96CAD58D26D37E67FE55`
- `Creation-SSH_0.6.10_linux-amd64.deb`: `2A1FEE0CB982ED886131D1416613B4A99A8D8B92C86E6EF2F28AB68099F11179`

## v0.6.9 - 可配置主机采集与首次刷新修复

### 下载
- Windows 安装版: `Creation-SSH_0.6.9_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.9_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.9_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.9_android-arm64.apk`
- Android AAB: `C-SSH_0.6.9_android-arm64.aab`
- Linux 桌面安装包本次暂缓发布。

### 新增
- Windows、Linux 与 Android 后台采集支持 `1–10` 跨主机并发,默认 `4`;采集间隔范围 `1–3600` 秒,默认 `6` 秒。
- Android 设置页新增采集间隔、跨主机并发和本地保留期设置,三项均持久化到本地 SQLite。
- 独立 Linux 客户端工程与构建门禁已落地,但安装包本次不进入公开 Release。

### 修复
- 修复首次采集已写入 SQLite、Hosts 页面却仍显示旧指标的问题,当前页面会在整轮写库完成后立即刷新。
- 统一 Windows/Linux Hosts 与 MonitorList 的事件和兜底轮询调度,合并重叠刷新并阻止迟到响应覆盖新结果。
- 修复采集设置可能混合新旧值的问题,三项设置改为单事务原子保存和单查询一致快照读取。

### 验证
- 根 Rust workspace 全量测试和 Windows/Linux/Android 前端生产构建通过。
- Windows 正式程序已真实验证停留在 Hosts 原页时首轮指标自动刷新。
- Android x86_64 `0.6.9` 测试包通过签名/版本/ABI 检查,已安装并在 MuMu 模拟器启动;该测试包不上传 Release。
- Android arm64 正式 APK/AAB 构建通过;APK 版本为 `0.6.9 (6009)`,SDK `24–36`,ABI 仅 `arm64-v8a` 且 v2 签名通过,AAB 也仅含 arm64 原生库。

### SHA256
- `Creation-SSH_0.6.9_x64-setup.exe`: `6ECF9CBB4A06440CE735C4EDD70F43F770DBBF774AEEC70FE74914D1FC19B3F1`
- `Creation-SSH_0.6.9_x64_en-US.msi`: `D9F4A11D8562093F5859530448EC4CD2CA317022391E1D504EB04B161661BF87`
- `Creation-SSH_0.6.9_portable-Windows-x64.zip`: `9B49B7D69F64E9FFC3386BA663962FFA7B06F8DEFCF14E8340795951713E0E09`
- `C-SSH_0.6.9_android-arm64.apk`: `4245852EAEB217AAC0F00F7731D30FDD011759D2F5BCB9811E49E383DFD9437F`
- `C-SSH_0.6.9_android-arm64.aab`: `FF5488C3547D1E42F83A6B5185BEDEF1BF03C370264CE1CDDCC4785158AB07DA`

## v0.6.8 - AI 工作台、监控摘要与全局自适应体验

### 下载
- Windows 安装版: `Creation-SSH_0.6.8_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.8_x64_en-US.msi`
- Windows 便携版: `Creation-SSH_0.6.8_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.8_android-arm64.apk`
- Android AAB: `C-SSH_0.6.8_android-arm64.aab`

### 新增
- 桌面端支持多个独立 AI 弹窗;桌面端与移动端统一历史、主机、权限、执行档、工具循环和上下文入口,便于并行处理不同主机与任务。
- AI 历史、权限模式、执行档、工具循环上限、自定义 AI 上下文窗口和上下文偏好持久化到本地 SQLite,重开页面或应用后继续沿用用户选择。
- AI 与其他短 agent 请求提供低/中/高/超级性能档位,对应 1/2/4/8 并发;检测到 SSH、channel 或 streamlocal 传输错误后,会按主机临时自动降级到 1 并发。
- 主机监控入口新增跨主机状态摘要,详情页支持 6 秒自动采集与可折叠侧栏,更适合持续巡检和窄屏操作。
- Android 支持跟随系统语言和主题,并落实 GitHub Issue #18 提出的移动端浅色模式需求。
- Windows、Android、五个公开资产固定名称与 Linux agent 版本统一为 `0.6.8`;agent 握手报告 `0.6.8`。iOS 和 macOS 不在本次发布范围内。

### 修复
- 修复桌面端与移动端 AI 工作台在页面切换、应用重启或多窗口使用后可能丢失历史入口、权限、执行档、工具循环或上下文偏好的问题。
- 修复高并发 AI 工具调用可能放大 agent 短请求传输故障的问题,统一并发保护并加入自动降级恢复路径。
- 修复 CentOS 等旧 OpenSSH 环境拒绝 direct streamlocal 后 agent 能力不可用的问题,客户端会自动使用 stdio bridge 兜底。
- 修复新增密钥主机后的凭据复用与认证回退路径:优先使用本地加密保险库中的密钥,需要时可回退到已保存密码或明确提示输入密码。
- 修复全局布局在桌面窄窗口、移动竖屏和动态内容下的适配问题;侧栏、工具栏、弹层、命令片段执行/结果容器与主要工作区会自适应可用空间。
- 修复 Android 软键盘遮挡 AI 输入区,以及 GitHub Issue #16 中 AI 回复内容越出卡片轮廓的问题。
- 修复 Android 系统语言与主题在部分生命周期或重开场景下未及时同步的问题,并完善跟随系统的切换反馈。
- 修复文件上传/下载在分块和断点续传完成阶段的完整性处理,避免不完整结果被标记为成功。

### 验证
- Windows 正式桌面版已完成构建、实机启动与本次相关功能验证。
- Ubuntu 与 CentOS 真实链路已验证 agent 通信及兼容兜底;DeepSeek AI 对话与工具循环 E2E 已通过。
- Android 正式构建与 AndroidX SplashScreen 静态 C 自动门禁已通过;MuMu x86_64 模拟器已从 Launcher 真实冷启动逐帧验证,系统 C、前端文字、大 C 与主界面连续衔接,无纯色空档或裁切。

### SHA256
- `Creation-SSH_0.6.8_x64-setup.exe`: `87FC035CF668A3DCC1F0DB9DC9F9DFD0762BFC4673B23E135253D6227B4C1A40`
- `Creation-SSH_0.6.8_x64_en-US.msi`: `B4F8611C3AF885378C2A908E61387381005138BC45CCBED0FC038DC08758CBE9`
- `Creation-SSH_0.6.8_portable-Windows-x64.zip`: `47629E99884378CBCD65CB0AD004C7AF6441492AA741F341F50F63C447842DB5`
- `C-SSH_0.6.8_android-arm64.apk`: `1EE2636F5004C4204FD48F58953819DC67D95F35C464FC420A102E243CE40753`
- `C-SSH_0.6.8_android-arm64.aab`: `B009C9739C3AE4CE42339639BDA45676D9C4DB1D3D7926244B28D27DAD2E889A`

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
