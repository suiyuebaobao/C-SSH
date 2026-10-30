[简体中文](CHANGELOG.md) | **English**

# Changelog

## 0.9.8 — Host connections, Cloud access and stability

2026-10-30

This release updates Windows, macOS, Linux and Android, together with the C-SSH Agents. It focuses on host connection errors, self-hosted Cloud access, Android session recovery, host deletion and diagnostic overhead.

### 1. Clearer host connection errors

When adding a host, connection failures now appear in a dialog that matches the C-SSH interface. Messages distinguish address resolution, network connectivity, SSH handshake, authentication and proxy authentication failures so that users can identify which part of the connection needs attention.

Closing the dialog preserves the entered connection details. An authentication rejection is not automatically labeled an incorrect password: the username, key and server authentication policy may also be responsible.

### 2. Separate hosts at the same IP address

Fixes hosts with the same address but different SSH ports being treated as duplicates during creation or synchronization. Connection endpoints are identified by both address and SSH port; their host records, credentials, trust records and Cloud mappings remain separate.

Existing host identities are preserved. Adding a different port does not require deleting the original host or replacing its credentials.

### 3. Automatic protocol detection for self-hosted Cloud

In Settings → Cloud service → Self-hosted cloud, enter an IP address, domain or address with a port. Test the connection to see the detected protocol, then save and switch. A protocol prefix is optional for a new address.

Detection tries HTTPS first. An explicitly configured, trusted private-network deployment may use HTTP without importing certificates on every client. The interface displays the selected protocol and warns that HTTP traffic is not encrypted.

Explicit or previously saved HTTPS addresses, certificate verification failures and public-network addresses are never silently downgraded to HTTP. Detection sends no existing account credentials or session cookies. The official Cloud continues to use HTTPS. Self-hosted servers must use a compatible server version and explicitly enable the private-network HTTP mode.

### 4. Android AI session recovery after interruptions

Improves recovery when the phone is locked, the network disconnects, connectivity returns or the app returns to the foreground. Addresses blocked follow-up messages, unrecognized task errors and save state that did not recover promptly.

Recovery reconciles the original task with locally saved state, preserves content already received and allows the conversation to continue. It does not automatically replay model requests or remote modification commands. Related save acknowledgements are handled in the correct order to reduce discrepancies between stored conversation content and the displayed state.

### 5. Less unnecessary waiting when deleting hosts

Fixes host deletion waiting on a resource lock already held by the same cleanup operation when the host has server-browser environments. This could leave the interface showing “Deleting” for an unnecessarily long time.

The deletion path now releases the relevant resources before completing browser-environment cleanup. The remote Agent uninstall attempt remains time limited; a failure does not indefinitely block deletion of the local host record.

### 6. Lower Agent diagnostic overhead

When diagnostics are disabled, recording work is skipped earlier. The Linux Agent summarizes normal, short monitoring operations while retaining the necessary details for errors, slow operations and other activity that needs diagnosis.

Log maintenance uses bounded periodic work instead of heavier cleanup on request paths. The Windows Agent receives the corresponding maintenance scheduling changes. These changes target diagnostic recording and maintenance costs; they do not reduce monitoring frequency or change logging settings for other server programs.

### 7. Cloud administrator account deletion

Fixes deletion being blocked by device references when an account contains hosts, AI configurations, proxy profiles or synchronization history.

The account and its dependent synchronization records are handled in one database transaction. Failures roll back completely, and other accounts remain unchanged. Administrator protections, global history and audit requirements remain in place. The official Cloud has received this backend fix; self-hosted deployments need the updated server implementation.

### Versions and installation

- Windows, macOS, Linux and Android clients, and C-SSH Agents: 0.9.8.
- This release announces the update and directs users to download and install it manually. It does not force updates or download and install them automatically.
- Keep the existing installation identity and local data. Do not uninstall, clear app data or remove keys as an update step.
- macOS remains available for Apple Silicon. The app is not notarized with an Apple Developer ID; this installation limitation is unchanged.
- iOS development and releases remain paused. The mobile server-browser entry remains hidden.

## 2026-10-30 — Cloud account deletion fix

Fixes account deletion being blocked by device references when the account has hosts, AI configurations, proxy profiles, or synchronization history. Dependencies and the account are deleted in one transaction; failures roll back completely and other accounts remain unchanged.

