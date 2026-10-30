# C-SSH 0.9.8

[English](README_EN.md) · [官网](https://c-ssh.com) · [安装说明](INSTALL.md) · [功能详解](FEATURES.md) · [自建云](SELF_HOSTING.md) · [更新日志](CHANGELOG.md)

本次更新面向 Windows、macOS、Linux 和 Android，集中处理主机接入、第三方云连接、Android 会话恢复，以及删除和诊断记录相关问题。客户端与自研 Agent 同步更新至 0.9.8。

## 1. 主机连接失败，提示更清楚

新增主机时，连接错误改用 C-SSH 自身样式的弹窗显示，并按实际失败阶段区分地址解析、网络连接、SSH 握手、身份验证和代理认证问题，方便判断应该检查哪一项。

关闭错误提示后保留已填写的连接资料，便于修改后重试。身份验证被服务器拒绝时，不再笼统地判定为“密码错误”；用户名、密钥和服务器认证策略也可能影响结果。

## 2. 相同 IP、不同 SSH 端口可以分别管理

修正相同地址使用不同 SSH 端口时，新增或同步主机可能被误认为重复的问题。现在按照地址和 SSH 端口识别连接端点，各自主机资料、凭据、信任记录与云同步映射保持独立。

已有主机的身份保持，不要求删除后重新添加，也不会因为新增另一个端口而覆盖原来的凭据。

## 3. 第三方云地址自动识别

在“设置 → 云服务 → 自建云”中，可以直接填写 IP、域名或带端口的地址，点击“测试连接”后查看识别结果，再保存并切换；不必每次手动输入协议前缀。

首次未指定协议时优先检测 HTTPS。经明确配置的可信内网云可使用 HTTP，不需要在每台访问设备上导入证书。界面会清楚显示实际协议，并提示 HTTP 传输未加密，仅适合可信内网。

明确指定或已经保存的 HTTPS 地址、HTTPS 证书验证失败，以及公开网络地址，都不会悄悄降级为 HTTP。连接检测不会携带原账号的密码或会话信息。官方云继续使用 HTTPS；自建服务端需采用支持此模式的新版本并明确启用内网 HTTP。

## 4. Android 锁屏、断网后的 AI 会话恢复

改善 Android 在锁屏、网络中断、重新联网或返回前台后，AI 会话无法继续发送、错误提示无法识别，以及保存状态未及时恢复的问题。

恢复时会核对原任务和本机保存状态，保留已收到的内容，并允许在原会话继续交流；不会自动重发模型请求或远端修改命令。修正相关保存回执的处理时序，减少中断后聊天内容与页面状态不一致的情况。

## 5. 减少删除主机时的不必要等待

修正删除带有服务器浏览器环境的主机时，清理流程重复等待自身资源锁，可能造成长时间停留在“正在删除”的问题。

删除完成后正确释放资源，再执行浏览器环境收尾；远端短时卸载尝试仍有时间限制，失败不会无限阻挡本机删除。

## 6. 降低 Agent 诊断记录的开销

诊断关闭时更早跳过记录处理；Linux Agent 对正常、短时的监控操作采用汇总记录，异常、慢操作和需要追踪的业务仍保留必要细节。

日志维护改为有界的周期任务，减少在请求处理中进行较重清理的机会。Windows Agent 同步调整相关维护调度。此优化针对诊断记录和维护开销，不降低实际监控采样频率，也不改动服务器其他程序的日志设置。

## 7. 云端后台账号删除修复

修正删除包含主机、AI 配置、代理或同步历史的账号时，可能被设备关联约束阻挡的问题。

相关同步依赖与账号在同一笔数据库事务中清理；出现错误时完整回滚，其他账号的数据保持不变。管理员保护、全局历史记录和审计要求继续保留。官方云已同步此项后端修正，自建云需更新服务端才能采用。

## 版本与更新方式

- Windows、macOS、Linux、Android 客户端及自研 Agent：0.9.8。
- 本次通过公告提示更新，请前往官网下载并手动安装；不强制更新，不自动下载安装。
- 使用原安装身份覆盖更新，保留既有本机资料；请勿为更新而卸载、清空数据或手动删除密钥。
- macOS 继续提供 Apple Silicon 版本；当前未取得 Apple Developer ID 公证，安装边界与上一版相同。
- iOS 开发及发布继续暂停；移动端服务器浏览器入口仍保持隐藏。

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.8_arm64-setup.exe | 55,120,724 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_arm64-setup.exe) |
| C-SSH_0.9.8_portable-Windows-arm64.zip | 50,865,532 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_portable-Windows-arm64.zip) |
| C-SSH_0.9.8_portable-Windows-x64.zip | 52,359,569 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_portable-Windows-x64.zip) |
| C-SSH_0.9.8_x64-setup.exe | 57,113,796 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_x64-setup.exe) |
| C-SSH_0.9.8_macOS-arm64.dmg | 55,033,590 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_macOS-arm64.dmg) |
| C-SSH_0.9.8_android-arm64.apk | 53,086,313 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_android-arm64.apk) |
| C-SSH_0.9.8_aarch64.AppImage | 127,085,064 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_aarch64.AppImage) |
| C-SSH_0.9.8_amd64.AppImage | 128,088,568 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_amd64.AppImage) |
| C-SSH_0.9.8_amd64.deb | 54,934,216 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_amd64.deb) |
| C-SSH_0.9.8_arm64.deb | 54,041,416 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_arm64.deb) |

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.8_arm64-setup.exe | `e79dc574d0278dc419bdc130eaf4201afd33b31029f40dbccf0fe222f475cbcb` |
| C-SSH_0.9.8_portable-Windows-arm64.zip | `c499806e7d5bef2a48691e083fb24ec789235ae2e939da24e3b3bffd77cd00cd` |
| C-SSH_0.9.8_portable-Windows-x64.zip | `9b926ad08286e5c7fefbbb9eed40293ad45f8a0f036af7de0922201610a40941` |
| C-SSH_0.9.8_x64-setup.exe | `e526564486f14f84aec0ab265cd7eb92c892d5dfbc759e62461548eefccc8e82` |
| C-SSH_0.9.8_macOS-arm64.dmg | `73a2d6136fb6f4d34aa245984eae008e732152807878349479b25b947e9a1af1` |
| C-SSH_0.9.8_android-arm64.apk | `add46254cfdfc322d575da5f36328822e3b3b97b9c10b1d41f4f5a603f41bda1` |
| C-SSH_0.9.8_aarch64.AppImage | `2873949dfbfda78456b19e1aecdeb8b8a58d134651adfc71b2e4536a32ccff20` |
| C-SSH_0.9.8_amd64.AppImage | `f936a01b9621173f66be8c43ce77c53d6fe9e5206f1b9d52a5453ac5f39cba3b` |
| C-SSH_0.9.8_amd64.deb | `00630c2cc0cb1fdf1d23d80bb83242c85a13e09f977b25b3cb29952ee2c41a20` |
| C-SSH_0.9.8_arm64.deb | `97105e890023f27d8b69f300745f09e248e2eb5542c68668ef2802e4a38ba2e4` |

AI 对话和原始记忆保存在本机；Cloud 仅同步用户选择的加密配置。公开仓库包含说明、下载和 Creation Cloud 生产源码镜像，客户端与 Agent 源码不在公开范围。
