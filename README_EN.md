# C-SSH 0.9.7

[中文](README.md) · [Website](https://c-ssh.com) · [Installation and data](INSTALL_EN.md)

This release brings together recent improvements to the AI assistant, server browser, monitoring, cloud sync, and the Windows, macOS, Linux, and Android interfaces. It also addresses problems found during everyday use, including stalled conversations, confusing status displays, and connection switching.

**This release uses notifications and manual downloads only. Visit the download page to install it yourself. There are no forced upgrades or background downloads or installations.** iOS development and distribution remain paused.


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

### Concurrent AI tasks and clearer progress

- Configure how many independent read-only tasks may run at once, including independent tasks on the same host. Additional work is queued.
- Delegated tasks follow the main assistant's configured context window. Dependent work and changes requiring confirmation continue to respect ordering and permission boundaries.
- Responses, subtask progress, and available results appear incrementally. A compact activity summary keeps the conversation readable, with details available on demand.
- Interjections appear in the conversation, and pause, stop, and resume states are clearer.

### Token usage, context, and cache statistics

- See cumulative tokens, context usage, and cache hit rate. Android uses a compact three-value panel; desktop clients retain detailed statistics.
- Cumulative tokens include reported input and output across requests and subtasks. Context usage describes one main-assistant request, with a short explanation to distinguish it from the accumulated total.
- Previously confirmed values remain visible while a new turn or refresh is underway. Missing cache reports are shown as unknown rather than zero.
- Provider-specific usage fields are normalized and requests deduplicated. Cache hit rate is weighted by observable input tokens rather than averaging request percentages.

### Desktop server browser

- Browse through a server Agent using the server's network exit on Windows, macOS, and Linux. A local Google Chrome installation is required and checked before launch.
- Create multiple browser environments for one server, each with separate browsing data, a persistent fingerprint seed, and settings.
- Configure operating-system presentation, timezone, language, and permissions. Location, microphone, and camera access can use normal permission prompts.
- Improved reopening and recovery preserve environment data and identity after abnormal application exits.
- The Android server-browser entry remains hidden while mobile validation is paused.

## Fixed

### AI responsiveness, context, and history

- Reduced repetitive directory reads, repeated queries, and unnecessary tool cycles during simple host checks.
- Improved context preparation, stable prefixes, and history loading. Context handling follows the configured model window instead of imposing an unrelated small fixed limit.
- Improved tool-result handling and long-conversation storage while preserving original records.
- Addressed failures to send after pausing or interrupting, incorrect notices about another window taking over, and missing conversation content after reopening.
- Fixed Mac crashes in affected AI execution paths.
- Fixed Android history selection being invalidated by other conversations, history restoration changing the active model, and usage not refreshing after model-window changes.

### Monitoring and local storage

- Raised the configurable collection concurrency ceiling to 1000, with queuing and independent host scheduling. This is a configuration ceiling, not a throughput guarantee for every computer or network.
- Reduced database overhead and repeated work in frequent collection paths.
- Added selective compression for suitable communication payloads while leaving small responses uncompressed.
- Improved Agent status refresh after cloud sync and clarified initial collection states without requiring an application restart.
- Improved Windows Server Agent monitoring sampling costs and CPU usage.

### Browser launch and stability

- Improved first-launch reliability, preparation time, and retry behavior.
- Fixed unexpected browser closure triggered by affected verification pages, page events, and navigation waits.
- Browser environments retain separate cookies and other browsing data, with a stable identity across launches.
- Websites and network exits still determine when a verification challenge appears. This release does not promise CAPTCHA-free access or complete concealment of every local fingerprint.

### Agent access and Linux ARM compatibility

- Improved access to existing Agents from multiple devices and handling of dedicated public keys.
- Improved the detection, reuse, update, and readiness stages and their status messages.
- Updated Linux x86_64 and ARM64 Agents, improved independent-channel compatibility, and recorded targeted UOS and Kylin ARM validation.
- SSH-only hosts continue to respect their selected mode; updating the client does not automatically install an Agent on them.

### Manual cloud sync and data protection

- Reuse a valid data-protection verification within the same application process across page changes, checks, and subsequent sync operations. A new process requires verification again.
- Added select-all and cloud-default choices and improved the sync list and confirmation dialogs.
- Fixed Android's first-login download failure when the model catalog had not yet been cached. Downloads now obtain the required catalog themselves.
- AI deletion records with no matching local account are no longer shown as downloadable configurations or allowed to block host downloads. Necessary internal sync acknowledgements remain.
- Conflicts involving an existing local account still require a clear choice. Other accounts are not identified or removed by name. Sync remains manual and does not automatically upload AI conversations or memory.

### Interface, terminals, and Android

- Organized settings into four groups while preserving direct access to existing settings.
- Improved desktop window controls, Mac corners, theme consistency, confirmation dialogs, and affected dark-theme text contrast.
- Removed redundant Android context displays and corrected duplicated bottom safe-area spacing and keyboard-related layout problems.
- Fixed host-card terminal entry selecting the wrong host and connection waits preventing users from switching hosts or terminals.
- Late errors and authentication prompts from a previous connection no longer interfere with the newly selected terminal.

## Verified

The changes use their existing targeted shared-layer, interface, and native validation evidence. Release checks additionally cover version, architecture, embedded resources, signing, and packaging. These results are not presented as a fresh exhaustive test of every platform, distribution, or physical phone.

The Android first-login cloud download fix was exercised in the normal Android SDK Emulator client against an authorized test account, including configuration download and subsequent checks without warming the model catalog first.

macOS builds support Apple Silicon and remain ad-hoc signed, without Developer ID notarization. Android retains its existing APK signing identity. Keep important data before upgrading and do not open a migrated database with an older incompatible client.

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

## Data and source

AI conversations and memory remain local. Cloud sync transfers selected encrypted configuration. This repository contains public documentation, downloads, and the Creation Cloud production source mirror; client and Agent source code is not public.
