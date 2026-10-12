# C-SSH 0.9.6 Revision 3 Feature Guide

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

## Revision 3

- Adding an Agent host checks an existing Agent for compatibility first. Compatible Agents are reused without uploading, reinstalling or restarting. Missing Agents are installed; confirmed older versions follow the safe upgrade path. Unknown probe outcomes or future versions do not overwrite an installation.
- Identity adoption, READY confirmation and credential persistence appear as separate real stages. A readiness failure is no longer reported as an initial SSH connection failure, and does not automatically replay installation.
- Deployment progress is scoped to the current invocation, host and operation; late events after completion or leaving the view are rejected.
- Windows and Mac workbench close buttons match the adjacent window controls in outline, background and size, while preserving danger hover and the existing exit action. Linux retains its normal system title bar.
- Classic and workbench search entry points on Windows, Mac and Linux default to a magnifying-glass icon. Clicking opens the existing search panel, and the existing shortcuts are retained. The terminal toolbar wraps in narrow windows to keep search and connection controls from overlapping.

## Revision 2

- Fresh installations and data without an explicit interface choice default to workbench and system appearance; saved interface and appearance choices are retained.
- A separate revision-2 download record preserves original 0.9.6 and historical downloads.
- Fixes the Mac initialization abort. The validated legacy schema 25 data moves forward to 26 and the existing Cloud session page loads normally.
- Fixes the Mac top-right maximize/restore, minimize and close controls.
- The notification center shows announcements and version messages only, removing visible account-security entries and their unread indicators.
- Follow-up in the same interrupted AI conversation retains recorded original text, assistant explanations and tool records with complete, unique known results. Missing results are not invented, and incomplete coverage is not marked as covered.
- SQLite moves forward to 26. Known legacy 23–25 data is continued only after strict shape and trust-preservation checks; plugin execution is not restored.

This revision uses SQLite 26. Exit, preserve complete data and keep existing OS Keychain/keystore entries before replacing the program in place. Do not manually move, delete or export raw keys, clear data, or uninstall to work around startup/signing issues. Normal 22→26 and the validated known 25 data were exercised. Other 23–25 data is accepted only for strictly known shapes with trust preserved. Once data adopts 26, do not open it with original 0.9.6 schema 22 binaries. A rollback requires the matching older program and a complete pre-upgrade backup.

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

- Direct frontend verification completed 170 checks across 38 layouts for classic/workbench search buttons, the existing panel, shortcuts and narrow windows on Windows, Mac and Linux. All three frontend builds passed.
- All ten release packages have matching source and package receipts. The nine refreshed desktop packages contain the final search interface; all four Windows packages have new actual installed identities and valid signatures. Mac is arm64, ad-hoc signed and not notarized.
- Agent acceptance covers three Linux nodes through Mac, native ARM64 and compatible x64 execution on Windows 11 ARM64, and both Linux VM architectures. The search-only change reuses the unchanged Rust-flow evidence; it does not imply a new full-function test of every package or physical Windows x64 acceptance.
- The Android arm64 release APK retains the existing signer. Its normal UI completed existing-Agent reuse, readiness, credential persistence and monitor loading on the SDK Emulator. This does not imply physical-device or ordinary host-deletion acceptance. iOS source and frontend are synchronized only and are not uploaded.
