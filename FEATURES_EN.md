# C-SSH 0.9.4 Feature Guide

[Home](README_EN.md) · [Installation](INSTALL_EN.md) · [Gallery](screenshots/README.md)

This release requires manual installation. The sections below describe available entry points and their platform or server-capability limits.

## How it works

### Four platforms

Windows x64/ARM64, Apple Silicon Mac arm64, Android arm64 and Linux x86_64/ARM64 share core capabilities with desktop and phone layouts. 0.9.4 · Manual installation.

### SSH and RDP connections

Enter an IPv4 address or domain and a separate port. Domains may resolve to IPv4 or IPv6; existing IPv6 hosts remain compatible. New host form: IPv4 / Domain.

### Agent or standard SSH

Agent mode provides structured management and persistent services; standard SSH uses existing PTY, SFTP and related services. Choose per host.

### Persistent terminals

Linux uses tmux; Windows uses the matching Agent and Terminal2/psmux. Reconnect to an existing session after closing the client. Not a guarantee for every failure scenario.

### Files and system tools

Browse, transfer, edit and verify files. Monitoring, processes, firewall and service controls depend on the target capabilities. Linux and Windows capabilities are negotiated.

### One manual server setup

For Windows management, run the supplied Setup on the server once, then connect with a Windows account. No automatic deployment or OCR installation.

### Local-first operation

Use local server tools without a Cloud login. Optional accounts provide device management and explicit end-to-end encrypted sync. Cloud does not relay SSH traffic.

## New in 0.9.4

- **Bulk host import**: Import CSV or OpenSSH configuration, preview and select entries, then create pending connections. A host becomes active only after credentials are supplied and real authentication succeeds. Previewing is not connecting; passwords and private keys are not stored in the plaintext pending queue.
- **Skippable first-use guide**: Discover connection, import and operations entry points, or skip directly to the workspace. The choice survives a cold restart and the guide can be reopened in Settings. Upgrades with existing data do not force onboarding.
- **Automatic AI operations Skills**: Experience is learned only after tools execute and acceptance succeeds. Later tasks reuse it only when the current environment matches. No manual save or adoption step is required; the normal UI shows a single light notice when past experience is referenced. Existing permissions and confirmations still apply. Skills, AI conversations, raw records and long-term memory remain local and are excluded from Cloud sync.
- **Readable operation-log rows**: One business operation per row, with time, level, local host name and a trustworthy result. Collection is off by default; enable and query it as needed, switch to the technical view or export support material. A successful substep or transfer never implies that the whole operation succeeded; unknown and incomplete outcomes are explicit.
- **Version and data upgrade**: Four platforms ship 0.9.4 with client SQLite schema 22. iOS is maintained at 0.9.4/9004000 but no iOS artifact is uploaded.

- **Frost-white theme readability**: Corrected primary-button backgrounds, text, hover, disabled and keyboard-focus states on Windows, Mac and Linux.
- **Android fresh-install startup**: Corrected the earlier 0.9.4 candidate that was incorrectly rejected during cold startup. The new formal APK passed a fresh arm64 Android SDK Emulator cold start and normal UI checks, retaining the original release signer. The earlier candidate's failed result is preserved.

## Feature areas

### Hosts and projects

Create, edit, search, favorite and group hosts. Use IPv4 or domains, separate ports, passwords or keys, and direct or proxy routes. Hosts / Projects.

### Terminals and remote desktops

Persistent and standard SSH terminals, desktop tabs and separate RDP windows, with platform-specific keyboard, clipboard and file interactions. SSH / RDP.

### Monitoring and history

CPU, memory, disk, load, network and disk IO, live charts, history ranges, top processes and collection settings. Available fields come from the target.

### File management

Browse, upload, download, create, rename, delete, edit text and verify hashes. Mobile transfers use system file pickers. Confirm the destination before acting.

### Forwarding and broadcasts