Verified account isolation, rollback, and concurrent writes in an isolated database, plus real deletion API calls on the private deployment. The same fix is deployed to the official Cloud. This server maintenance update does not change client versions, HTTPS settings, database schema, or existing installers.

## 0.9.7 — Kylin, AI, Server Browser, monitoring, and sync improvements

2026-10-28

This release improves server connectivity, the AI assistant, Server Browser, monitoring collection, and cloud synchronization across Windows, macOS, Linux, and Android.

### 1. Kylin and ARM server compatibility

Improves compatibility with Kylin Linux Advanced Server V10 ARM64, including its 64KiB memory-page environment. The changes cover Agent connectivity, monitoring, file operations, persistent terminals, and command execution.

Fixes initial Agent channel failures in specific SSH configurations and compatibility checks that failed when the relevant logs were large. The ARM Agent is updated to the current version.

### 2. Agent connectivity across devices

Updates the Linux x64, Linux ARM64, and Windows Server Agents.

Improves authorization and Agent reuse when multiple computers or phones connect to the same server, including fixes for some public-key authorization conflicts. Improves existing-Agent detection, capability checks, and update handling, and fixes deployment status not refreshing promptly after cloud synchronization.

### 3. Desktop Server Browser

Windows, macOS, and Linux can use local Chrome through the server's Agent.

Each server can have multiple browser environments, with separate cookies, login sessions, browsing data, and fixed fingerprint seeds. Settings include the displayed operating system, timezone, language, location, microphone, and camera permissions.

Improves launch speed and first-launch reliability, fixes unexpected browser closure on some websites and verification pages, improves recovery after an abnormal exit, and adds a reminder when Chrome is not installed.

### 4. AI concurrency and query efficiency

Adds configurable AI task concurrency and queuing. Independent read-only tasks can run concurrently across servers or on the same server. Workers inherit the main Agent's context-window configuration for the current run.

Improves server-status checks by reducing repeated file reads, duplicate queries, and unnecessary tool steps. Dependent operations and changes still run in the required order.

### 5. Live AI responses, progress, and conversation recovery

Responses appear as the model returns them, with timely worker progress, results, and errors.

The main conversation shows compact progress and result summaries. Full tool activity is available under View details.

Fixes missing interjection messages, sending failures after pausing, and some conversation recovery failures after interruption or abnormal exit. Improves saved-history conflict detection and fixes some Mac AI crashes.

### 6. AI context, long conversations, and usage

Context preparation follows the model's actual window instead of a fixed 256K threshold. Tool results are no longer uniformly cut to a fixed character count; large results can be read in pages as needed.

Long conversations use chunked storage and loading, addressing the previous 256MiB whole-conversation limit and reducing waits when reopening long chats or preparing context.

Adds actual usage reporting for **cumulative tokens used, context usage, and cache hit rate**, normalizes usage fields across model APIs, and includes worker usage. The last confirmed values remain visible during execution. Missing cache data is not treated as a zero hit rate.

Cumulative tokens used counts usage across requests in the conversation. Context usage measures the main assistant's input for one request; the two are explained separately.

### 7. Monitoring and transfer efficiency

Raises the configurable collection concurrency limit to **1,000**, improves queuing and scheduling, and reduces delays during the first refresh and the impact of slow hosts on other hosts.

Optimizes local database access and reduces duplicate operations. Larger monitoring-history responses use selective compression to reduce transfer volume, while small responses remain lightweight.

### 8. Cloud synchronization and password verification

After data-protection password verification, the running application reuses that verification when changing pages, checking changes, uploading, and downloading.

Adds Select all and a default cloud choice, with clearer sync-item layout and prompts.

Fixes download confirmation failures after the first Android login. Deleted AI accounts that are also absent locally no longer repeatedly appear as pending sync items.

### 9. Android interface and interaction fixes

Brings shared AI, context, memory, and usage changes to Android, with a usage summary at the bottom and a compact three-value detail panel.

Fixes excessive bottom space and keyboard/safe-area spacing. Fixes opening the wrong host's terminal from a host action card and being unable to change hosts while a connection is pending.

Fixes AI history selection, restored history interfering with the active model settings, and usage not refreshing after the model context window changes.

