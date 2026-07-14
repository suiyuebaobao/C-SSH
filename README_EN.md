[中文](README.md) | **English**

<div align="center">

# Creation-SSH (C-SSH)

### A new cross-platform SSH operations experience: native client, server-side tmux persistence, always-on monitoring, and a built-in AI assistant

[![Download Windows](https://img.shields.io/badge/Download-Windows-0078D6?logo=windows&logoColor=white)](../../releases/latest)
[![Download Android](https://img.shields.io/badge/Download-Android-3DDC84?logo=android&logoColor=white)](../../releases/latest)
[![Global](https://img.shields.io/badge/Global-Worldwide-2ea44f)](../../releases/latest)
[![Free Forever](https://img.shields.io/badge/Free-Forever-ff69b4)](../../releases/latest)
[![Open Source](https://img.shields.io/badge/Open%20Source-after%20iOS%20and%20macOS%20stable%20releases-orange)](../../releases/latest)

</div>

---

## What Is It

Creation-SSH is not another web ops panel, and it is not just a plain SSH terminal. It combines the native feel of tools like Xshell, structured capabilities from an always-on server-side agent, and tmux-grade persistent terminal sessions.

In one line: **native client + structured resident agent + persistent sessions**, a modern three-in-one SSH operations tool.

---

## Desktop Page Guide

> Screenshots below use sanitized demo data such as `example.com`; they do not include real servers or credentials.

<div align="center">

### Host Management
<img width="820" src="screenshots/hosts.png" alt="Host management" />

</div>

The home page manages SSH hosts, groups, favorites, search, agent deployment, and repair. Host creation supports password or OpenSSH private-key authentication, and credentials stay in the local encrypted vault.

<div align="center">

### AI Assistant
<img width="820" src="screenshots/ai.png" alt="AI assistant" />

</div>

The AI assistant works with host context: it can read metrics, inspect logs, edit files, and run commands under explicit permission modes. The top workspace keeps history, host selection, permissions, context usage, and performance presets close at hand; the desktop app also supports an independent AI pop-out window.

<div align="center">

### Terminal
<img width="820" src="screenshots/terminal.png" alt="Dual-mode terminal" />

</div>

In **persistent mode**, the agent drives tmux directly. After a disconnect, reboot, or device switch, reconnecting restores the full screen through `capture-pane`, so running tasks stay alive. **Direct mode** is a native PTY fallback that works even without the agent.

<div align="center">

### Monitoring Overview
<img width="820" src="screenshots/monitor-list.png" alt="Monitoring overview" />

</div>

The monitoring overview shows health across all hosts before you drill into one machine. It is designed for quick daily checks: online state, load, warnings, and recent cached status are visible without opening a terminal.

<div align="center">

### Monitoring Detail
<img width="820" src="screenshots/monitor.png" alt="Monitoring" />

</div>

The resident agent continuously samples CPU, memory, disk, network, disk I/O, and top processes. Live cards show current state, while historical data is stored in redb for time-range review.

<div align="center">

### File Manager
<img width="820" src="screenshots/files.png" alt="File manager" />

</div>

Browse remote files graphically with create, read, update, delete, online editing, permission viewing, chunked transfers, and resumable upload/download. File operations are provided by the agent in a structured way instead of being stitched together from shell commands.

<div align="center">

### Port Forwarding
<img width="820" src="screenshots/ports.png" alt="Port forwarding" />

</div>

Port forwarding uses SSH local forwarding to expose remote internal services safely on your machine. Local listeners bind to `127.0.0.1` by default to avoid accidental LAN exposure, and saved forwards can be rebuilt, stopped, or removed.

<div align="center">

### Command Snippets
<img width="820" src="screenshots/snippets.png" alt="Command snippets" />

</div>

Command snippets turn repeated operations into a local command library. Select multiple hosts, run a snippet, and review grouped results per host.

<div align="center">

### System Management
<img width="820" src="screenshots/sysmgmt.png" alt="System management" />

</div>

System management covers read-only system facts, process control, firewall ports, and SSH password changes. Destructive actions require confirmation and run as the SSH login user without extra privilege escalation.

<div align="center">

### App Center
<img width="820" src="screenshots/appcenter.png" alt="App Center" />

</div>

Install Docker itself in one click, deploy common containerized apps such as Nginx and Redis, and manage Docker containers, images, and systemd services. Destructive actions require confirmation and run as the SSH login user.

<div align="center">

### Access Grants
<img width="820" src="screenshots/grants.png" alt="Access grants" />

</div>

Access grants centralize the local vault, generated SSH keys, one-time authorization, and AI audit records. Credentials stay on the local device and are never uploaded.

<div align="center">

### Settings
<img width="820" src="screenshots/settings.png" alt="Settings" />

</div>

Settings collect AI provider configuration, custom context windows, tool-loop limits, system-language following, login password, desktop transparency, monitoring collection, and GitHub update checks.

---

## Mobile Page Guide (Android)

Desktop power in your pocket. The same persistent tmux sessions, monitoring, file management, and built-in AI assistant are available from Android.

<div align="center">
<img width="180" src="screenshots/mobile-login.png" alt="Mobile login" />
<img width="180" src="screenshots/mobile-hosts.png" alt="Mobile hosts" />
<img width="180" src="screenshots/mobile-terminal.png" alt="Mobile terminal" />
<img width="180" src="screenshots/mobile-files.png" alt="Mobile files" />
<img width="180" src="screenshots/mobile-monitor.png" alt="Mobile monitoring" />
<img width="180" src="screenshots/mobile-ai.png" alt="Mobile AI assistant" />
<img width="180" src="screenshots/mobile-sysmgmt.png" alt="Mobile system management" />
<img width="180" src="screenshots/mobile-me.png" alt="Mobile Me" />
</div>

### Mobile Hosts
The hosts page uses cards for server management, including creation, editing, deletion, agent deployment, and quick jumps into terminal, monitoring, and system management.

### Mobile Terminal
The terminal page keeps both persistent tmux and direct PTY modes, with mobile shortcut keys for Ctrl, Esc, Tab, and arrows.

### Mobile Files
The files page supports directory browsing, editing, downloading into the app sandbox, creating, renaming, deleting, and toggling hidden files.

### Mobile Monitoring
The monitoring page subscribes to live metrics and shows CPU, memory, disk, network, and top process state on a phone-sized layout.

### Mobile AI Assistant
The AI page keeps host selection, permissions, context, history, and configuration in mobile sheets. The input area avoids the soft keyboard so typed text stays visible.

### Mobile System Management
System management opens as an inner page from the host actions and covers system facts, firewall ports, process termination, and SSH password changes.

### Me / Login Gate
The Me page includes language, update checks, version information, login password, and local security settings. If a login password is configured, the app starts at the local login gate to unlock the vault.

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
| Linux desktop | Supported | Independent AppImage / deb |
| Server agent (Linux) | Supported | x86_64 / ARM64 static musl binary |
| macOS | Planned | Open source after the stable iOS and macOS releases |
| iOS | In development | Open source after the stable iOS and macOS releases |

---

## Global And Free Forever

Creation-SSH is built for users worldwide, with 9 built-in languages: Simplified Chinese, Traditional Chinese, English, Spanish, French, German, Portuguese, Russian, and Korean.

The product is **free forever**: no subscription, no paid tier, and no locked features.

---

## Open Source

**The project will be open-sourced after the stable iOS and macOS releases are published.** We want to bring a genuinely useful native SSH operations tool to the community and maintain it openly for the long term.

---

## Download

Grab the latest build from [**Releases**](../../releases/latest):

**Current stable version**: `v0.6.10`. `v0.6.11` has been moved to prerelease and is not recommended as a stable install.

All example configurations use placeholders such as `example.com`; replace them with your own server details.

## v0.6.11 (Prerelease)

> Real Ubuntu uninstall verification found that tmux may leave its socket pathname after a controlled shutdown, causing the current safety policy to stop before product-data cleanup. This build remains fail-closed and does not delete unknown resources, but a complete uninstall may not finish. The correction will ship under a new version without replacing these assets.

### Downloads

- Windows installer: `Creation-SSH_0.6.11_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.11_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.11_portable-Windows-x64.zip`
- Android arm64 APK: `C-SSH_0.6.11_android-arm64.apk`
- Android arm64 AAB: `C-SSH_0.6.11_android-arm64.aab`
- Linux AppImage: `Creation-SSH_0.6.11_linux-x86_64.AppImage`
- Linux deb: `Creation-SSH_0.6.11_linux-amd64.deb`

### Added

- Windows, Linux, and Android now reuse an authenticated SSH transport for the same host. Monitoring, files, AI, system management, and terminals can work in parallel on separate channels, reducing repeated logins and waits without one completed operation interrupting the others.
- The Windows/Linux desktop clients and Android can stay connected to the same host and work independently. After one client exits, monitoring and requests on the other continue, and persistent terminals remain available for reconnection.
- The Android terminal now uses a compact two-row toolbar that keeps the host, target IP address, connection status, persistent/standard terminal switch, window selector, and common actions in one top area, leaving more room for the terminal canvas.
- Android now supports both reconnectable persistent terminals and standard terminals that end when closed. Persistent windows receive unique `terminal-N` names by default, with safe renaming and normalization of older duplicate names.
- Android terminal display controls now include fit, fixed `80x24`, and custom sizes, plus a `1-24px` font range. Fixed/custom modes support two-dimensional canvas browsing, and size, font, and scrolling preferences are restored after restart.
- Android adds an on-demand shortcut overlay for Esc, Tab, Ctrl, arrow keys, and `-` without permanently shrinking the terminal. Copy uses the selection first, falls back to visible terminal content, and writes to the system clipboard.
- Windows and Linux use the system Save As dialog for file or directory downloads, while Android uses the Storage Access Framework (SAF) system document picker. Canceling does not start a connection or download; chosen destinations retain resume support and integrity verification.
- SSH connections now have clear staged 8-second failure boundaries for DNS resolution, TCP connection, SSH handshake, and authentication. A failed or timed-out stage returns immediately instead of repeating the wait with another credential.
- When no password is entered explicitly and the stored private key is explicitly rejected, the client can try the encrypted-vault password within the same SSH session. After authentication succeeds, it continues public-key repair to reduce future password prompts.

### Fixed

- Fixed fresh local databases sometimes reporting `database is locked` when multiple pages or background tasks initialized at once. AI, files, monitoring, host data, and preferences can now open and recover reliably from the same SQLite database.
- Fixed installed-but-stopped firewalld being reported as a query failure. The client now shows it as not running, keeps port actions disabled, and never starts or installs the firewall on its own.
- A failure in one feature channel no longer disconnects a healthy shared SSH transport. Reconnection occurs only after the connection is confirmed lost, and mutating operations that may already have arrived are not replayed automatically.
- Host-key trust now stops safely when its record cannot be read, parsed, or saved. The error is not treated as a first connection, the current session is not delivered, and no other credential path is attempted.
- Before deleting a host or reinstalling, C-SSH verifies that the related service, process, persistent session, data, and public key belong to C-SSH. If any resource cannot be verified, it stops and preserves the current state instead of touching another service, session, or key.
- Older leftovers that are confirmed to belong to C-SSH can now be recovered safely into a reinstallable state. Local host and credential records remain available for retry when remote cleanup is incomplete, while foreign or unknown resources always remain unchanged.

### Verified

- Shared connection reuse on Windows, Linux, and Android; simultaneous cross-client access; continued operation after one client exits; and real AI, file, monitoring, and both terminal workflows all passed verification.
- The Android compact toolbar, host IP, persistent/standard terminals, unique window names, `1-24px` fonts, sizing, scrolling, copying, shortcut overlay, and restart restoration all passed verification.
- Windows/Linux system Save As, the Android SAF system document picker, cancellation paths, resumed downloads, and download integrity checks all passed verification.
- Concurrent first open of a fresh SQLite database, staged 8-second SSH failure messages, and same-session credential recovery all passed verification.
- Safe stopping for host-key errors and deletion, preservation of foreign resources, and reinstall recovery for confirmed older leftovers all passed verification.
- The root workspace gates and Windows, Android, and Linux build tests passed; installed-but-stopped firewalld on CentOS correctly returned `NotRunning`.
- The production Windows app passed independent launch, shutdown, SQLite, and `0.6.11` version checks.
- The Android x86_64 test build entered real terminal, file, monitoring, and AI flows in MuMu without crashing; the arm64 APK/AAB passed package-name, version, ABI, and signature checks.
- The Linux deb/AppImage passed real installation, launch, shutdown, and SQLite checks in the authorized VM; both packages have identical payloads and bundle agent `0.6.11`.

### SHA256

- `Creation-SSH_0.6.11_x64-setup.exe`: `bf03f3805c28cdaf6d545e6b5bfac3d2ed0ec44265f591569c78be35fceb8c5b`
- `Creation-SSH_0.6.11_x64_en-US.msi`: `647b4b8978433385950b34578588366657206f2746eb38355f2102f01295a911`
- `Creation-SSH_0.6.11_portable-Windows-x64.zip`: `f319942c1710e794a78792b84dcc1e0a1178efb4b2b0d1dab1f205a832aa8b61`
- `C-SSH_0.6.11_android-arm64.apk`: `92246daa0cbcd0283e238bc02d729f497a94407c6c4efc384de7fd3787a061ab`
- `C-SSH_0.6.11_android-arm64.aab`: `3e3394bde08a9c8c96fcea6cc1660475ff509dec1d2ca588fa6032b0eaeee063`
- `Creation-SSH_0.6.11_linux-x86_64.AppImage`: `2567e21b8498b6593d26d899728ad086302647acfbcb5948bbc8766358669fcb`
- `Creation-SSH_0.6.11_linux-amd64.deb`: `cd10a93610caf3153c8ff7f711db84c2cf60576fb6dfce7187bfbfdac36b076f`

## Releases And Changelog

- See the prerelease assets and full notes in the [v0.6.11 Release](../../releases/tag/v0.6.11); stable installers remain under [Releases latest](../../releases/latest).
- Historical changes are tracked in [CHANGELOG_EN.md](CHANGELOG_EN.md).
- Release notes are bilingual and include Downloads, Added, Fixed, Verified, and SHA256 sections.

## Contact And Community

- WeChat: **`suiyue_creation`**
- QQ Group (AI Innovation Community): **[Join here](https://qm.qq.com/q/OWYQ9hwFWy)**

<div align="center">
<img src="screenshots/qq-group-qr.png" width="260" alt="QQ group QR - AI Innovation Community" />
<br/><sub>Scan to join the QQ group (AI Innovation Community) - Group No. 1041937161</sub>
</div>

Questions, feedback, or want to follow iOS / macOS / open-source progress? Come say hi.

---

<div align="center">

This repository is used only for public project introduction, screenshots, and release distribution. The source code is not hosted here yet and will be opened according to the commitment above.

</div>
