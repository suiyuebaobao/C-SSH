[中文](README.md) | **English**

<div align="center">

# Creation-SSH (C-SSH)

### A new cross-platform SSH operations experience: native client, server-side tmux persistence, always-on monitoring, and a built-in AI assistant

[![Download Windows](https://img.shields.io/badge/Download-Windows-0078D6?logo=windows&logoColor=white)](../../releases/latest)
[![Download Android](https://img.shields.io/badge/Download-Android-3DDC84?logo=android&logoColor=white)](../../releases/latest)
[![Global](https://img.shields.io/badge/Global-Worldwide-2ea44f)](../../releases/latest)
[![Free Forever](https://img.shields.io/badge/Free-Forever-ff69b4)](../../releases/latest)
[![Open Source Soon](https://img.shields.io/badge/Open%20Source-Soon-orange)](../../releases/latest)

</div>

---

## What Is It

Creation-SSH is not another web ops panel, and it is not just a plain SSH terminal. It combines the native feel of tools like Xshell, structured capabilities from an always-on server-side agent, and tmux-grade persistent terminal sessions.

In one line: **native client + structured resident agent + persistent sessions**, a modern three-in-one SSH operations tool.

---

## Features

<div align="center">

### Host Management With A Lightweight Dashboard
<img width="820" src="screenshots/hosts.png" alt="Host management" />

</div>

Manage all your servers in one place. The host list includes a lightweight resource dashboard so you can see online status and load at a glance. Group, search, and connect quickly; credentials are encrypted locally and never uploaded.

<div align="center">

### Dual-Mode Terminal (Persistent tmux + Direct SSH)
<img width="820" src="screenshots/terminal.png" alt="Dual-mode terminal" />

</div>

In **persistent mode**, the agent drives tmux directly. After a disconnect, reboot, or device switch, reconnecting restores the full screen through `capture-pane`, so running tasks stay alive. **Direct mode** is a native PTY fallback that works even without the agent.

<div align="center">

### Always-On Monitoring
<img width="820" src="screenshots/monitor.png" alt="Monitoring" />

</div>

The resident agent continuously samples CPU, memory, disk, network, disk I/O, and top processes. Live cards show current state, while historical data is stored in redb for time-range review.

<div align="center">

### File Manager
<img width="820" src="screenshots/files.png" alt="File manager" />

</div>

Browse remote files graphically with create, read, update, delete, online editing, permission viewing, chunked transfers, and resumable upload/download. File operations are provided by the agent in a structured way instead of being stitched together from shell commands.

<div align="center">

### App Center
<img width="820" src="screenshots/appcenter.png" alt="App Center" />

</div>

Install Docker itself in one click, deploy common containerized apps such as Nginx and Redis, and manage Docker containers, images, and systemd services. Destructive actions require confirmation and run as the SSH login user without extra privilege escalation.

<div align="center">

### Built-In AI Operations Assistant
<img width="820" src="screenshots/ai.png" alt="AI assistant" />

</div>

The built-in AI assistant can read monitoring data, inspect logs, write files, edit configs, and run commands to help diagnose issues and write scripts. Five permission tiers plus per-action confirmation keep write and execution actions controllable and auditable. Both OpenAI-compatible APIs and Anthropic are supported.

---

## Mobile Companion (Android)

Desktop power in your pocket. The same persistent tmux sessions, always-on monitoring, and built-in AI assistant are available from Android.

<div align="center">
<img width="200" src="screenshots/mobile-hosts.png" alt="Hosts" />
<img width="200" src="screenshots/mobile-terminal.png" alt="Terminal" />
<img width="200" src="screenshots/mobile-ai.png" alt="AI assistant" />
<img width="200" src="screenshots/mobile-me.png" alt="Settings and languages" />
</div>

## Why C-SSH

- **Native client experience**: full-stack Rust + Tauri 2, fast startup, low resource use, and a desktop-first workflow.
- **Sessions that survive disconnects**: the agent drives tmux directly, so reconnecting restores long-running work.
- **Structured resident agent**: monitoring, files, apps, and system management are delivered by a server-side agent, not by fragile client-side shell stitching.
- **Built-in AI with two API families**: OpenAI-compatible APIs and Anthropic, with five permission tiers and execution confirmation.
- **Local encrypted credentials**: private keys and passwords stay in the local encrypted vault and are never uploaded.
- **Global by design**: the interface ships with 9 languages.
- **Desktop and mobile**: Windows desktop plus Android companion.

---

## Supported Platforms

| Platform | Status | Notes |
| --- | --- | --- |
| Windows | Supported | Desktop client, setup.exe / MSI / portable zip |
| Android | Supported | Mobile companion, arm64 APK |
| Server agent (Linux) | Supported | x86_64 / ARM64 static musl binary |
| iOS | In development | Client work in progress |

---

## Global And Free Forever

Creation-SSH is built for users worldwide, with 9 built-in languages: Simplified Chinese, Traditional Chinese, English, Spanish, French, German, Portuguese, Russian, and Korean.

The product is **free forever**: no subscription, no paid tier, and no locked features.

---

## Open-Source Commitment

**The project will become fully open-source once it reaches 500 GitHub stars or once the iOS client is complete, whichever happens first.** We want to bring a genuinely useful native SSH operations tool to the community and maintain it openly for the long term.

---

## Download

Grab the latest build from [**Releases**](../../releases/latest):

**Current latest version**: `v0.6.3`.

- **Windows**: download `Creation-SSH_0.6.3_x64-setup.exe` (recommended) or `Creation-SSH_0.6.3_x64_en-US.msi`.
- **Portable Windows**: download `Creation-SSH-portable-Windows-x64.zip`, unzip it, and run `Creation-SSH.exe`. Keep the bundled `resources` folder next to the executable.
- **Android**: download and install `C-SSH-android-arm64.apk`.

All example configurations use placeholders such as `example.com`; replace them with your own server details.

## Releases And Changelog

- Download the latest installers and read the full release notes in [GitHub Releases](../../releases/latest).
- Historical changes are tracked in [CHANGELOG_EN.md](CHANGELOG_EN.md).
- Release notes are bilingual and include Downloads, Added, Fixed, Verified, and SHA256 sections.

## Contact And Community

- WeChat: **`suiyue_creation`**
- QQ Group (AI Innovation Community): **[Join here](https://qm.qq.com/q/OWYQ9hwFWy)**

<div align="center">
<img src="screenshots/qq-group-qr.png" width="260" alt="QQ group QR - AI Innovation Community" />
<br/><sub>Scan to join the QQ group (AI Innovation Community) - Group No. 1041937161</sub>
</div>

Questions, feedback, or want to nudge us on iOS / open-source progress? Come say hi.

---

<div align="center">

This repository is used only for public project introduction, screenshots, and release distribution. The source code is not hosted here yet and will be opened according to the commitment above.

</div>
