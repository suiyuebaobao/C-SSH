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

<div align="center">
<img width="180" src="screenshots/mobile-hosts.png" alt="Android host list" />
<img width="180" src="screenshots/mobile-terminal.png" alt="Android persistent terminal" />
<img width="180" src="screenshots/mobile-files.png" alt="Android file manager" />
<img width="180" src="screenshots/mobile-monitor.png" alt="Android live monitoring" />
<img width="180" src="screenshots/mobile-ai.png" alt="Android AI real response" />
</div>

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

<div align="center">
<img width="400" src="screenshots/hosts.png" alt="Desktop host management" />
<img width="400" src="screenshots/terminal.png" alt="Desktop terminal" />
<img width="400" src="screenshots/monitor-list.png" alt="Desktop monitoring overview" />
<img width="400" src="screenshots/monitor.png" alt="Desktop monitoring" />
<img width="400" src="screenshots/files.png" alt="Desktop file manager" />
<img width="400" src="screenshots/ai.png" alt="Desktop AI assistant" />
</div>

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
