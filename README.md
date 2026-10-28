# C-SSH 0.9.7

[English](README_EN.md) · [官网](https://c-ssh.com) · [安装说明](INSTALL.md) · [更新日志](CHANGELOG.md)

本次更新集中改善服务器接入、AI 助手、服务器浏览器、监控采集与云端同步，并同步更新 Windows、macOS、Linux 和 Android 客户端。

## 1. 麒麟与 ARM 服务器适配

完善麒麟高级服务器操作系统 V10 ARM64 适配，支持其 64KiB 内存页环境。改善 Agent 接入、监控采集、文件操作、持久终端与命令执行的兼容性。

修正特定 SSH 环境下首次建立 Agent 通道失败，以及日志量较大时兼容性检测失败的问题。ARM Agent 同步更新至当前版本。

## 2. Agent 接入与多设备使用

更新 Linux x64、Linux ARM64 及 Windows Server Agent。

改善多台电脑、手机接入同一服务器的授权与复用流程，修正部分公钥授权冲突。完善已有 Agent 的识别、能力检测与更新流程，并修正云端同步后主机部署状态未及时刷新的问题。

## 3. 桌面服务器浏览器

Windows、macOS、Linux 支持通过服务器 Agent 使用本机 Chrome 浏览网站。

同一台服务器可建立多个浏览器环境，每个环境独立保存 Cookie、登录状态、浏览数据与固定指纹种子，并可设置显示系统、时区、语言，以及定位、麦克风和摄像头权限。

改善启动速度与首次打开成功率，修正部分网站、安全验证页导致浏览器意外关闭的问题，完善异常退出后的环境恢复，并增加未安装 Chrome 时的提示。

## 4. AI 并发执行与查询效率

增加可设置的 AI 任务并发与排队机制，支持多台主机，以及同一台主机上的独立只读任务同时执行。子 Agent 沿用主 Agent 本次设置的上下文窗口。

优化服务器状态检查流程，减少重复读文件、反复查询及不必要的工具步骤。有依赖关系或涉及修改的操作仍依序处理。

## 5. AI 即时回答、进度与会话恢复

回答随模型返回逐步显示，子任务进度、结果与错误及时更新。

聊天主界面改用精简的进度与结果摘要，完整工具操作可通过“查看详情”展开。

修正插话内容未显示、暂停后无法继续发送，以及中断、异常退出后部分会话无法正常恢复的问题。改善历史保存冲突判断，修正部分 Mac AI 崩溃问题。

## 6. AI 上下文、长会话与用量统计

上下文整理改为依据模型实际窗口，不再采用固定 256K 门槛；工具结果不再一律截取固定字数，大结果可按需分页读取。

长会话采用分块保存与加载，处理原先整份会话 256MiB 限制造成的阻断，减少重新打开长聊天与准备上下文的等待。

新增真实用量统计，显示 **累计使用 Token、上下文用量与缓存命中率**，统一处理不同模型接口的用量字段，并计入子任务。执行期间继续显示上次已确认数据，供应商未提供的缓存数据不再误作零命中。

“累计使用 Token”表示会话历次请求的累计用量；“上下文用量”表示主助手单次请求的输入量，两者分开说明。

## 7. 监控采集与数据传输优化

采集并发上限提高至可设置的 **1,000**，完善排队与调度，改善启动后首轮数据更新及慢主机影响其他主机的情况。

优化本机数据库读写，减少重复操作；较大的监控历史响应采用选择性压缩，降低传输量，小数据维持轻量传输。

## 8. 云端同步与密码验证

同一次程序运行期间，完成一次数据保护密码验证后，切页、检查、上传与下载可沿用验证，减少重复输入。

同步界面增加全选与默认采用云端选项，改善项目排列和操作提示。

修正 Android 首次登录后确认下载失败的问题；已删除且本机也不存在的 AI 账号，不再反复出现在待同步列表中。

## 9. Android 界面与操作修复

同步共享层的 AI、上下文、记忆与用量统计改动，使用底部用量摘要及简洁的三项详情。

修正底部大面积空白、键盘与安全区间距问题。修正从主机功能卡片进入终端时未选中对应主机，以及连接等待期间无法切换主机的问题。

修正 AI 历史对话无法选取、历史模型干扰当前设置，以及切换模型窗口后用量未刷新的问题。

## 10. 桌面界面统一与细节改善

Windows、macOS、Linux 的共用设置与 AI 界面统一调整。设置首页整合为四个分类卡片，子项可直接进入对应设置。

确认弹窗改用 C-SSH 自身样式，改善 Mac 窗口控制、圆角，以及部分深色弹窗文字不清楚的问题。

**更新方式：**本次版本只提供更新提示，请前往官网下载并自行安装；不强制更新，也不自动下载安装。

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

## 数据与源码

AI 对话和记忆保存在本机；Cloud 仅同步用户选择的加密配置。本仓库提供公开说明、下载及 Creation Cloud 生产源码镜像，客户端与 Agent 源码不在公开范围。
