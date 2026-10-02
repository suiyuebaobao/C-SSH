[中文](README.md) | **English**

# C-SSH 0.9.3

A server-operations workspace for **Windows, Apple Silicon Mac, Android and Linux**: hosts, persistent terminals, RDP, monitoring, files and scoped AI. Local features work without a Cloud login.

> **0.9.3 is announced through notices only. Download and install manually; no upgrade prompt or forced update is sent.**
>
> Android keeps the 0.9.0 signer and can replace 0.9.0 and 0.9.1. Earlier official builds with a different signer cannot be replaced directly. Uninstalling clears local data. Read the [installation and data-preservation guide](INSTALL_EN.md), especially the note that AI conversations and memory are not included in Cloud sync. The Mac DMG is Apple Silicon only, ad-hoc signed and not notarized.

## Downloads

| Platform | Official 0.9.3 download |
|---|---|
| Windows x64 · NSIS | [C-SSH_0.9.3_x64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_x64-setup.exe) |
| Windows x64 · Portable | [C-SSH_0.9.3_portable-Windows-x64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_portable-Windows-x64.zip) |
| macOS · Apple Silicon | [C-SSH_0.9.3_macOS-arm64.dmg](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_macOS-arm64.dmg) |
| Android · arm64 | [C-SSH_0.9.3_android-arm64.apk](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_android-arm64.apk) |
| Windows 11 ARM64 · NSIS | [C-SSH_0.9.3_arm64-setup.exe](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_arm64-setup.exe) |
| Windows 11 ARM64 · Portable | [C-SSH_0.9.3_portable-Windows-arm64.zip](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_portable-Windows-arm64.zip) |
| Linux x86_64 · deb | [C-SSH_0.9.3_amd64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_amd64.deb) |
| Linux x86_64 · AppImage | [C-SSH_0.9.3_amd64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_amd64.AppImage) |
| Linux ARM64 · deb | [C-SSH_0.9.3_arm64.deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_arm64.deb) |
| Linux ARM64 · AppImage | [C-SSH_0.9.3_aarch64.AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.3/C-SSH_0.9.3_aarch64.AppImage) |

[Website](https://c-ssh.com/en) · [Website downloads](https://c-ssh.com/en/downloads) · [Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.3) · [Installation](INSTALL_EN.md)

iOS is not uploaded in this release. No Intel Mac, MSI, AAB or x86 Android release package is provided. Windows requires WebView2 Runtime. Minimum package targets are macOS 13 and Android 7.0/API 24; this does not mean every supported OS has been tested.

## What changed

- **Terminal keyboard handling:** Android adaptive terminals resize to the visible area, with corrected cursor and input-anchor positioning. Shortcut keys stay above the keyboard and leave room for the input line when expanded.
- **Accessible form closing:** add/edit host forms have a top-right close button that remains available while scrolling. Clicking outside the form preserves entered values.
- **Mobile terminal font:** Android/iOS default to size 6, preserving saved custom sizes.
- **Version alignment:** iOS is maintained at 0.9.3, without an iOS package upload.
- **Manual installation:** download and install this release yourself. Announcement only; no upgrade prompts or forced updates.

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

This release checks changed behavior and new packages: five-platform form components in a background browser; Android SDK Emulator keyboard handling and upgrade; Windows lifecycle checks on both architectures; Mac DMG/arm64/ad-hoc signatures; Linux packages and forms on Ubuntu 24.04/X11. Earlier functional evidence retains its original version scope. No claim covers a physical Honor phone, all keyboards, all Linux distributions, or a physical iPhone.

## Data and source code

Cloud does not relay SSH traffic. The client encrypts host credentials and model keys before explicit sync. AI conversations and memory stay local; selected context is sent to the chosen model provider. Deleting a local host does not uninstall its Agent or clean up remote sessions.

This public repository contains product information, screenshots, downloads and a filtered Creation Cloud production-source mirror. Client, shared-core, AI and Agent source code is not public.

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.3_android-arm64.apk | `261f028ed76462577f0bd369404a5c4e5a47cf6b7b1ff8422851b1e0401612e9` |
| C-SSH_0.9.3_aarch64.AppImage | `842c1ca1b01650a8e19a27f43cd4aef839e7e0255e34d0aa923591e86ac291a4` |
| C-SSH_0.9.3_amd64.AppImage | `26249275cc7d2e9f9e85cf4d47e89d11ca9740590fc3f6626328274cd214145b` |
| C-SSH_0.9.3_amd64.deb | `8686242edc6c470f039dbb6e710143e964c75da4e18750a18b6e57c408b88292` |
| C-SSH_0.9.3_arm64.deb | `d6b92a1b0d2f33144f6ab063703483f491a94092f69d7aeb1d22ba59ebcfe17f` |
| C-SSH_0.9.3_macOS-arm64.dmg | `d86c2cdf8563f53602f74e1d0200d95b79dd35b89bc08f6f55f236b88ba09785` |
| C-SSH_0.9.3_arm64-setup.exe | `98ee1a090ec8143a0fd47336319c86a9dd03877cd5b62c302dae8da294310091` |
| C-SSH_0.9.3_portable-Windows-arm64.zip | `8c419270eff33874887e2848c5167c77c05ca873d6b6ebdfcf2e45e431b4f35e` |
| C-SSH_0.9.3_portable-Windows-x64.zip | `bf290d13963885b8f5c6eac8da576b8aac731d26d20df0de204e39ad190aeb54` |
| C-SSH_0.9.3_x64-setup.exe | `0defc336bfe744b5f6b76917d3954c53dd5e0ccfa0c9b74b8973deb8f78a1b09` |

## Feedback and contact

- [GitHub Issues](https://github.com/suiyuebaobao/C-SSH/issues) · [Website feedback](https://c-ssh.com/en/feedback)
- QQ community: [Join AI Innovation Community](https://qm.qq.com/q/OWYQ9hwFWy), group 1041937161.
- Languages: 简体中文, 繁體中文, English, Español, Français, Deutsch, Português, Русский and 한국어.

Older releases and their original assets/hashes are preserved in the [release archive](https://github.com/suiyuebaobao/C-SSH/releases).
