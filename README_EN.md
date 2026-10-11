# C-SSH 0.9.6 — Revision 2

[中文](README.md) · [Website](https://c-ssh.com) · [Installation](INSTALL_EN.md)

A server-operations client for Windows, Apple Silicon Mac, Android and Linux. Local SSH/Agent operations do not require Cloud login.

Announcements and manual downloads only: no upgrade reminder or forced update. The product version remains 0.9.6; revision 2 uses the new tag `v0.9.6-r2`. Ten assets cover four platforms; iOS is not uploaded. All earlier releases and original 0.9.6 assets remain intact. Android/iOS build number is 9006000; wire and signing identities are unchanged.

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,898,408 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,155,548 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,037,776 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,842,326 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,801,567 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,881,210 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,255,362 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,774,840 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,403,870 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,754,952 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r2/C-SSH_0.9.6_aarch64.AppImage) |

## Added

- Fresh installations and data without an explicit interface choice default to workbench and system appearance; saved interface and appearance choices are retained.
- A separate revision-2 download record preserves original 0.9.6 and historical downloads.

## Fixed

- Fixes the Mac initialization abort. The validated legacy schema 25 data moves forward to 26 and the existing Cloud session page loads normally.
- Fixes the Mac top-right maximize/restore, minimize and close controls.
- The notification center shows announcements and version messages only, removing visible account-security entries and their unread indicators.
- Follow-up in the same interrupted AI conversation retains recorded original text, assistant explanations and tool records with complete, unique known results. Missing results are not invented, and incomplete coverage is not marked as covered.
- SQLite moves forward to 26. Known legacy 23–25 data is continued only after strict shape and trust-preservation checks; plugin execution is not restored.

## Capabilities

| Area | Features |
|---|---|
| Hosts and projects | Create, edit, search, favorite, group; IPv4/domains, separate ports, passwords/keys and explicit proxy routes |
| SSH and Agent | Choose standard SSH or Agent per host, with shared identity, capability and error handling |
| Terminals | Linux tmux, Windows Terminal2/psmux and standard SSH; desktop tabs and mobile input/session recovery |
| Windows RDP | Remote desktops and DVC management within RDP, with certificate/account verification and a real-logon gate |
| Monitoring | CPU, memory, disk, load, network/disk IO, live/history charts, top processes and collection settings |
| Files | Browse, upload, download, create, rename, delete, edit text and verify hashes; native mobile file pickers |
| AI | Global/project/host scope, model accounts/bindings, permissions, confirmations, stop, history and local memory |
| System and apps | System information, processes, firewall and services; desktop Docker/systemd/Windows-service tools depend on server capabilities |
| Desktop tools | Windows/Mac/Linux port forwarding, saved commands, broadcasts, per-host results and access grants |
| Proxies | Direct, saved SOCKS5/HTTP CONNECT and explicitly selected official routes; no silent direct fallback |
| Cloud | Optional login, devices, explicit encrypted host/proxy/model-account sync and self-hosted service selection |
| Preferences | Nine languages, appearance, collection and data protection; diagnostic logging is off by default |

[Detailed features and platform differences](FEATURES_EN.md) · [Self-hosted Cloud](SELF_HOSTING_EN.md) · [Screenshot gallery](screenshots/README_EN.md)

## Verified

- Ten artifacts total 583,805,859 bytes. Sizes and SHA256 come from current artifact and transfer receipts. Unchanged Mac r26/Windows r11 SSH/SFTP, monitor switching, navigation and UI-R57 deletion evidence is reused within its original scope.
- Mac arm64 reuses the first candidate’s actual 25→26 migration evidence with unchanged Store inputs. Final revision 2 starts under the ordinary identity on data already migrated to schema 26 and passes the existing Cloud page, window controls and close. A real-model text follow-up retained prior information; stopping occurred before provider context with tool=0, so incomplete tool rounds were not tested against a real model. Mac remains ad-hoc signed and unnotarized.
- Windows 11 ARM VM: native ARM64 and emulated x64. Release binaries passed 22→26 preservation, workbench and same-conversation stop/follow-up with an HTTP fixture. A new ordinary-user installer-only lane passed original 0.9.6 same-version replacement, actual EXE identities, data and registration preservation. All four updater epoch3 signatures were verified; no Authenticode. New-identity Session0 close/exit_app natural shutdown failures remain recorded; ARM64 raw separately passed Session1 close/destroyed. This is not a full formal-runtime pass.
- The signed arm64 APK passed same-signer/same build number 9006000 replacement, 22→26 preservation of 51 Required tables and preferences/wrapping data, plus fresh workbench/system appearance on a real Android SDK Emulator. AI entry only; no physical phone or general CleanShutdown validation.
- Both Linux architectures passed 22→26 preservation of 57 tables and 12 old rows, followed by direct FUSE AppImage cold startup after deb installation and real business views in Ubuntu VMs. Native Linux AI follow-up was not tested without independent model credentials.
- Incomplete tool rounds, unique complete results and exact coverage watermarks passed 3 shared integration tests / 4 real local HTTP scenarios; Windows native validation used a controlled HTTP fixture. This does not imply real-model or remote-tool validation on every platform.
- iPhone Simulator version/entry, fresh workbench/system and the two notification categories were exercised; no public upload. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `78e084620bf3d6b610de983986d8d6dfb13c38bdb557febdf8ddc8c594edbd8d` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `141ccd8f5f4c900c36ec7e6a71621d75b8f2680f7a6dfbf6d722453ead0e454e` |
| C-SSH_0.9.6_arm64-setup.exe | `a08b6ae653d1741c4d192aa8b13a151ca5b7906bc009b210964703f50352e46b` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `a44878b3726ea7317d2ae0de3443d8d8a4135235a23139f46483b1232e3d9b03` |
| C-SSH_0.9.6_macOS-arm64.dmg | `42e4154303c150d5f31b88d969eb75d3bdcb304c603e0275088b208668038756` |
| C-SSH_0.9.6_android-arm64.apk | `efacd601d0299ec61f122394e5be666b43fd731f9a8ed21ba3b45bfd8e60b88f` |
| C-SSH_0.9.6_amd64.deb | `604f8100e1397bb0ea9dc7a89c74efa4072dc59143da3356e4170cda31efcc37` |
| C-SSH_0.9.6_amd64.AppImage | `bd13eee859d5ac91af86232fcf3f9ed277eb9582a2b58d36bb4b9d82f5b94ea4` |
| C-SSH_0.9.6_arm64.deb | `6db75f6b7ff9dc3d7923524fd2b9de3ab3ea885bdc1281be16e22eb19728789c` |
| C-SSH_0.9.6_aarch64.AppImage | `65a92848bd5c7e34340d9462eeb95436300badbc3c399026e86a7982e4465762` |

## Data and source

This revision uses SQLite 26. Exit, preserve complete data and keep existing OS Keychain/keystore entries before replacing the program in place. Do not manually move, delete or export raw keys, clear data, or uninstall to work around startup/signing issues. Normal 22→26 and the validated known 25 data were exercised. Other 23–25 data is accepted only for strictly known shapes with trust preserved. Once data adopts 26, do not open it with original 0.9.6 schema 22 binaries. A rollback requires the matching older program and a complete pre-upgrade backup.

AI conversations, raw records, Skills and long-term memory remain local and are not Cloud-sync data. Selected model context is sent to the chosen provider. Client/shared-core/AI/Agent sources are not public; this repository contains docs, binaries and the whitelisted Cloud mirror.
