# C-SSH 0.9.6 Revision 2 Feature Guide

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

- Ten artifacts total 583,805,859 bytes. Sizes and SHA256 come from current artifact and transfer receipts. Unchanged Mac r26/Windows r11 SSH/SFTP, monitor switching, navigation and UI-R57 deletion evidence is reused within its original scope.
- Mac arm64 reuses the first candidate’s actual 25→26 migration evidence with unchanged Store inputs. Final revision 2 starts under the ordinary identity on data already migrated to schema 26 and passes the existing Cloud page, window controls and close. A real-model text follow-up retained prior information; stopping occurred before provider context with tool=0, so incomplete tool rounds were not tested against a real model. Mac remains ad-hoc signed and unnotarized.
- Windows 11 ARM VM: native ARM64 and emulated x64. Release binaries passed 22→26 preservation, workbench and same-conversation stop/follow-up with an HTTP fixture. A new ordinary-user installer-only lane passed original 0.9.6 same-version replacement, actual EXE identities, data and registration preservation. All four updater epoch3 signatures were verified; no Authenticode. New-identity Session0 close/exit_app natural shutdown failures remain recorded; ARM64 raw separately passed Session1 close/destroyed. This is not a full formal-runtime pass.
- The signed arm64 APK passed same-signer/same build number 9006000 replacement, 22→26 preservation of 51 Required tables and preferences/wrapping data, plus fresh workbench/system appearance on a real Android SDK Emulator. AI entry only; no physical phone or general CleanShutdown validation.
- Both Linux architectures passed 22→26 preservation of 57 tables and 12 old rows, followed by direct FUSE AppImage cold startup after deb installation and real business views in Ubuntu VMs. Native Linux AI follow-up was not tested without independent model credentials.
- Incomplete tool rounds, unique complete results and exact coverage watermarks passed 3 shared integration tests / 4 real local HTTP scenarios; Windows native validation used a controlled HTTP fixture. This does not imply real-model or remote-tool validation on every platform.
- iPhone Simulator version/entry, fresh workbench/system and the two notification categories were exercised; no public upload. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.
