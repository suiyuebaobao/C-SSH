[中文](README.md) | **English**

# C-SSH 0.9.1

A server-operations workspace for **Windows, Apple Silicon Mac and Android**: hosts, persistent terminals, RDP, monitoring, files and scoped AI. Local features work without a Cloud login.

> **0.9.1 is announced through notices only. Download and install manually; no upgrade prompt or forced update is sent.**
>
> Android keeps the 0.9.0 signer and can replace 0.9.0. Earlier official builds with a different signer cannot be replaced directly. Uninstalling clears local data. Read the [installation and data-preservation guide](INSTALL_EN.md), especially the note that AI conversations and memory are not included in Cloud sync. The Mac DMG is Apple Silicon only, ad-hoc signed and not notarized.

## Downloads

| Platform | Official 0.9.1 download |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.1_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.1_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.1_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.1_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.1/C-SSH_0.9.1_android-arm64.apk) |

[Website](https://c-ssh.com/en) · [Website downloads](https://c-ssh.com/en/downloads) · [Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.1) · [Installation](INSTALL_EN.md)

iOS is not uploaded in this release. No Intel Mac, Linux client, MSI, AAB or x86 Android release package is provided. Windows requires WebView2 Runtime. Minimum package targets are macOS 13 and Android 7.0/API 24; this does not mean every supported OS has been tested.

## What changed

- **Automatic address detection:** enter an IPv4 address or domain directly, with a separate port. The address-type selector is removed; existing IPv6 hosts stay compatible.
- **Embedded Windows management installer:** the DVC host form in Windows, Mac and Android exports a standard Setup ZIP. Android also offers system sharing; RDP-only mode hides the card. A Windows client download is no longer required to obtain Setup.
- **Offline export:** component version 0.9.1, ZIP size 13,308,604 bytes. No host, account or credential is exported, no remote connection is made and the client does not execute the EXE.
- **Notice-only manual upgrade:** no upgrade prompt or forced update is sent. Mac automatic updates are deferred; installation continues through a DMG.

Existing self-hosted Cloud, Windows RDP/DVC and Terminal2/psmux, proxies and scoped AI remain available. See the [changelog](CHANGELOG_EN.md).

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

The [new 0.9.1 form](screenshots/v0.9.1/README_EN.md) and retained 0.9.0 full gallery render real Vue interfaces with synthetic data in a background browser. It illustrates the UI, not native-app, physical-device or live-server validation. Personal contact details are hidden in public screenshots.

![Windows host form and Setup export](screenshots/v0.9.1/windows-add-host-setup.png)
![Mac monitoring](screenshots/v0.9.0/macos-monitor-detail.png)

## Validation boundaries

- Windows test VM: same-build NSIS/Portable checks, installation/startup/uninstallation, **manual** 0.9.0-to-0.9.1 installer upgrade and data preservation.
- Mac: arm64 build, DMG mounting, version/architecture and ad-hoc signature integrity. Not notarized; this is not verification of every Mac model or the minimum OS.
- Android: package/version/ABI/signature/16KiB alignment/deployment assets and same-signer 0.9.0 upgrade, startup, retained settings and system export/share in Android SDK Emulator. Earlier signer limitations remain; no physical-phone claim is made.
- Focused address, UI, Cloud-isolation and shared-runtime checks passed. Public IPv6, every OS combination and all degraded-network scenarios are outside the verified scope.

## Data and source code

Cloud does not relay SSH traffic. The client encrypts host credentials and model keys before explicit sync. AI conversations and memory stay local; selected context is sent to the chosen model provider. Deleting a local host does not uninstall its Agent or clean up remote sessions.

This public repository contains product information, screenshots, downloads and a filtered Creation Cloud production-source mirror. Client, shared-core, AI and Agent source code is not public.

## SHA256

- `87738a053666b9a44503cfba10f70db2a075d80c4f703fd857d6c7933974d49f`  `C-SSH_0.9.1_x64-setup.exe`
- `a19dd017d41e79fbd8cdfd1bdb159828eb4368c131b381fcdf537fdc543b3eba`  `C-SSH_0.9.1_portable-Windows-x64.zip`
- `bc0aa6ffb0459e66ea7efc8536c1c6fe8076b3cf1a4dec7c9bcc96a0b91f0a74`  `C-SSH_0.9.1_macOS-arm64.dmg`
- `9cf10f2049a6824ff9bb5cd5af95f3576a3964d3700ee0ccc77ccff63c74a20e`  `C-SSH_0.9.1_android-arm64.apk`

## Feedback and contact

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Website feedback](https://c-ssh.com/en/feedback)
- QQ community: [Join AI Innovation Community](https://qm.qq.com/q/OWYQ9hwFWy), group 1041937161.
- Languages: 简体中文, 繁體中文, English, Español, Français, Deutsch, Português, Русский and 한국어.

Older releases and their original assets/hashes are preserved in the [release archive](https://github.com/suiyuebaobao/C-SSH/releases).
