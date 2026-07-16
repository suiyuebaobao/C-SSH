[中文](README.md) | **English**

<div align="center">

# Creation-SSH (C-SSH)

### Keep operating from your phone: persistent terminals, monitoring, files, and an AI assistant

[![Android](https://img.shields.io/badge/Download-Android-3DDC84?logo=android&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.apk)
[![Windows](https://img.shields.io/badge/Download-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)
[![Linux](https://img.shields.io/badge/Download-Linux-FCC624?logo=linux&logoColor=black)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)
[![Stable](https://img.shields.io/badge/stable-v0.6.13-2ea44f)](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13)

</div>

Creation-SSH is a cross-platform SSH operations client. Android is more than a read-only remote: it manages hosts, restores server-side tmux sessions, shows monitoring data, handles files, runs the AI assistant, and opens system-management workflows. The Windows and Linux desktop clients cover broader day-to-day operations.

Core capabilities are delivered through a structured resident agent on the Linux server, while standard terminals and port forwarding retain pure SSH paths. The current public stable release is **`v0.6.13`**. `v0.6.11` remains available only as prerelease history and is not recommended.

## v0.6.13 Highlights

- Windows/Linux host rows use a compact responsive metric grid with the real OS, CPU, memory, disk, load, uptime, and Normal/Paused/Failed states.
- Windows, Linux, and Android now retain standard and persistent terminal connections, screen state, and input across menu navigation, without duplicate prompts after reattachment.
- Repairing an agent immediately performs one real metrics collection; an optional system-information failure no longer discards valid dynamic metrics.
- Unreachable hosts can remove their local record after a clear remote-residue warning, while reachable hosts retain strict ownership auditing and safe cleanup.
- The Windows/Linux AI assistant disables input and sending for clearly offline hosts, and the Windows splash screen is centered against the full window.

## Android First

The same hosts and tmux sessions can continue across desktop and phone. Android `v0.6.13` ships as arm64 APK/AAB and passed real in-app workflow verification. Public Releases do not include the x86_64 emulator test build.

## Download

| Platform | Recommended download | Other production assets |
| --- | --- | --- |
| Android arm64 | [APK](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.apk) | [AAB](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/C-SSH_0.6.13_android-arm64.aab) for store distribution |
| Windows x64 | [EXE installer](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_x64-setup.exe) | [MSI](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_x64_en-US.msi) · [portable ZIP](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_portable-Windows-x64.zip) |
| Linux x86_64 | [AppImage](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_linux-x86_64.AppImage) | [Debian/Ubuntu deb](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.6.13/Creation-SSH_0.6.13_linux-amd64.deb) |

See the [v0.6.13 Release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.13) for release notes and SHA256 values, or [CHANGELOG_EN.md](CHANGELOG_EN.md) for history.

## Delivered Platforms

| Platform | Delivered in `v0.6.13` |
| --- | --- |
| Android | Host management, agent deployment entry, persistent/standard terminals, files, live monitoring, AI, system management, local login gate, and Me settings |
| Windows | Complete desktop workflow, distributed as EXE, MSI, and portable ZIP |
| Linux desktop | Independent AppImage/deb; public verification covers persistent-terminal reopen, monitoring, system/processes, files, AI, and reconnect after invalidation |
| Linux agent | x86_64 static musl binary deployed by the client over SSH; no extra public agent port is required |
| iOS / macOS | **Not released** and not part of the `v0.6.13` delivery |

## Main Pages

### Android

| Page | What it does |
| --- | --- |
| Hosts | Add, edit, and remove hosts; deploy/repair the agent; enter terminal, monitoring, and system management |
| Terminal | Switch between reconnectable tmux sessions and standard SSH PTY; manage windows, font, sizing, scrolling, copy, and mobile shortcut keys |
| Files | Browse, edit, create, rename, and delete remote files; choose download destinations through Android SAF with resume and integrity checks |
| Monitoring | View CPU, memory, disk, network, disk I/O, and top processes; background multi-host collection settings persist in local SQLite |
| AI assistant | Select host, model, permission profile, history, and context; tool execution is governed by permissions and confirmation |
| System management | Inspect system facts, processes, and firewall ports; confirm actions such as process termination and SSH password changes |
| Me / login gate | Manage language, theme, version, updates, and local security; a configured login password unlocks the local vault at startup |

### Real Android Screenshots (v0.6.13)

These screenshots come from real Android workflows and were reviewed for redaction before publication.

#### Host Management

<div align="center">
<img width="360" src="screenshots/mobile-hosts.png" alt="Android host management" />
</div>

View host connectivity and agent deployment status in one place, and add new hosts when needed. Select a host to continue to terminal, files, monitoring, AI, and other workflows.

#### Persistent And Standard Terminals

<div align="center">
<img width="360" src="screenshots/mobile-terminal.png" alt="Android persistent and standard terminals" />
</div>

Switch between reconnectable tmux sessions and standard SSH terminals while managing the active window. Persistent sessions can be reattached so command-line work can continue on mobile.

#### File Manager

<div align="center">
<img width="360" src="screenshots/mobile-files.png" alt="Android file manager" />
</div>

Browse remote directories, create files or folders, toggle hidden files, and refresh the listing. Android's system file picker chooses download destinations, while each item provides additional management actions.

#### Live Monitoring

<div align="center">
<img width="360" src="screenshots/mobile-monitor.png" alt="Android live monitoring" />
</div>

Monitor CPU, memory, load, network, disk usage, disk I/O, and top processes in real time. The page also shows monitoring health and uptime for quick mobile checks.

#### AI Assistant

<div align="center">
<img width="360" src="screenshots/mobile-ai.png" alt="Android AI assistant real response" />
</div>

Select a target host, model, and permission profile before chatting with the AI, with controls for context, history, and settings. The screenshot shows a real read-only response, while tool execution remains governed by permissions and confirmation.

### Windows And Linux Desktop

Windows exposes the complete desktop navigation below. An independent Linux desktop client is delivered; the public `v0.6.13` verification explicitly covers host connectivity, terminals, monitoring, files, AI, and system/process workflows.

| Page | What it does |
| --- | --- |
| Hosts | Groups, favorites, search, credential selection, plus agent deployment, repair, and status |
| AI assistant | Uses explicitly authorized host context for metrics, logs, files, and tools; desktop supports a separate AI window |
| Terminal | Dual tmux-persistent and standard SSH PTY modes, including persistent-window recovery after disconnects or device changes |
| Monitoring | Fleet health overview, per-host live details, and historical time-range queries |
| Files | Remote file management, online editing, chunked transfer, resume, and integrity verification |
| Port forwarding | SSH local forwarding, bound to `127.0.0.1` by default, with saved start/stop controls |
| Command snippets | Save common commands, run them across selected hosts, and group results by host |
| System management | System facts, processes, firewall ports, and SSH password management |
| App Center | Install Docker, deploy apps such as Nginx/Redis, and manage containers, images, and systemd services |
| Access grants | Review the local vault, SSH keys, one-time grants, and AI audit records |
| Settings | AI provider, language, appearance, local login, monitoring collection, and update checks |

### Real Desktop Screenshots (v0.6.13)

Windows and Linux share the same desktop experience. The screenshots below come from the production desktop candidate and were redacted before publication.

#### Host Management

<div align="center">
<img width="920" src="screenshots/hosts.png" alt="Desktop host management" />
</div>

Manage SSH hosts through groups, favorites, and search while reviewing agent deployment status. Each host row also surfaces the OS, CPU, memory, disk, load, uptime, and monitoring health.

#### Persistent And Standard Terminals

<div align="center">
<img width="920" src="screenshots/terminal.png" alt="Desktop persistent and standard terminals" />
</div>

Select a host and switch between persistent tmux sessions and a standard SSH PTY. Standard terminals keep their live state across menu navigation, while persistent windows can be reattached after disconnects or device changes.

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

Browse and search remote directories, show hidden files, and create, upload, download, edit, or refresh items. The listing includes sizes, modification times, and per-item actions.

#### AI Assistant

<div align="center">
<img width="920" src="screenshots/ai.png" alt="Desktop AI assistant" />
</div>

Select a host, model, and permission profile so the AI can read authorized metrics and system information and return a result. The page also provides history, context settings, and a separate AI window.

#### Command Snippets

<div align="center">
<img width="920" src="screenshots/snippets.png" alt="Desktop command snippets" />
</div>

Save frequently used commands as named, tagged snippets instead of typing them repeatedly. Select multiple hosts to execute a snippet once and review results by host.

#### Access Grants

<div align="center">
<img width="920" src="screenshots/grants.png" alt="Desktop access grants" />
</div>

Create an independent temporary SSH access key for a selected host, with the private key shown only when the grant is created. Revoke the grant at any time without sharing the host's long-term credentials.

#### Settings

<div align="center">
<img width="920" src="screenshots/settings.png" alt="Desktop settings" />
</div>

Configure system-language following, local login and vault protection, AI providers, appearance, and monitoring collection in one place. The About view exposes the current version and update check, while persistent settings stay local.

## Security Boundaries

- Private keys and passwords stay in the current device's local encrypted vault. They are not uploaded to servers or a C-SSH cloud; C-SSH does not provide a hosted credential service.
- The agent is reached through an SSH tunnel and listens only on a server-local Unix socket. It exposes no extra public port and runs as the current SSH login identity without self-elevation.
- Host-key anomalies stop the connection, destructive actions require explicit confirmation, and services, sessions, sockets, data, or public keys that cannot be proven to belong to C-SSH are not removed automatically.
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