### 10. Desktop interface consistency

Aligns shared settings and AI interfaces across Windows, macOS, and Linux. Settings are organized into four category cards with direct access to individual settings.

Confirmation dialogs use C-SSH's own style. Improves Mac window controls and corners, and text contrast in some dark dialogs.

**Updating:** This release provides an update notice. Download the new client from the official website and install it yourself; there is no forced update or automatic download and installation.

## 0.9.6 — Revision 3: Agent reuse and readiness progress

2026-10-12


Announcements and manual downloads only, with no upgrade reminder or forced update. Version0.9.6, SQLite26 and build9006000 are unchanged. Revision3 uses the independent tag `v0.9.6-r3`; ten packages cover four platforms, iOS is not uploaded, and original0.9.6, revision2 and historical downloads remain available.

The product version remains 0.9.6 with SQLite 26. Update in place with the matching platform package while preserving existing data and OS Keychain/keystore entries. Mac is ad-hoc signed and not notarized. Announcement and manual download only, with no upgrade reminders or forced updates.

### Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,912,113 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,180,565 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,045,545 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,861,101 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,822,796 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,899,610 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,271,530 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,750,264 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,422,970 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,779,528 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_aarch64.AppImage) |

### Added

- A separate revision-3 download record preserves original 0.9.6, revision 2 and historical downloads.

### Fixed

- Adding an Agent host checks an existing Agent for compatibility first. Compatible Agents are reused without uploading, reinstalling or restarting. Missing Agents are installed; confirmed older versions follow the safe upgrade path. Unknown probe outcomes or future versions do not overwrite an installation.
- Identity adoption, READY confirmation and credential persistence appear as separate real stages. A readiness failure is no longer reported as an initial SSH connection failure, and does not automatically replay installation.
- Deployment progress is scoped to the current invocation, host and operation; late events after completion or leaving the view are rejected.
- Windows and Mac workbench close buttons match the adjacent window controls in outline, background and size, while preserving danger hover and the existing exit action. Linux retains its normal system title bar.
- Classic and workbench search entry points on Windows, Mac and Linux default to a magnifying-glass icon. Clicking opens the existing search panel, and the existing shortcuts are retained. The terminal toolbar wraps in narrow windows to keep search and connection controls from overlapping.

### Verified

- Direct frontend verification completed 170 checks across 38 layouts for classic/workbench search buttons, the existing panel, shortcuts and narrow windows on Windows, Mac and Linux. All three frontend builds passed.
- All ten release packages have matching source and package receipts. The nine refreshed desktop packages contain the final search interface; all four Windows packages have new actual installed identities and valid signatures. Mac is arm64, ad-hoc signed and not notarized.
- Agent acceptance covers three Linux nodes through Mac, native ARM64 and compatible x64 execution on Windows 11 ARM64, and both Linux VM architectures. The search-only change reuses the unchanged Rust-flow evidence; it does not imply a new full-function test of every package or physical Windows x64 acceptance.
- The Android arm64 release APK retains the existing signer. Its normal UI completed existing-Agent reuse, readiness, credential persistence and monitor loading on the SDK Emulator. This does not imply physical-device or ordinary host-deletion acceptance. iOS source and frontend are synchronized only and are not uploaded.

### SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `68c108d74ce246513b87d66a10fb7bd88c2bc729d150394f09b1a1cfdad41de8` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `67e46eae24524e4478c014aaea660e8ad938dafe5269b9059cc8ed73327c95f2` |
| C-SSH_0.9.6_arm64-setup.exe | `b3287cbcb86daac82963015d5abedb2786541c2c1f3e60bac6b2c125831eff0c` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `ffee1c505cbd0883a20babeb51d9cada3173510c48f3247b897b26bbd85f13f5` |
| C-SSH_0.9.6_macOS-arm64.dmg | `28d6962f39104739030dc192d513ae00f088e1bf32930a979ec887d5c05bf000` |
| C-SSH_0.9.6_android-arm64.apk | `a0245a283b45ed9f1a5b4c35091ff419e1cc9bdf3054d4cef3f2488357c5d239` |
| C-SSH_0.9.6_amd64.deb | `a482d1d7c9e452533c4980790b02f88a6d71bf350bd0a231e47216eeaff76a22` |
| C-SSH_0.9.6_amd64.AppImage | `60ef49c4a3e9ac2abada6ad5f2baa0a6a43cb4a513ed5cc3f472fd8b2e42f218` |
| C-SSH_0.9.6_arm64.deb | `117b9915c92af9eecbac46c9c51b37d5414a751fcfe85339f5f32e6be30c4794` |
| C-SSH_0.9.6_aarch64.AppImage | `6528806c24497e6e696a1a038c0025819295c5cc68b9db959d1376d5f9abff14` |

