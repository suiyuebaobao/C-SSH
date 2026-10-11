[中文](CHANGELOG.md) | **English**

# Changelog


## 0.9.6 — Interfaces, AI chat and host switching

2026-10-11

Announcement and manual downloads only; no upgrade reminders or forced updates. iOS is excluded and old versions remain available. SQLite stays at schema22, Android/iOS build number is 9006000, and wire versions and signing identities are unchanged.

### Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,881,563 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,128,433 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,030,473 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,816,423 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,668,120 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,849,374 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,197,684 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,733,880 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,350,070 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,730,376 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6/C-SSH_0.9.6_aarch64.AppImage) |

### Added

- Classic and workbench interfaces coexist. Three desktop card sizes share one compact layout and retain the selected preference.
- Desktop workbench AI chat adds semantic Markdown, tool summaries, highlighted code with raw copying, reading-position handling, quotations and evidence-backed file or snippet diffs.
- Windows and Mac support browsing hosts and tool cards while an AI run stays bound to its original conversation. These two page adaptations are not claimed for Linux or mobile.

### Fixed

- Applicable file and monitor pages allow host switching during reads and reject stale results or authentication callbacks. Pending writes keep their guard across navigation.
- Deleting the selected host clears invalid read targets; the hidden monitor page no longer navigates to or automatically reads another host.
- Logs show safe setting values and reduce routine or successful autosave noise. Duplicate icons, card layout and idle animations in the hidden updater are corrected.
- Mac exit waits for saving. Linux Agent diagnostics use the shared DTO conversion and reject client-only preference fields without a wire change.

### Verified

- Ten artifacts total 583,386,396 bytes and are bound to the current build and package receipts. Existing Mac/Windows real-VM SSH/SFTP, monitor switching, navigation and deletion regressions are reused without repeating unchanged passing checks.
- Windows x64/ARM64 packages passed per-architecture package, install, 0.9.4→0.9.6 upgrade, uninstall and data-preservation checks in a Windows 11 ARM VM. x64 updater epoch3 signatures were cryptographically verified; these are not Authenticode signatures.
- Mac passed ordinary arm64 Release, ad-hoc strict-signature and DMG checks; it remains unnotarized. Android retained the release signer and passed actual Manifest/DEX 0.9.6/9006000, 16KiB alignment, and same-signer upgrade with settings and 21 protected tables preserved in an API36 arm64 emulator.
- Linux deb/AppImage packages passed per-architecture embedding and isolated startup, log entry and preference preservation on Ubuntu24.04 X11. AppImage used its extracted entry point; FUSE mounting and all remote workflows were not retested.
- iPhone received a Simulator build/identity check only and is not uploaded. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.

### SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `0d87b5a42e4b7196877fd726170aead75d7b7714c9393efc4c1a9770e95d884b` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `ab15668c497e2009fcfa8a5efe2696546b866577a1855afaf5f7cbd44ce9ca20` |
| C-SSH_0.9.6_arm64-setup.exe | `538993bfa604d8e00af58b28273c8f23c56efe85431633dafbb93cd5279ac0b8` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `fa1b4287038c435083c4c13e12eb4194180fbe9f05ea40b24b26b3b6b4db38b8` |
| C-SSH_0.9.6_macOS-arm64.dmg | `c0fd388fcc7b65693ebe5c71c59efaa45ea1798b72791a61e67480961ec21dd9` |
| C-SSH_0.9.6_android-arm64.apk | `423be8b7227cfb026b87413422bc822daeff81527bb1f3588482b81f3a98aac7` |
| C-SSH_0.9.6_amd64.deb | `608baecb6e9106060e2aca7f5af424c6c7f2629136bd27a789c49f473f529980` |
| C-SSH_0.9.6_amd64.AppImage | `e0bd964461331bb790f41f5e6683ee3bfe4e6774f5698e682d6cff2bbb2d8a13` |
| C-SSH_0.9.6_arm64.deb | `fa17a74b7d21e4bb895ac1296c4c96c282e4e58dbd296eadc60c25a078b7677d` |
| C-SSH_0.9.6_aarch64.AppImage | `08a00b83da19789ff30ea94cc7aa8ddb87a0dfba59e1061ef661f914512eab79` |

## 0.9.4 — Import, onboarding, automatic operations Skills and log rows

2026-10-04

0.9.4 uses announcements and manual downloads only, with no upgrade prompt or forced update. Mac is Apple Silicon arm64 only, minimum target macOS13, ad-hoc signed and not notarized. Windows installers have no Authenticode signature. Android retains the release signer used since 0.9.0; earlier differently signed builds cannot be replaced directly. AI content is excluded from Cloud sync; read the installation and data-preservation guide first.

### Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.4_android-arm64.apk | 39,707,218 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_android-arm64.apk) |
| C-SSH_0.9.4_aarch64.AppImage | 116,402,696 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_aarch64.AppImage) |
| C-SSH_0.9.4_amd64.AppImage | 117,410,296 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_amd64.AppImage) |
| C-SSH_0.9.4_amd64.deb | 43,882,432 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_amd64.deb) |
| C-SSH_0.9.4_arm64.deb | 43,049,976 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_arm64.deb) |
| C-SSH_0.9.4_macOS-arm64.dmg | 42,342,556 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_macOS-arm64.dmg) |
| C-SSH_0.9.4_arm64-setup.exe | 45,720,976 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_arm64-setup.exe) |
| C-SSH_0.9.4_portable-Windows-arm64.zip | 41,482,505 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_portable-Windows-arm64.zip) |
| C-SSH_0.9.4_portable-Windows-x64.zip | 42,794,746 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_portable-Windows-x64.zip) |
| C-SSH_0.9.4_x64-setup.exe | 47,565,127 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.4/C-SSH_0.9.4_x64-setup.exe) |

### Added

- **Bulk host import**: Import CSV or OpenSSH configuration, preview and select entries, then create pending connections. A host becomes active only after credentials are supplied and real authentication succeeds. Previewing is not connecting; passwords and private keys are not stored in the plaintext pending queue.
- **Skippable first-use guide**: Discover connection, import and operations entry points, or skip directly to the workspace. The choice survives a cold restart and the guide can be reopened in Settings. Upgrades with existing data do not force onboarding.
- **Automatic AI operations Skills**: Experience is learned only after tools execute and acceptance succeeds. Later tasks reuse it only when the current environment matches. No manual save or adoption step is required; the normal UI shows a single light notice when past experience is referenced. Existing permissions and confirmations still apply. Skills, AI conversations, raw records and long-term memory remain local and are excluded from Cloud sync.
- **Readable operation-log rows**: One business operation per row, with time, level, local host name and a trustworthy result. Collection is off by default; enable and query it as needed, switch to the technical view or export support material. A successful substep or transfer never implies that the whole operation succeeded; unknown and incomplete outcomes are explicit.
- **Version and data upgrade**: Four platforms ship 0.9.4 with client SQLite schema 22. iOS is maintained at 0.9.4/9004000 but no iOS artifact is uploaded.

### Fixed

- **Frost-white theme readability**: Corrected primary-button backgrounds, text, hover, disabled and keyboard-focus states on Windows, Mac and Linux.
- **Android fresh-install startup**: Corrected the earlier 0.9.4 candidate that was incorrectly rejected during cold startup. The new formal APK passed a fresh arm64 Android SDK Emulator cold start and normal UI checks, retaining the original release signer. The earlier candidate's failed result is preserved.

### Verified

- All ten formal artifacts have names, byte counts and SHA256 values bound to their platform receipts, totaling 580,358,528 bytes. Windows has current Release and package gates on both architectures; Portable contains only `C-SSH/C-SSH.exe`. Installation lifecycle and ordinary user-profile startup were not repeated in this packaging round.
- Mac is the ordinary arm64 Release/DMG with checked ad-hoc signing and no notarization. Same-source Mac business evidence is reused without opening the user's normal profile. Android checks cover 0.9.4/9004000, the original signer, 16KB alignment, a fresh SDK Emulator cold start and a normal operation-log row.
- Both Linux architectures passed deb/AppImage package checks and isolated minimal startup; AppImage used its extracted entry point. This round adds no FUSE-mount, upgrade, other-distribution or physical-machine claim.
- Import, onboarding and automatic Skills reuse the October 2–3 native evidence: Mac arm64; Windows ARM64 and x64 within an ARM VM; Linux ARM64, with x64 native differences checked separately; Android SDK Emulator and iPhone Simulator. SSH/Windows management and real-provider learning/cold-restart reuse retain their individual scopes. A complete matrix was not rerun on every architecture.
- Operation-log evidence includes same-source Mac business operations and a normal Android row. Other platforms have the current shared implementation, frontend/package checks and their minimal-startup evidence; this does not claim every business-log path was rerun on every platform.
- No claim covers physical Android/iPhone devices, every input method, every Linux distribution/Wayland, every recovery scenario or untested features in this round.

### SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.4_android-arm64.apk | `8ee9649e69ed73660f1bdc42c6c56de9964de64f1ecfe9d72bb52d2ec26fd76c` |
| C-SSH_0.9.4_aarch64.AppImage | `a30eeadb2052a3a7702b4ac4de20ab0544ccf6a1552cd8b4fcfdccab3a744402` |
| C-SSH_0.9.4_amd64.AppImage | `6e536a4c070539d7ff1195ebfb92b6654402450fdf5d82d5a86de3a47ff91ad5` |
| C-SSH_0.9.4_amd64.deb | `8bd0330364baf7fe5f5fd6bc71c3c64cb54c4351184dc2a578ccbaf58dd98edc` |
| C-SSH_0.9.4_arm64.deb | `fb12a27ae9fc2e93729aeff6ea826e3b45cc01ca5f168f2283ccc4a8d752d281` |
| C-SSH_0.9.4_macOS-arm64.dmg | `b6c472e64e98965f9859e65cc738de2f2895dbebfb1d5a080ba4c9f553cfe401` |
| C-SSH_0.9.4_arm64-setup.exe | `463ae846ea6d2129f9fd1e2a25055f02a78a9dcffb9e80abf0672e9477ec8bb6` |
| C-SSH_0.9.4_portable-Windows-arm64.zip | `1dd2abe077602c94fb976b849648e5432f984ad482de1da91828040520321627` |
| C-SSH_0.9.4_portable-Windows-x64.zip | `32689bfaf190bab66ae5197d12ece09270358bc139f40c1a30ced15563f72841` |
| C-SSH_0.9.4_x64-setup.exe | `280b1144366a283f20fd75f224c800239db072d5dd1c0fdd162b71c4ea21ff54` |

## 0.9.3 — Terminal keyboard and form fixes

2026-10-02.

- **Terminal keyboard handling:** Android adaptive terminals resize to the visible area, with corrected cursor and input-anchor positioning. Shortcut keys stay above the keyboard and leave room for the input line when expanded.
- **Accessible form closing:** add/edit host forms have a top-right close button that remains available while scrolling. Clicking outside the form preserves entered values.
- **Mobile terminal font:** Android/iOS default to size 6, preserving saved custom sizes.
- **Version alignment:** iOS is maintained at 0.9.3, without an iOS package upload.
- **Manual installation:** download and install this release yourself. Announcement only; no upgrade prompts or forced updates.

## 0.9.2 — Linux and Windows ARM64

2026-10-02. Four platforms, ten packages; no iOS upload.

- **Linux desktop client:** x86_64 and ARM64 deb/AppImage packages, sharing desktop hosts, SSH/Agent, persistent terminals, RDP/DVC, monitoring, files, AI, proxies, port forwarding and broadcasts.
- **Native Windows ARM64:** NSIS and Portable packages, validated under an ordinary user on Windows 11 ARM64. Remote Windows management components remain x64.
- **Older Windows terminals:** the bundled psmux adds a WinPTY backend for Server 2016 without ConPTY. Modern Windows keeps ConPTY. Management components use the static CRT to reduce runtime dependencies on clean systems.
- **Reliability fixes:** host edit/delete waits, Windows domain connections, file-overwrite permission cleanup and Linux legacy-database migration.
- **Cleaner announcements:** sync-operation messages are removed from the announcement dialog; Cloud sync remains available.
- **Manual installation:** announced through notices, without upgrade prompts or forced updates.

## 0.9.1 — Address detection and embedded Windows Setup export

2026-09-30. Windows x64 NSIS/Portable, Apple Silicon Mac arm64 DMG and Android arm64 APK. Notices and manual downloads only; no upgrade prompt or forced update. No iOS release; the Linux client stays frozen.

Added:
- Enter an IPv4 address or domain directly with automatic detection and a separate port; the address-type selector is removed. Existing IPv6 hosts remain compatible.
- Embedded Windows management Setup ZIP with offline export in DVC, missing-component and maintenance flows; Android also offers system sharing. RDP-only hides the card. No host/credential is exported and the client never runs the EXE.
- Component version 0.9.1; ZIP size 13,308,604 bytes. Setup and Agent/psmux maintenance keep the same verified source.

Fixed and verified:
- Existing Setup conflicts remain fail-closed. Portable instructions require backing up the old EXE/Setup while preserving data. Windows ordinary-user manual upgrades and data retention passed.
- Mac arm64/version/ad-hoc signature/DMG mounting and unique embedded-resource checks passed. No notarization, automatic-update or minimum-OS/every-model claim is made.
- Android keeps the 0.9.0 signer; replacement, startup, retained settings and system export/share are verified in Android SDK Emulator. This does not claim a physical phone or recipient delivery.
- Earlier Android signer limitations remain. Uninstalling clears local AI history/memory; preserve data first. 0.9.0 does not require uninstalling.

[Installation](INSTALL_EN.md) · [New form samples](screenshots/v0.9.1/README_EN.md) · [v0.9.1 downloads](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.1)


[Historical changelog 1](CHANGELOG_HISTORY_1_EN.md)

[Historical changelog 2](CHANGELOG_HISTORY_2_EN.md)
