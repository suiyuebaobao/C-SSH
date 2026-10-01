[中文](README.md) | **English**

# C-SSH 0.9.2

A server-operations workspace for **Windows, Apple Silicon Mac, Android and Linux**: hosts, persistent terminals, RDP, monitoring, files and scoped AI. Local features work without a Cloud login.

> **0.9.2 is announced through notices only. Download and install manually; no upgrade prompt or forced update is sent.**
>
> Android keeps the 0.9.0 signer and can replace 0.9.0 and 0.9.1. Earlier official builds with a different signer cannot be replaced directly. Uninstalling clears local data. Read the [installation and data-preservation guide](INSTALL_EN.md), especially the note that AI conversations and memory are not included in Cloud sync. The Mac DMG is Apple Silicon only, ad-hoc signed and not notarized.

## Downloads

| Platform | Official 0.9.2 download |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.2_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.2_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.2_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.2_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_android-arm64.apk) |
| Windows 11 ARM64 · NSIS | [C-SSH_0.9.2_arm64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_arm64-setup.exe) |
| Windows 11 ARM64 · Portable | [C-SSH_0.9.2_portable-Windows-arm64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_portable-Windows-arm64.zip) |
| Linux x86_64 · deb | [C-SSH_0.9.2_amd64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_amd64.deb) |
| Linux x86_64 · AppImage | [C-SSH_0.9.2_amd64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_amd64.AppImage) |
| Linux ARM64 · deb | [C-SSH_0.9.2_arm64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_arm64.deb) |
| Linux ARM64 · AppImage | [C-SSH_0.9.2_aarch64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.2/C-SSH_0.9.2_aarch64.AppImage) |

[Website](https://c-ssh.com/en) · [Website downloads](https://c-ssh.com/en/downloads) · [Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.2) · [Installation](INSTALL_EN.md)

iOS is not uploaded in this release. No Intel Mac, MSI, AAB or x86 Android release package is provided. Windows requires WebView2 Runtime. Minimum package targets are macOS 13 and Android 7.0/API 24; this does not mean every supported OS has been tested.

## What changed

- **Linux desktop client:** x86_64 and ARM64 deb/AppImage packages, sharing desktop hosts, SSH/Agent, persistent terminals, RDP/DVC, monitoring, files, AI, proxies, port forwarding and broadcasts.
- **Native Windows ARM64:** NSIS and Portable packages, validated under an ordinary user on Windows 11 ARM64. Remote Windows management components remain x64.
- **Older Windows terminals:** the bundled psmux adds a WinPTY backend for Server 2016 without ConPTY. Modern Windows keeps ConPTY. Management components use the static CRT to reduce runtime dependencies on clean systems.
- **Reliability fixes:** host edit/delete waits, Windows domain connections, file-overwrite permission cleanup and Linux legacy-database migration.
- **Cleaner announcements:** sync-operation messages are removed from the announcement dialog; Cloud sync remains available.
- **Manual installation:** announced through notices, without upgrade prompts or forced updates.

[Full changelog](CHANGELOG_EN.md)

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

## Screenshots

[Windows](screenshots/WINDOWS_EN.md) · [Mac](screenshots/MACOS_EN.md) · [Android](screenshots/ANDROID_EN.md)

The [new 0.9.1 form](screenshots/v0.9.1/README_EN.md) and retained 0.9.0 full gallery render real Vue interfaces with synthetic data in a background browser. It illustrates the UI, not native-app, physical-device or live-server validation. Personal contact details are hidden in public screenshots.

![Windows host form and Setup export](screenshots/v0.9.1/windows-add-host-setup.png)
![Mac monitoring](screenshots/v0.9.0/macos-monitor-detail.png)

## Validation scope

- Windows x64/ARM64: isolated ordinary-user NSIS/Portable installation, startup, uninstall, manual 0.9.1-to-0.9.2 upgrade and data preservation. ARM64 also has live SSH/file/tmux and Server 2022 RDP/DVC/Terminal2 reconnection evidence.
- Mac: arm64, DMG mount, version, embedded resources and complete ad-hoc signature checks passed. Not notarized; this is not validation of every Mac model or the minimum OS.
- Android: package/version/ABI/original signer/16KiB alignment/resources passed; SDK Emulator upgrade from 0.9.1 retained settings and launched normally. No physical-phone claim.
- Linux: ARM64 and x86_64 Ubuntu 24.04.5 VMs, actual deb/AppImage startup, Secret Service, cold recovery and desktop feature checks. Other distributions and Wayland remain unvalidated; GPU-less Xvfb fixtures used a test-only WebKit graphics setting.
- Server 2016 WinPTY integration has live development-candidate evidence; this release's 0.9.2 Setup was also tested on local Server 2022. Issue #57's original installer error 46 was not reproduced and is not claimed fixed. Initial monitoring and immediate-detach close rejections remain recorded; later success is not a root-cause fix.

## Data and source code

Cloud does not relay SSH traffic. The client encrypts host credentials and model keys before explicit sync. AI conversations and memory stay local; selected context is sent to the chosen model provider. Deleting a local host does not uninstall its Agent or clean up remote sessions.

This public repository contains product information, screenshots, downloads and a filtered Creation Cloud production-source mirror. Client, shared-core, AI and Agent source code is not public.

## SHA256

- `eb55cf041a5c430ceb9264e278f54c889436322db0e4fffacfcf46bca45b818a`  `C-SSH_0.9.2_android-arm64.apk`
- `c20223ed8751abe9f8484672eb5c589d8c51c4f21fb9ec7dfb8c116ab01316fa`  `C-SSH_0.9.2_aarch64.AppImage`
- `34a2d102a3934a2cbbf013c5eca364381d9b8eab20a714fd0b2315543e081e51`  `C-SSH_0.9.2_amd64.AppImage`
- `13be972dcfc6bf860e3b587237493f4dca4b0b7df909b8976db7539780031142`  `C-SSH_0.9.2_amd64.deb`
- `287d2a4c0c193f9b9e1d9ea4c352b11b40b93ec32018d0eb88ec4a07cba630b9`  `C-SSH_0.9.2_arm64.deb`
- `4c12425965724d76f004b9d7bfe8744d6c56cc81d859ad906c0a2618af7a50b1`  `C-SSH_0.9.2_macOS-arm64.dmg`
- `0f72009dfd5b91b70e699d7ce29f13b329ed929b2bb1f2b8e57f1baf1912772c`  `C-SSH_0.9.2_arm64-setup.exe`
- `b9d9f0b148615d00b2d438039125857b22412b4cf4cf09395efa2fb27aaea0e3`  `C-SSH_0.9.2_portable-Windows-arm64.zip`
- `9e635e857025732e3e68fcced78ab983a1c735c0a322c7aaccb539f3f6743b4b`  `C-SSH_0.9.2_portable-Windows-x64.zip`
- `447a880a1e3c7a5841907e1247a6e48a832c7e5e36704f18a67f27eed437fb38`  `C-SSH_0.9.2_x64-setup.exe`

## Feedback and contact

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Website feedback](https://c-ssh.com/en/feedback)
- QQ community: [Join AI Innovation Community](https://qm.qq.com/q/OWYQ9hwFWy), group 1041937161.
- Languages: 简体中文, 繁體中文, English, Español, Français, Deutsch, Português, Русский and 한국어.

Older releases and their original assets/hashes are preserved in the [release archive](https://github.com/suiyuebaobao/C-SSH/releases).