Windows, Mac and Linux provide SSH port forwarding, saved commands, batch execution and per-host results. Android has no forwarding or broadcast pages. Desktop features.

### Three AI scopes

Global can access all local scope histories; project is limited to that project and host to that host. Manage model accounts, bindings, permissions and confirmations. Conversations and memory remain local.

### System management

System information, processes and firewall tools. Windows Native and Linux Agent expose operations according to negotiated capabilities. Unknown capability is not treated as supported.

### Application center

Desktop management for supported Docker containers, images, systemd units and Windows services. Availability depends on the server.

### Grants and diagnostics

Desktop access-grant management. Diagnostic logging is off by default and can be enabled, queried and exported when needed. Platform entry points follow the actual UI.

## Accounts, self-hosting and data

### Optional account

Registration, login, device management and logout. Logging in neither decrypts data nor automatically starts a sync. Official Cloud by default.

### Official or self-hosted Cloud

Enter the HTTPS URL of your own deployment of the same Creation Cloud server. Login and sync state are isolated when switching sources. Operated by the third-party administrator.

### Explicit encrypted sync

Sync hosts, proxy profiles and model-account settings. Passwords, private keys, RDP credentials and model keys are encrypted by the client before upload. AI conversations and long-term memory are excluded.

### Model accounts and bindings

Manage model accounts, custom endpoints, bindings and context settings; use tools within the chosen scope and permission level. Use your own model service account.

### Data protection

The Cloud data-protection password and local device key have separate roles. Changing Cloud does not move local hosts or chats automatically. Confirm the target Cloud before syncing.

### Manual download and update source

0.9.4 must be downloaded manually. With a self-hosted Cloud, later checks use that source; its administrator uploads official downloads and original signatures. No automatic update to this release.

## Platform differences

Windows, Mac and Linux expose port forwarding, saved commands, broadcasts, separate RDP windows and desktop access-grant management. Android has no forwarding or broadcast pages. Windows server features require the matching components and capability handshake; ordinary OpenSSH does not imply full Agent functionality.

Mac is M-series only, not notarized and has no automatic updater. iOS is not published. Recovery evidence is environment-specific, not a guarantee for every outage, reboot or server configuration.

## Embedded Windows Setup ZIP

Windows, Mac, Linux and Android provide offline export from the DVC management form; Android also offers system sharing. RDP-only mode hides the installer card. The ZIP carries no host or credential and the client never executes its EXE.


## Verified in this release

- All ten formal artifacts have names, byte counts and SHA256 values bound to their platform receipts, totaling 580,358,528 bytes. Windows has current Release and package gates on both architectures; Portable contains only `C-SSH/C-SSH.exe`. Installation lifecycle and ordinary user-profile startup were not repeated in this packaging round.
- Mac is the ordinary arm64 Release/DMG with checked ad-hoc signing and no notarization. Same-source Mac business evidence is reused without opening the user's normal profile. Android checks cover 0.9.4/9004000, the original signer, 16KB alignment, a fresh SDK Emulator cold start and a normal operation-log row.
- Both Linux architectures passed deb/AppImage package checks and isolated minimal startup; AppImage used its extracted entry point. This round adds no FUSE-mount, upgrade, other-distribution or physical-machine claim.
- Import, onboarding and automatic Skills reuse the October 2–3 native evidence: Mac arm64; Windows ARM64 and x64 within an ARM VM; Linux ARM64, with x64 native differences checked separately; Android SDK Emulator and iPhone Simulator. SSH/Windows management and real-provider learning/cold-restart reuse retain their individual scopes. A complete matrix was not rerun on every architecture.
- Operation-log evidence includes same-source Mac business operations and a normal Android row. Other platforms have the current shared implementation, frontend/package checks and their minimal-startup evidence; this does not claim every business-log path was rerun on every platform.
- No claim covers physical Android/iPhone devices, every input method, every Linux distribution/Wayland, every recovery scenario or untested features in this round.
