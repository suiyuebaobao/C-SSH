# C-SSH 0.9.6

[中文](README.md) · [Website](https://c-ssh.com/en) · [Installation and data preservation](INSTALL_EN.md)

Server operations for Windows, Apple Silicon Mac, Android and Linux: hosts, SSH/RDP, persistent terminals, monitoring, files and scoped AI. Local features do not require a Cloud login.

Announcement and manual downloads only; no upgrade reminders or forced updates. iOS is excluded and old versions remain available. SQLite stays at schema22, Android/iOS build number is 9006000, and wire versions and signing identities are unchanged.

## Downloads

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

## Added

- Classic and workbench interfaces coexist. Three desktop card sizes share one compact layout and retain the selected preference.
- Desktop workbench AI chat adds semantic Markdown, tool summaries, highlighted code with raw copying, reading-position handling, quotations and evidence-backed file or snippet diffs.
- Windows and Mac support browsing hosts and tool cards while an AI run stays bound to its original conversation. These two page adaptations are not claimed for Linux or mobile.

## Fixed

- Applicable file and monitor pages allow host switching during reads and reject stale results or authentication callbacks. Pending writes keep their guard across navigation.
- Deleting the selected host clears invalid read targets; the hidden monitor page no longer navigates to or automatically reads another host.
- Logs show safe setting values and reduce routine or successful autosave noise. Duplicate icons, card layout and idle animations in the hidden updater are corrected.
- Mac exit waits for saving. Linux Agent diagnostics use the shared DTO conversion and reject client-only preference fields without a wire change.

## Capabilities

| Area | Features |
|---|---|
| Hosts and projects | Create, edit, search, favorite, group; IPv4/domains, separate ports, passwords/keys and explicit proxy routes |
| SSH and Agent | Choose standard SSH or Agent per host, with shared identity, capability and error handling |
| Terminals | Linux tmux, Windows Terminal2/psmux and standard SSH; desktop tabs and mobile input/session recovery |
| Windows RDP | Remote desktops and DVC management within RDP, with certificate/account verification and a real-logon gate |
| Monitoring | CPU, memory, disk, load, network/disk IO, live/history charts, top processes and collection settings |
| Files | Browse, upload, download, create, rename, delete, edit text and verify hashes; native mobile file pickers |
| AI | Global/project/host scope, model accounts/bindings, permissions, confirmations, stop, history and local memory |
| System and apps | System information, processes, firewall and services; desktop Docker/systemd/Windows-service tools depend on server capabilities |
| Desktop tools | Windows/Mac/Linux port forwarding, saved commands, broadcasts, per-host results and access grants |
| Proxies | Direct, saved SOCKS5/HTTP CONNECT and explicitly selected official routes; no silent direct fallback |
| Cloud | Optional login, devices, explicit encrypted host/proxy/model-account sync and self-hosted service selection |
| Preferences | Nine languages, appearance, collection and data protection; diagnostic logging is off by default |

[Detailed features and platform differences](FEATURES_EN.md) · [Self-hosted Cloud](SELF_HOSTING_EN.md) · [Screenshot gallery](screenshots/README_EN.md)


## Verified

- Ten artifacts total 583,386,396 bytes and are bound to the current build and package receipts. Existing Mac/Windows real-VM SSH/SFTP, monitor switching, navigation and deletion regressions are reused without repeating unchanged passing checks.
- Windows x64/ARM64 packages passed per-architecture package, install, 0.9.4→0.9.6 upgrade, uninstall and data-preservation checks in a Windows 11 ARM VM. x64 updater epoch3 signatures were cryptographically verified; these are not Authenticode signatures.
- Mac passed ordinary arm64 Release, ad-hoc strict-signature and DMG checks; it remains unnotarized. Android retained the release signer and passed actual Manifest/DEX 0.9.6/9006000, 16KiB alignment, and same-signer upgrade with settings and 21 protected tables preserved in an API36 arm64 emulator.
- Linux deb/AppImage packages passed per-architecture embedding and isolated startup, log entry and preference preservation on Ubuntu24.04 X11. AppImage used its extracted entry point; FUSE mounting and all remote workflows were not retested.
- iPhone received a Simulator build/identity check only and is not uploaded. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.

## Data and source

Mac is arm64-only, ad-hoc signed and unnotarized; Windows installers have no Authenticode signature. Android keeps the release signer; this round directly verified upgrades from 0.9.4. Do not uninstall or clear data to work around a signer difference. This schema22 build cannot open higher-schema test data; retain that test build and its separate data.

Cloud receives only permitted, explicitly synced client-encrypted ciphertext. AI conversations, raw records, Skills and long-term memory stay local; selected context is sent to the chosen model provider. This public repository contains documentation, binaries and a whitelisted Cloud production-source mirror. Client, shared-core, AI and Agent sources are not public.

[Historical screenshots](screenshots/README.md) use synthetic sample data; their original versions and dates remain intact and are not 0.9.6 screenshots or native acceptance evidence.

## SHA256

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

[GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Release archive](https://github.com/suiyuebaobao/C-SSH/releases)
