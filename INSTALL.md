# C-SSH 0.9.0 安装与数据保全

[下载](README.md#下载) · [English](INSTALL_EN.md)

> **本次必须自行下载并安装，旧版无法自动更新。**

## 安装前

- 只使用官网或 `suiyuebaobao/C-SSH` 的 v0.9.0 下载，按首页 SHA256 核对文件。
- 退出正在使用的 C-SSH，再处理安装或替换程序。
- 先保全重要数据。Cloud 仅手动同步主机、代理和模型账户等加密配置，**不包含 AI 对话与长期记忆**。
- 保留数据保护密码；它与网站登录密码、本地设备密钥用途不同。

## Windows x64

1. 安装版使用 `C-SSH_0.9.0_x64-setup.exe`；便携版解压 ZIP 后运行 `C-SSH/C-SSH.exe`。
2. Windows 需要已安装的 WebView2 Runtime。安装包没有 Authenticode 签名，系统可能提示未知发布者。
3. 使用原安装目录或保留便携程序旁的 `data`，不要把空目录覆盖到原数据上。不要在程序运行时替换 EXE。
4. 本次 Windows 更新签名与旧正式版不同，不能从旧版自动下载升级；请手动运行新安装器或替换便携程序。
5. 旧 MSI 用户需先保全原数据，再按原安装方式处理旧安装，不要混用安装器直接覆盖。

首次管理 Windows 服务器时，从 Windows 客户端目录取得 `Windows/C-SSH-Windows-Setup.exe`，复制到目标服务器手动运行一次并完成系统确认；之后使用客户端的 Windows 账号连接流程。便携版首次启动会按内嵌资源补齐这个文件。Setup 不是通用应用卸载工具，也不需要每台客户端重新安装服务器组件。

## M 系列 Mac

1. 下载 `C-SSH_0.9.0_macOS-arm64.dmg`，打开后将 `C-SSH.app` 拖到“应用程序”。
2. 仅支持 Apple Silicon M 系列，最低系统目标 macOS 13；不提供 Intel/Universal 版本。
3. 应用使用 ad-hoc 签名，**未经 Apple 公证**。首次打开可能受系统安全检查限制；确认官方来源与文件摘要后，可按 [Apple 官方说明](https://support.apple.com/zh-cn/102445) 在“系统设置 → 隐私与安全性”处理该应用的打开许可。
4. 数据位于当前用户的 Application Support，密钥由钥匙串保护。替换应用时不要删除应用数据或钥匙串条目；系统请求钥匙串权限时在本机处理。
5. 当前 Mac 不提供自动更新，后续版本也以实际发布说明为准。

## Android arm64

1. 下载 `C-SSH_0.9.0_android-arm64.apk`，由系统安装器安装；按系统提示允许所用下载应用安装 APK。
2. 本次只提供 arm64 APK，最低系统目标 Android 7.0/API 24；不提供 AAB 或 x86 发布包。
3. **0.9.0 与旧正式版签名不同，不能直接覆盖旧正式版。** 系统报告安装冲突时，不要反复安装或直接清除应用数据。
4. **卸载旧版会清除它的本地数据。** 请先确认需要保留的数据已有独立备份。AI 对话和记忆不在 Cloud 同步范围；无法确认数据保全时，请保留旧版，暂勿卸载。
5. 本次在 SDK Emulator 中验证的同签名测试包覆盖保留数据，不代表旧正式版可直接覆盖。

## 首次使用

添加主机时选择 IPv4 或域名，地址不带协议和端口，端口单独填写。确认 SSH 主机密钥或 RDP 证书后再继续。Cloud 登录可选；自建云通过“设置 → 云服务”配置，详见[自建云指南](SELF_HOSTING.md)。

iOS 本次不提供 IPA、TestFlight 或下载入口。问题反馈请使用 [Issues](https://github.com/suiyuebaobao/C-SSH/issues) 或[官网反馈](https://c-ssh.com/feedback)，不要提交密码、私钥、Token 或真实会话正文。
