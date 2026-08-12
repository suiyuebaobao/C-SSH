[中文](README.md) | **English**

<div align="center">

# Creation-SSH (C-SSH)

### Keep operating from your phone: persistent terminals, monitoring, files, and an AI assistant

[![Android](https://img.shields.io/badge/Download-Android-3DDC84?logo=android&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.7.7/C-SSH_0.7.7_android-arm64.apk)
[![Windows](https://img.shields.io/badge/Download-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.7.7)
[![Stable](https://img.shields.io/badge/stable-v0.7.7-2ea44f)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.7.7)

</div>

Creation-SSH is an SSH operations client for Windows PCs and Android. Android is more than a read-only remote: it manages hosts, restores server-side tmux sessions, shows monitoring data, handles files, runs the AI assistant, and opens system-management workflows. The Windows client covers broader day-to-day operations.

Creation-SSH provides explicit Agent and native SSH host modes. Agent mode retains persistent tmux sessions and server-side monitoring. SSH mode requires no installed agent and provides terminal access, port forwarding, SFTP file management, online monitoring, system management, app-center operations, and SSH AI tools. The current public stable release is **`v0.7.7`**. Older versions remain available as historical records.

> Windows security notice: the NSIS and MSI packages in this release are not Authenticode-signed. Windows SmartScreen may show Unknown Publisher or require an extra confirmation. Download only from this repository's Release and verify the SHA256 values below.

> Android `v0.7.7` upgrade notice: this release creates only fresh schema 12. Before `1.0.0`, older Android data is not migrated. When the installation generation changes, the previous local database, hosts, settings, credential references, and AI data are not retained.

## v0.7.7 Highlights

- Windows and Android now share SQLite schema 12. Local keys use transparent platform device wrapping, and ordinary startup no longer asks for a Cloud data-protection password.
- Cloud uses a random account data-key envelope. The protection password is used transiently only for setup, change, reset, upload, download, and explicit recovery. A password change rewraps the same key without re-encrypting Host/AI business ciphertext.
- Adding either Agent or SSH hosts must first complete real SSH negotiation, host-key verification, and authentication. Only then are host, mode, credential, and trust committed atomically; failure leaves no partial host.
- Ordinary deletion removes only local host, credential, trust, and related state. It never connects to the server or removes a remote agent, authorized key, tmux session, or server data.
- Android restores the AI tool-loop limit with a `1..=200` range, a default of `30`, and local SQLite persistence, including cold-start restoration after force-stop.
- Fixed Cloud registration in Traditional Chinese and other UI languages, and separated deterministic input/auth/conflict/rate-limit errors from service unavailability.
- All four Windows agent/tmux resources are embedded. Install directories contain no `resources` folder or runtime DLL, and the portable ZIP contains a single EXE.
- Production assets are exactly three Windows packages—NSIS, MSI, and portable ZIP—and one Android arm64 APK. The Linux client remains frozen, and Android produces or uploads no AAB.

## Current Capabilities

- Windows and Android transparently wrap local keys using platform device capabilities. The Cloud data-protection password is not involved in startup, SSH, AI, or ordinary use.
- Creation Cloud provides accounts, device sessions, and manual sync. Host credentials and AI-provider Key/Token values, API endpoints, and model bindings are encrypted by the account data key; Cloud cannot read or use them.
- AI supports multiple local provider accounts and model bindings. Conversations and five-layer memory remain local and are never synchronized.
- AI permissions remain View, Edit, and Full Access and are independent from Global, Project, and Host scope. Canonical raw events and five-layer memory remain available when switching models.
- Ordinary scoped summaries use the shared cognition snapshot directly. Full original conversations are read only in explicit exact-evidence mode, and cognition recall never grants remote execution authority.
- Shared Rust orchestration handles transient AI retries and durably records each retry schedule before emitting it. A stop request can cancel the wait.
- Broadcast execution can mix Agent and native SSH hosts with common per-host results, isolation, and confirmation behavior.
- Windows installers, portable builds, and direct execution all use the adjacent `data` directory; file drag-out and download recovery remain inside the same isolated data root.
- Windows Frost, the compact host rail, real latency, and image messages, plus Android scoped AI and image messages, remain included in `v0.7.7`.
- Windows terminal right-click takes over only to copy an existing selection. With no selection, it preserves the remote terminal's right-click behavior.
- Windows and Android provide Contact Us cards for WeChat, QQ group, and WhatsApp, plus a compact mobile AI toolbar.
- Fixed data-protection cold start and forgot-password reset, empty-cloud download previews, live AI configuration refresh, and host-monitor status projection.
- Fixed the Android new-device password dialog being blocked by an invisible reminder overlay and fixed corrupted data-protection copy.

## Android First

The same hosts and tmux sessions can continue across desktop and phone. Android `v0.7.7` ships one arm64 APK. No AAB is generated or uploaded, and x86_64 emulator test builds remain private.

## Download

| Platform | Recommended download | Other production assets |
| --- | --- | --- |
| Android arm64 | [APK](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.7.7/C-SSH_0.7.7_android-arm64.apk) | No AAB for this release |
| Windows x64 | [EXE installer](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.7.7/Creation-SSH_0.7.7_x64-setup.exe) | [MSI](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.7.7/Creation-SSH_0.7.7_x64_en-US.msi) · [portable ZIP](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.7.7/Creation-SSH_0.7.7_portable-Windows-x64.zip) |

### SHA256

- `DBFFE88A5A2DCC59A0AF463EF9A16520E3B207717DF8DA9A313095C4038FB513`  `Creation-SSH_0.7.7_x64-setup.exe`
- `F80767097D2BB3F0422B8652C7C45349F9882797240805AE7982840EA883CD57`  `Creation-SSH_0.7.7_x64_en-US.msi`
- `4C44A98EF6655F66C2FE76926D951536678CFD36F57D7119755DA529FA8E7252`  `Creation-SSH_0.7.7_portable-Windows-x64.zip`
- `C1321415BC0CF0AB4D9188D8DFDD5947E68E49CB9950C056C60584291D79E5B4`  `C-SSH_0.7.7_android-arm64.apk`

Linux client development is discontinued and frozen. Historical source and historical releases remain available only as records.

See the [v0.7.7 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.7.7) for downloads and release notes, or [CHANGELOG_EN.md](CHANGELOG_EN.md) for history.

## Delivered Platforms

| Platform | Delivered in `v0.7.7` |
| --- | --- |
| Android | Host management, agent install and update/repair, persistent/standard terminals, file upload/download, live monitoring, AI, system management, local login gate, and Me settings |
| Windows | Complete desktop workflow, distributed as EXE, MSI, and portable ZIP |
| iOS / macOS | **Not released** and not part of the `v0.7.7` delivery |

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
