[中文](README.md) | **English**

# C-SSH 0.9.0

A server-operations workspace for **Windows, Apple Silicon Mac and Android**: hosts, persistent terminals, RDP, monitoring, files and scoped AI. Local features work without a Cloud login.

> **Download and install 0.9.0 manually. Automatic updates from older versions are unavailable.**
>
> Android uses a new installation signature and cannot directly replace an older official build. Uninstalling clears local data. Read the [installation and data-preservation guide](INSTALL_EN.md), especially the note that AI conversations and memory are not included in Cloud sync. The Mac DMG is Apple Silicon only, ad-hoc signed and not notarized.

## Downloads

| Platform | Official 0.9.0 download |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.0_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.0/C-SSH_0.9.0_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.0_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.0/C-SSH_0.9.0_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.0_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.0/C-SSH_0.9.0_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.0_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.0/C-SSH_0.9.0_android-arm64.apk) |

[Website](https://c-ssh.com/en) · [Website downloads](https://c-ssh.com/en/downloads) · [Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.0) · [Installation](INSTALL_EN.md)

iOS is not uploaded in this release. No Intel Mac, Linux client, MSI, AAB or x86 Android release package is provided. Windows requires WebView2 Runtime. Minimum package targets are macOS 13 and Android 7.0/API 24; this does not mean every supported OS has been tested.

## What changed

- **Apple Silicon Mac client:** shared desktop capabilities with native windows, Keychain and file pickers, distributed as a DMG.
- **IPv4 or domain host addresses:** only these two choices are shown, with a separate port. Domain resolution still supports IPv6, and existing IPv6 hosts remain usable.
- **Self-hosted Cloud selection:** the official Creation Cloud is the default; settings can point to a compatible HTTPS deployment. Login, sync and update sources are isolated by service.
- **Clearer Windows onboarding:** manually run the supplied Setup on the server, then connect using a Windows account and trusted RDP/DVC. Automatic deployment, OCR and automatic UAC installation are no longer offered.
- **Windows persistent terminals:** the matching Agent manages Terminal2/psmux; standard OpenSSH remains available. Linux continues to use tmux.
- **AI scope-history correction:** global can access all local scope histories, project only its project, and host only its host. Chats are not automatically synced between devices through Cloud.
- **Data, connection and UI fixes:** shared migration, Cloud switching, host refresh, RDP logon readiness, terminal input and system-information handling. See the [changelog](CHANGELOG_EN.md).

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
| Desktop tools | Windows/Mac port forwarding, saved commands, broadcasts, per-host results and access grants |
| Proxies | Direct, saved SOCKS5/HTTP CONNECT and explicitly selected official routes; no silent direct fallback |
| Cloud | Optional login, devices, explicit encrypted host/proxy/model-account sync and self-hosted service selection |
| Preferences | Nine languages, appearance, collection and data protection; diagnostic logging is off by default |

[Detailed features and platform differences](FEATURES_EN.md) · [Self-hosted Cloud](SELF_HOSTING_EN.md) · [Screenshot gallery](screenshots/README_EN.md)

## Screenshots

[Windows](screenshots/WINDOWS_EN.md) · [Mac](screenshots/MACOS_EN.md) · [Android](screenshots/ANDROID_EN.md)

The gallery renders the real 0.9.0 Vue interface with synthetic sample data in a background browser. It illustrates the UI, not native-app, physical-device or live-server validation. Personal contact details are hidden in public screenshots.

![Windows host workspace](screenshots/v0.9.0/windows-hosts.png)
![Mac monitoring](screenshots/v0.9.0/macos-monitor-detail.png)

## Validation boundaries

- Windows test VM: same-build NSIS/Portable checks, installation/startup/uninstallation, **manual** 0.8.8-to-0.9.0 installer upgrade and data preservation.
- Mac: arm64 build, DMG mounting, version/architecture and ad-hoc signature integrity. Not notarized; this is not verification of every Mac model or the minimum OS.
- Android: package/version/ABI/signature/16KiB alignment/deployment assets and same-signer test-build upgrade/startup/data retention in Android SDK Emulator. Older official signatures cannot directly upgrade; no physical-phone claim is made.
- Focused address, UI, Cloud-isolation and shared-runtime checks passed. Public IPv6, every OS combination and all degraded-network scenarios are outside the verified scope.

## Data and source code

Cloud does not relay SSH traffic. The client encrypts host credentials and model keys before explicit sync. AI conversations and memory stay local; selected context is sent to the chosen model provider. Deleting a local host does not uninstall its Agent or clean up remote sessions.

This public repository contains product information, screenshots, downloads and a filtered Creation Cloud production-source mirror. Client, shared-core, AI and Agent source code is not public.

## SHA256

- `02ce52d75dff8ec917d08ea98359e1be69da5e3364512273cda509de04df7d96`  `C-SSH_0.9.0_x64-setup.exe`
- `adf258281f6c9682037a63cbf96009ae64908f7b44f6d839507439809c249c9e`  `C-SSH_0.9.0_portable-Windows-x64.zip`
- `9a74d7b10297f07b623b22d36087d8c0da705e54bae5488e99e13e6e9d1ecff3`  `C-SSH_0.9.0_macOS-arm64.dmg`
- `e7fc1cc792ee8fb4fc9ba676298a812ec9a3395d52cbadef7d5353485704cc40`  `C-SSH_0.9.0_android-arm64.apk`

## Feedback and contact

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Website feedback](https://c-ssh.com/en/feedback)
- QQ community: [Join AI Innovation Community](https://qm.qq.com/q/OWYQ9hwFWy), group 1041937161.
- Languages: 简体中文, 繁體中文, English, Español, Français, Deutsch, Português, Русский and 한국어.

Older releases and their original assets/hashes are preserved in the [release archive](https://github.com/suiyuebaobao/C-SSH/releases).
