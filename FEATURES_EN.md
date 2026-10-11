# C-SSH 0.9.6 Feature Guide

[Home](README_EN.md) · [Installation](INSTALL_EN.md) · [Gallery](screenshots/README.md)

This release requires manual installation. The sections below describe available entry points and their platform or server-capability limits.

## How it works

### Four platforms

Windows x64/ARM64, Apple Silicon Mac arm64, Android arm64 and Linux x86_64/ARM64 share core capabilities with desktop and phone layouts. 0.9.6 · Manual installation.

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

## 0.9.6

- Classic and workbench interfaces coexist. Three desktop card sizes share one compact layout and retain the selected preference.
- Desktop workbench AI chat adds semantic Markdown, tool summaries, highlighted code with raw copying, reading-position handling, quotations and evidence-backed file or snippet diffs.
- Windows and Mac support browsing hosts and tool cards while an AI run stays bound to its original conversation. These two page adaptations are not claimed for Linux or mobile.

- Applicable file and monitor pages allow host switching during reads and reject stale results or authentication callbacks. Pending writes keep their guard across navigation.
- Deleting the selected host clears invalid read targets; the hidden monitor page no longer navigates to or automatically reads another host.
- Logs show safe setting values and reduce routine or successful autosave noise. Duplicate icons, card layout and idle animations in the hidden updater are corrected.
- Mac exit waits for saving. Linux Agent diagnostics use the shared DTO conversion and reject client-only preference fields without a wire change.

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

0.9.6 must be downloaded manually. With a self-hosted Cloud, later checks use that source; its administrator uploads official downloads and original signatures. No automatic update to this release.

## Platform differences

Windows, Mac and Linux expose port forwarding, saved commands, broadcasts, separate RDP windows and desktop access-grant management. Android has no forwarding or broadcast pages. Windows server features require the matching components and capability handshake; ordinary OpenSSH does not imply full Agent functionality.

Mac is M-series only, not notarized and has no automatic updater. iOS is not published. Recovery evidence is environment-specific, not a guarantee for every outage, reboot or server configuration.

## Embedded Windows Setup ZIP

Windows, Mac, Linux and Android provide offline export from the DVC management form; Android also offers system sharing. RDP-only mode hides the installer card. The ZIP carries no host or credential and the client never executes its EXE.


## Verified

- Ten artifacts total 583,386,396 bytes and are bound to the current build and package receipts. Existing Mac/Windows real-VM SSH/SFTP, monitor switching, navigation and deletion regressions are reused without repeating unchanged passing checks.
- Windows x64/ARM64 packages passed per-architecture package, install, 0.9.4→0.9.6 upgrade, uninstall and data-preservation checks in a Windows 11 ARM VM. x64 updater epoch3 signatures were cryptographically verified; these are not Authenticode signatures.
- Mac passed ordinary arm64 Release, ad-hoc strict-signature and DMG checks; it remains unnotarized. Android retained the release signer and passed actual Manifest/DEX 0.9.6/9006000, 16KiB alignment, and same-signer upgrade with settings and 21 protected tables preserved in an API36 arm64 emulator.
- Linux deb/AppImage packages passed per-architecture embedding and isolated startup, log entry and preference preservation on Ubuntu24.04 X11. AppImage used its extracted entry point; FUSE mounting and all remote workflows were not retested.
- iPhone received a Simulator build/identity check only and is not uploaded. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.