## 0.9.6 — Revision 2: startup, migration and interrupted context

2026-10-11

Announcements and manual downloads only: no upgrade reminder or forced update. The product version remains 0.9.6; revision 2 uses the new tag `v0.9.6-r2`. Ten assets cover four platforms; iOS is not uploaded. All earlier releases and original 0.9.6 assets remain intact. Android/iOS build number is 9006000; wire and signing identities are unchanged.

This revision uses SQLite 26. Exit, preserve complete data and keep existing OS Keychain/keystore entries before replacing the program in place. Do not manually move, delete or export raw keys, clear data, or uninstall to work around startup/signing issues. Normal 22→26 and the validated known 25 data were exercised. Other 23–25 data is accepted only for strictly known shapes with trust preserved. Once data adopts 26, do not open it with original 0.9.6 schema 22 binaries. A rollback requires the matching older program and a complete pre-upgrade backup.

### Downloads

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

### Added

- Fresh installations and data without an explicit interface choice default to workbench and system appearance; saved interface and appearance choices are retained.
- A separate revision-2 download record preserves original 0.9.6 and historical downloads.

### Fixed

- Fixes the Mac initialization abort. The validated legacy schema 25 data moves forward to 26 and the existing Cloud session page loads normally.
- Fixes the Mac top-right maximize/restore, minimize and close controls.
- The notification center shows announcements and version messages only, removing visible account-security entries and their unread indicators.
- Follow-up in the same interrupted AI conversation retains recorded original text, assistant explanations and tool records with complete, unique known results. Missing results are not invented, and incomplete coverage is not marked as covered.
- SQLite moves forward to 26. Known legacy 23–25 data is continued only after strict shape and trust-preservation checks; plugin execution is not restored.

### Verified

- Ten artifacts total 583,805,859 bytes. Sizes and SHA256 come from current artifact and transfer receipts. Unchanged Mac r26/Windows r11 SSH/SFTP, monitor switching, navigation and UI-R57 deletion evidence is reused within its original scope.
- Mac arm64 reuses the first candidate’s actual 25→26 migration evidence with unchanged Store inputs. Final revision 2 starts under the ordinary identity on data already migrated to schema 26 and passes the existing Cloud page, window controls and close. A real-model text follow-up retained prior information; stopping occurred before provider context with tool=0, so incomplete tool rounds were not tested against a real model. Mac remains ad-hoc signed and unnotarized.
- Windows 11 ARM VM: native ARM64 and emulated x64. Release binaries passed 22→26 preservation, workbench and same-conversation stop/follow-up with an HTTP fixture. A new ordinary-user installer-only lane passed original 0.9.6 same-version replacement, actual EXE identities, data and registration preservation. All four updater epoch3 signatures were verified; no Authenticode. New-identity Session0 close/exit_app natural shutdown failures remain recorded; ARM64 raw separately passed Session1 close/destroyed. This is not a full formal-runtime pass.
- The signed arm64 APK passed same-signer/same build number 9006000 replacement, 22→26 preservation of 51 Required tables and preferences/wrapping data, plus fresh workbench/system appearance on a real Android SDK Emulator. AI entry only; no physical phone or general CleanShutdown validation.
- Both Linux architectures passed 22→26 preservation of 57 tables and 12 old rows, followed by direct FUSE AppImage cold startup after deb installation and real business views in Ubuntu VMs. Native Linux AI follow-up was not tested without independent model credentials.
- Incomplete tool rounds, unique complete results and exact coverage watermarks passed 3 shared integration tests / 4 real local HTTP scenarios; Windows native validation used a controlled HTTP fixture. This does not imply real-model or remote-tool validation on every platform.
- iPhone Simulator version/entry, fresh workbench/system and the two notification categories were exercised; no public upload. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.

### SHA256

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
