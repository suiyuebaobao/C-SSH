[中文](README.md) | **English**

<div align="center">

# Creation-SSH (C-SSH)

### Keep operating from your phone: persistent terminals, monitoring, files, and an AI assistant

[![Android](https://img.shields.io/badge/Download-Android-3DDC84?logo=android&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_android-arm64.apk)
[![Windows](https://img.shields.io/badge/Download-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.8)
[![macOS Test](https://img.shields.io/badge/macOS-v0.8.8%20Test-000000?logo=apple&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_macOS-universal_TEST-UNVERIFIED.dmg)
[![Stable](https://img.shields.io/badge/stable-v0.8.8-2ea44f)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.8)

</div>

Creation-SSH is an SSH operations client for Windows PCs and Android, with a macOS Universal test candidate also available. Android is more than a read-only remote: it manages hosts, restores server-side tmux sessions, shows monitoring data, handles files, runs the AI assistant, and opens system-management workflows. The Windows client covers broader day-to-day operations.

Creation-SSH provides explicit Agent and native SSH host modes. Agent mode retains persistent tmux sessions and server-side monitoring. SSH mode requires no installed agent and provides terminal access, port forwarding, SFTP file management, online monitoring, system management, app-center operations, and SSH AI tools. The current Windows and Android public stable release is **`v0.8.8`**; the same Release also provides a macOS `arm64 + x86_64` Universal test build.

> Windows security notice: the NSIS package in this release is not Authenticode-signed. Windows SmartScreen may show Unknown Publisher or require an extra confirmation. Download only from this repository's Release and verify the SHA256 values below.

> Windows upgrade notice: `v0.8.8` is the first release under a new updater trust root. Windows clients on `v0.8.7` or earlier must install `v0.8.8` manually once; automatic updates resume for releases after `v0.8.8`. Android is unaffected.

> macOS test notice: the current package uses ad-hoc signing and has not undergone Developer ID signing, notarization, or real-Mac acceptance. macOS may require users to allow it manually. It is not a production release, has no automatic updates, and is not listed as an official Creation Cloud download. Use it for testing only.

## v0.8.8 Highlights

- Cloud data-protection pages on Windows and Android now use explicit loading, ready, and error states. Legacy-envelope migration, first-time setup, change, reset, and retry actions follow the actual server state.
- Fixed correct passwords being rejected before legacy-envelope verification, manual sync remaining stuck on loading, and stale previews after Cloud state changes.
- Revoked login sessions disappear immediately from the client list, while real account/sync notifications and cross-device receipts remain available.
- Fixed mismatched tab and card backgrounds on the Android account page in the light theme.
- Starting with `v0.8.8`, Windows ships only NSIS and portable ZIP; MSI is no longer built or published. Android ships one arm64 APK and no AAB.
- Added a macOS 13+ Universal test candidate containing `arm64 + x86_64`, distributed as clearly named `TEST-UNVERIFIED` DMG and `.app.zip` files.

## Current Capabilities

- Windows and Android transparently wrap local keys using platform device capabilities. The Cloud data-protection password is not involved in startup, SSH, AI, or ordinary use.
- Creation Cloud provides accounts, device sessions, and manual sync. Host credentials and AI-provider Key/Token values, API endpoints, and model bindings are encrypted by the account data key; Cloud cannot read or use them.
- AI supports multiple local provider accounts and model bindings. Conversations and five-layer memory remain local and are never synchronized.
- AI permissions remain View, Edit, and Full Access and are independent from Global, Project, and Host scope. Canonical raw events and five-layer memory remain available when switching models.
- Ordinary scoped summaries use the shared cognition snapshot directly. Full original conversations are read only in explicit exact-evidence mode, and cognition recall never grants remote execution authority.
- Shared Rust orchestration handles transient AI retries and durably records each retry schedule before emitting it. A stop request can cancel the wait.
- Broadcast execution can mix Agent and native SSH hosts with common per-host results, isolation, and confirmation behavior.
- Windows installers, portable builds, and direct execution all use the adjacent `data` directory; file drag-out and download recovery remain inside the same isolated data root.
- Windows Frost, the compact host rail, real latency, and image messages, plus Android scoped AI and image messages, remain included in `v0.8.8`.
- Windows terminal right-click takes over only to copy an existing selection. With no selection, it preserves the remote terminal's right-click behavior.
- Windows and Android provide Contact Us cards for WeChat, QQ group, and WhatsApp, plus a compact mobile AI toolbar.
- Fixed legacy Cloud data-protection migration, change/reset actions, manual-sync preview recovery, and login-session list behavior.
- Fixed Android light-theme account-page background inconsistency.

## Android First

The same hosts and tmux sessions can continue across desktop and phone. Android `v0.8.8` ships one arm64 APK. No AAB is generated or uploaded, and x86_64 emulator test builds remain private.

## Download

| Platform | Recommended download | Other assets / notes |
| --- | --- | --- |
| Android arm64 | [APK](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_android-arm64.apk) | No AAB for this release |
| Windows x64 | [EXE installer](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_x64-setup.exe) | [portable ZIP](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_portable-Windows-x64.zip); no MSI for this release |
| macOS 13+ Universal (test) | [TEST-UNVERIFIED DMG](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_macOS-universal_TEST-UNVERIFIED.dmg) | [TEST-UNVERIFIED `.app.zip`](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.8.8/C-SSH_0.8.8_macOS-universal_TEST-UNVERIFIED.app.zip); not accepted on a real Mac |

### SHA256

- `FF15C6CD40D3FC6725A413BD7253AABC191BD76C78CD3AFF83AA255758907736`  `C-SSH_0.8.8_x64-setup.exe`
- `55B42F281725D3995B9117C85A9E688F51AD4F2359D2921768C52E6AB027FAA0`  `C-SSH_0.8.8_portable-Windows-x64.zip`
- `A2C98E7A81BB4E5A66B38A2C8096FE41951AC8B66DD3DFE9AA4C64E17A1E4F80`  `C-SSH_0.8.8_android-arm64.apk`
- `E150EA982F65E458539A7DF2A4E8E45B12B12CAAD0D1CD57DEB5AA785CAD4FA3`  `C-SSH_0.8.8_macOS-universal_TEST-UNVERIFIED.dmg`
- `6359C20F6D9F70C8DAA1E825972597FA4CC7BF40C08869A5B7166F7F85976403`  `C-SSH_0.8.8_macOS-universal_TEST-UNVERIFIED.app.zip`

### Release Verification

- Windows NSIS and Portable passed formal install, exit, uninstall-with-data-retention, manual `0.8.7 → 0.8.8` upgrade, and cleanup gates. Both carry Creation Cloud updater-signature metadata.
- Android x86_64 on MuMu passed the affected Cloud data-protection and manual-sync path. The production arm64 APK passed version, ABI, non-debug, v2 single-signer, and four embedded deployment-resource checks; this is not presented as physical-arm64 acceptance.
- The macOS candidate passed Universal architecture, ad-hoc signing-structure, and DMG integrity gates on GitHub's `macos-15` runner, and the public downloads match the candidate SHA256 values. Real-Mac installation, Gatekeeper, Keychain, UI, and network paths remain untested.

Linux client development is discontinued and frozen. Historical source and historical releases remain available only as records.

See the [v0.8.8 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.8.8) for downloads and release notes, or [CHANGELOG_EN.md](CHANGELOG_EN.md) for history.

## Platform Availability

| Platform | `v0.8.8` status and scope |
| --- | --- |
| Android | Host management, agent install and update/repair, persistent/standard terminals, file upload/download, live monitoring, AI, system management, local login gate, and Me settings |
| Windows | Complete desktop workflow, distributed as NSIS EXE and portable ZIP |
| macOS | **Public test build** distributed as Universal DMG / `.app.zip`; not accepted on a real Mac and not a production release |
| iOS | **Not released** and not part of the `v0.8.8` delivery |

The Linux client is no longer developed, tested, built, or released. The server-side Linux agent is not a Linux client.

## Main Pages

### Android

| Page | What it does |
| --- | --- |
| Hosts | Add hosts only after real SSH authentication; edit hosts; delete local state while explicitly retaining remote state; install or update/repair the agent; enter terminal, monitoring, and system management |
| Terminal | Switch between reconnectable tmux sessions and standard SSH PTY; manage windows, font, sizing, scrolling, copy, and mobile shortcut keys |
| Files | Browse, edit, create, rename, and delete remote files; use Android SAF for single-file upload or download destinations with chunking, resume, and integrity checks |
| Monitoring | View CPU, memory, disk, network, disk I/O, and top processes; background multi-host collection settings persist in local SQLite |
| AI assistant | Select host, model, permission profile, history, and context; tool execution is governed by permissions and confirmation |
| System management | Inspect system facts, processes, and firewall ports; confirm actions such as process termination and SSH password changes |
| Me / settings | Manage language, theme, version, updates, transparent local protection, and the optional Creation Cloud account |

### Android Product Screenshots

These screenshots come from the `v0.7.5` Android test UI and use only RFC 5737, `example.com`, and explicitly labeled offline simulated data. They did not connect to a real host, Cloud, or AI provider, and they are not evidence of physical-arm64 acceptance or complete manual GUI coverage.

#### Host Management

<div align="center">
<img width="360" src="screenshots/mobile-hosts.png" alt="Android host management" />
</div>

View connectivity and agent deployment status in one place, install or update/repair the agent, then add, edit, or delete hosts. Adding requires real SSH authentication first. Deletion ends only the local lifecycle and explicitly leaves the remote agent, authorized key, tmux sessions, and data intact. Adding the same ID or address later creates a new host without inherited data.

#### Persistent And Standard Terminals

<div align="center">
<img width="360" src="screenshots/mobile-terminal.png" alt="Android persistent and standard terminals" />
</div>

Switch between reconnectable tmux sessions and standard SSH terminals while managing the active window. Persistent sessions can be reattached so command-line work can continue on mobile.

#### File Manager

<div align="center">
<img width="360" src="screenshots/mobile-files.png" alt="Android file manager" />
</div>

Use the compact two-row toolbar to browse remote directories, collapse deep paths, create files or folders, and toggle hidden items. Android's system picker selects one local file for upload and also chooses download destinations.

#### Live Monitoring

<div align="center">
<img width="360" src="screenshots/mobile-monitor.png" alt="Android live monitoring" />
</div>

Monitor CPU, memory, load, network, disk usage, disk I/O, and top processes in real time. The page also shows monitoring health and uptime for quick mobile checks.

#### AI Assistant

<div align="center">
<img width="360" src="screenshots/mobile-ai.png" alt="Android AI assistant simulated demo" />
</div>

Select a target host, model, and permission profile before chatting with the AI, with controls for context, history, and settings. The screenshot shows an offline timeline explicitly labeled as simulated data; real tool execution remains governed by permissions and confirmation.

### Windows Desktop

Windows exposes the complete desktop navigation below and follows the host hard-delete and lifecycle-isolation contract.

| Page | What it does |
| --- | --- |
| Hosts | Groups, favorites, search, credential selection, plus agent deployment, repair, and status |
| AI assistant | Uses explicitly authorized host context for metrics, logs, files, and tools; desktop supports a separate AI window |
| Terminal | Dual tmux-persistent and standard SSH PTY modes, including persistent-window recovery after disconnects or device changes, plus multiple separate terminal windows for parallel work |
| Monitoring | Fleet health overview, per-host live details, and historical time-range queries |
| Files | Remote file management, online editing, chunked transfer, resume, and integrity verification |
| Port forwarding | SSH local forwarding, bound to `127.0.0.1` by default, with saved start/stop controls |
| Broadcast execution | Select hosts, enter a command or UTF-8 `.sh` file, freeze confirmation, execute through the agent, and review per-host results |
| System management | System facts, processes, firewall ports, and SSH password management |
| App Center | Install Docker, deploy apps such as Nginx/Redis, and manage containers, images, and systemd services |
| Access grants | Review the local vault, SSH keys, one-time grants, and AI audit records |
| Settings | AI provider, language, appearance, local login, monitoring collection, and update checks |

### Desktop Product Screenshots

These screenshots come from the `v0.7.5` Windows candidate UI and use only RFC 5737, `example.com`, and explicitly labeled offline simulated data. They did not connect to a real host, Cloud, or AI provider, and the eight captured pages are not a complete manual GUI acceptance run.

#### Host Management

<div align="center">
<img width="920" src="screenshots/hosts.png" alt="Desktop host management" />
</div>

Manage SSH hosts through groups, favorites, and search while reviewing agent status and live metrics. Deletion clears attributable credentials, session history, window persistence, and monitoring cache instead of allowing a later host to inherit old state.

#### Persistent And Standard Terminals

<div align="center">
<img width="920" src="screenshots/terminal.png" alt="Desktop persistent and standard terminals" />
</div>

Select a host and switch between persistent tmux sessions and a standard SSH PTY. Standard terminals keep their live state across menu navigation, while persistent windows can be reattached after disconnects or device changes. Right-click shows Copy only when a selection exists; otherwise, it remains available to the remote TUI or mouse mode.

#### Multi-host Monitoring Overview

<div align="center">
<img width="920" src="screenshots/monitor-list.png" alt="Desktop multi-host monitoring overview" />
</div>

Compare CPU, memory, and live status across hosts while controlling active collection. Select any host to open its detailed monitoring view.

#### Per-host Monitoring Details

<div align="center">
<img width="920" src="screenshots/monitor.png" alt="Desktop per-host monitoring details" />
</div>

Inspect CPU, memory, disk, swap, load, network, and disk I/O for one host. Trend charts show recent changes, while top-process data helps identify resource usage.

#### File Manager

<div align="center">
<img width="920" src="screenshots/files.png" alt="Desktop file manager" />
</div>

Browse and search remote directories, show hidden files, and create, upload, download, edit, or refresh items. File drag-out and downloads support interruption recovery and integrity verification, while the listing includes sizes, modification times, and per-item actions.

#### AI Assistant

<div align="center">
<img width="920" src="screenshots/ai.png" alt="Desktop AI assistant" />
</div>

Select a host, model, and permission profile so the AI can read authorized metrics and system information and return a result. Provably transient read-only failures wait a fixed five seconds and retry at most five times; requests that may have side effects are never replayed automatically.

#### Broadcast Execution

Select one or more hosts, enter a command or UTF-8 `.sh` file, freeze the confirmation, and execute through the structured agent protocol. Per-host results remain isolated and use redacted aliases; AI summarization is always an explicit action.

#### Access Grants

<div align="center">
<img width="920" src="screenshots/grants.png" alt="Desktop access grants" />
</div>

Create an independent temporary SSH access key for a selected host, with the private key shown only when the grant is created. Revoke the grant at any time without sharing the host's long-term credentials.

#### Settings

<div align="center">
<img width="920" src="screenshots/settings.png" alt="Desktop settings" />
</div>

Configure system-language following, the optional Creation Cloud account, AI providers, appearance, and monitoring collection in one place. Windows business state uses the adjacent `data` directory, while local keys are transparently wrapped by platform device capabilities.

## Security Boundaries

- Private keys and passwords first stay in the current device's local encrypted vault. Only an explicit manual sync uploads opaque ciphertext encrypted by the account data key; Creation Cloud cannot read or use those secrets.
- The agent is reached through an SSH tunnel and listens only on a server-local Unix socket. It exposes no extra public port and runs as the current SSH login identity without self-elevation.
- Host-key anomalies stop the connection, and a host is stored only after SSH authentication succeeds. Deletion requires explicit confirmation but removes only local state and never connects to or cleans the remote server; remote uninstall is a separate user action.
- Port forwarding binds to `127.0.0.1` by default. Users who choose another listen address are responsible for evaluating LAN exposure.
- AI tools are controlled by permission profiles and execution confirmation. When a third-party AI provider is used, selected conversations and context are processed under that provider's terms.

## Free, Languages, And Open-Source Plan

Creation-SSH is currently free forever, with no subscription, paid tier, or feature lock. The interface includes Simplified Chinese, Traditional Chinese, English, Spanish, French, German, Portuguese, Russian, and Korean.

**The current release is not open source.** This repository contains the public product introduction, screenshots, and Release assets only. The plan is to open-source the project after the official iOS and macOS releases. That is a roadmap statement, not a claim that source is available now or a commitment to a specific date.

## Contact

- WeChat: `suiyue_creation`
- QQ Group (AI Innovation Community): [Join here](https://qm.qq.com/q/OWYQ9hwFWy)

### QQ Group: AI Innovation Community

<div align="center">
<img width="300" src="screenshots/qq-group-qr.png" alt="QQ group QR code - AI Innovation Community" />
</div>

Scan the QR code or use the link above to join, Group No. `1041937161`. The group is for product experience, issue feedback, and future release discussions.
