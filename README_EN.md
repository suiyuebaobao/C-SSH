# C-SSH 0.9.7

[简体中文](README.md) · [Website](https://c-ssh.com/en) · [Installation](INSTALL_EN.md) · [Changelog](CHANGELOG_EN.md)

This release improves server connectivity, the AI assistant, Server Browser, monitoring collection, and cloud synchronization across Windows, macOS, Linux, and Android.

## 1. Kylin and ARM server compatibility

Improves compatibility with Kylin Linux Advanced Server V10 ARM64, including its 64KiB memory-page environment. The changes cover Agent connectivity, monitoring, file operations, persistent terminals, and command execution.

Fixes initial Agent channel failures in specific SSH configurations and compatibility checks that failed when the relevant logs were large. The ARM Agent is updated to the current version.

## 2. Agent connectivity across devices

Updates the Linux x64, Linux ARM64, and Windows Server Agents.

Improves authorization and Agent reuse when multiple computers or phones connect to the same server, including fixes for some public-key authorization conflicts. Improves existing-Agent detection, capability checks, and update handling, and fixes deployment status not refreshing promptly after cloud synchronization.

## 3. Desktop Server Browser

Windows, macOS, and Linux can use local Chrome through the server's Agent.

Each server can have multiple browser environments, with separate cookies, login sessions, browsing data, and fixed fingerprint seeds. Settings include the displayed operating system, timezone, language, location, microphone, and camera permissions.

Improves launch speed and first-launch reliability, fixes unexpected browser closure on some websites and verification pages, improves recovery after an abnormal exit, and adds a reminder when Chrome is not installed.

## 4. AI concurrency and query efficiency

Adds configurable AI task concurrency and queuing. Independent read-only tasks can run concurrently across servers or on the same server. Workers inherit the main Agent's context-window configuration for the current run.

Improves server-status checks by reducing repeated file reads, duplicate queries, and unnecessary tool steps. Dependent operations and changes still run in the required order.

## 5. Live AI responses, progress, and conversation recovery

Responses appear as the model returns them, with timely worker progress, results, and errors.

The main conversation shows compact progress and result summaries. Full tool activity is available under View details.

Fixes missing interjection messages, sending failures after pausing, and some conversation recovery failures after interruption or abnormal exit. Improves saved-history conflict detection and fixes some Mac AI crashes.

## 6. AI context, long conversations, and usage

Context preparation follows the model's actual window instead of a fixed 256K threshold. Tool results are no longer uniformly cut to a fixed character count; large results can be read in pages as needed.

Long conversations use chunked storage and loading, addressing the previous 256MiB whole-conversation limit and reducing waits when reopening long chats or preparing context.

Adds actual usage reporting for **cumulative tokens used, context usage, and cache hit rate**, normalizes usage fields across model APIs, and includes worker usage. The last confirmed values remain visible during execution. Missing cache data is not treated as a zero hit rate.

Cumulative tokens used counts usage across requests in the conversation. Context usage measures the main assistant's input for one request; the two are explained separately.

## 7. Monitoring and transfer efficiency

Raises the configurable collection concurrency limit to **1,000**, improves queuing and scheduling, and reduces delays during the first refresh and the impact of slow hosts on other hosts.

Optimizes local database access and reduces duplicate operations. Larger monitoring-history responses use selective compression to reduce transfer volume, while small responses remain lightweight.

## 8. Cloud synchronization and password verification

After data-protection password verification, the running application reuses that verification when changing pages, checking changes, uploading, and downloading.

Adds Select all and a default cloud choice, with clearer sync-item layout and prompts.

Fixes download confirmation failures after the first Android login. Deleted AI accounts that are also absent locally no longer repeatedly appear as pending sync items.

## 9. Android interface and interaction fixes

Brings shared AI, context, memory, and usage changes to Android, with a usage summary at the bottom and a compact three-value detail panel.

Fixes excessive bottom space and keyboard/safe-area spacing. Fixes opening the wrong host's terminal from a host action card and being unable to change hosts while a connection is pending.

Fixes AI history selection, restored history interfering with the active model settings, and usage not refreshing after the model context window changes.

## 10. Desktop interface consistency

Aligns shared settings and AI interfaces across Windows, macOS, and Linux. Settings are organized into four category cards with direct access to individual settings.

Confirmation dialogs use C-SSH's own style. Improves Mac window controls and corners, and text contrast in some dark dialogs.

**Updating:** This release provides an update notice. Download the new client from the official website and install it yourself; there is no forced update or automatic download and installation.

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

## Data and source

AI conversations and memory remain local. Cloud sync transfers selected encrypted configuration. This repository contains public documentation, downloads, and the Creation Cloud production source mirror; client and Agent source code is not public.
