[中文](CHANGELOG.md) | **English**

# Changelog

Download complete installers from [GitHub Releases](../../releases). Each release includes binaries, release notes, and verification details.

## v0.6.11 - Cross-Platform Connection Reuse, Mobile Terminal, and Safe Recovery

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
- Android now offers system, light, and dark themes. System mode follows the device theme automatically, and the preference is stored in local SQLite and restored after restart.
- Windows and Linux use the system Save As dialog for file or directory downloads, while Android uses the Storage Access Framework (SAF) system document picker. Canceling does not start a connection or download; chosen destinations retain resume support and integrity verification.
- SSH connections now have clear staged 8-second failure boundaries for DNS resolution, TCP connection, SSH handshake, and authentication. A failed or timed-out stage returns immediately instead of repeating the wait with another credential.
- When no password is entered explicitly and the stored private key is explicitly rejected, the client can try the encrypted-vault password within the same SSH session. After authentication succeeds, it continues public-key repair to reduce future password prompts.

### Fixed
- Fixed fresh local databases sometimes reporting `database is locked` when multiple pages or background tasks initialized at once. AI, files, monitoring, host data, and preferences can now open and recover reliably from the same SQLite database.
- Fixed installed-but-stopped firewalld being reported as a query failure. The client now shows it as not running, keeps port actions disabled, and never starts or installs the firewall on its own.
- Fixed offline hosts retaining an old green status. Online state now comes from the current real connection result, while historical metrics are shown only as stale data.
- A failure in one feature channel no longer disconnects a healthy shared SSH transport. Reconnection occurs only after the connection is confirmed lost, and mutating operations that may already have arrived are not replayed automatically.
- Host-key trust now stops safely when its record cannot be read, parsed, or saved. The error is not treated as a first connection, the current session is not delivered, and no other credential path is attempted.
- Before deleting a host or reinstalling, C-SSH verifies that the related service, process, persistent session, data, and public key belong to C-SSH. If any resource cannot be verified, it stops and preserves the current state instead of touching another service, session, or key.
- Dangerous systemd directives are audited with legal whitespace syntax, while an unprovable leftover tmux socket is preserved with an explicit repair error instead of being deleted automatically.
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

## v0.6.10 - Production Linux Desktop and Transactional Agent Deployment

### Downloads
- Windows installer: `Creation-SSH_0.6.10_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.10_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.10_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.10_android-arm64.apk`
- Android AAB: `C-SSH_0.6.10_android-arm64.aab`
- Linux AppImage: `Creation-SSH_0.6.10_linux-x86_64.AppImage`
- Linux deb: `Creation-SSH_0.6.10_linux-amd64.deb`

### Added
- Added the first production Linux desktop AppImage and deb, built and verified from the independent `linux/` project.
- CLI, Windows, Android, and Linux now share one agent deployment transaction with unique staging/backup paths and byte-length plus SHA256 verification before replacement.
- Added a cross-client remote deployment lock. Existing locks are never taken over automatically: active locks report busy and stale locks require explicit repair.

### Fixed
- Fixed split SQLite roots that could show a host in the list while agent repair could not find it. Hosts, credentials, SSH/repair, monitoring, and AI now use the same data root.
- Unix default and explicit `CS_DATA_DIR` roots enforce `0700` on data/key directories and `0600` on SQLite, failing closed when permissions cannot be tightened.
- systemd deployment validates the fixed `FragmentPath`, raw and effective `ExecStart`, and the active executable before stop/replacement, rejecting foreign same-name units and drop-in overrides.
- Existing units preserve their enable state, fresh-unit cleanup failures no longer report success, and `KillMode=process` is verified so agent updates do not terminate persistent tmux workloads.
- Readiness and strict-version handshake failures use two-phase rollback; backups and locks are cleared only after the previous agent is restored and returns a valid protocol response.
- Process ownership now uses exact `/proc/<pid>/exe` matching for CentOS 7.9 compatibility. Linux packaging also compares the gzip payload with the raw agent by bytes, architecture, version, and SHA256.
- Refreshed the Windows/Linux desktop packages in place on 2026-07-12: AI host and model selectors no longer show nested native control chrome and now share one outer border, focus ring, and right-aligned chevron. The version remains `0.6.10`, and Android assets are unchanged.

### Verified
- Full Rust workspace tests, Clippy with `-D warnings`, formatting, platform-boundary, version-consistency, and gzip-payload gates passed.
- CentOS 7.9 and Ubuntu 24 passed real deployment, handshake, and `MetricsSnapshot`; a real 0.6.9 agent on Ubuntu was rejected and automatically restored to 0.6.10.
- Fault injection covered readiness failure, an effective-ExecStart drop-in, a disabled unit, active/stale locks, tmux survival, and zero residual deployment files.
- The refreshed Windows portable package launched with a working main window, Tauri runtime, isolated SQLite, main AI page, and standalone AI window; task processes and isolated data were cleaned afterward.
- The final Android x86_64 test package was freshly installed on MuMu and verified agent 0.6.10 deployment, user-systemd, persistent terminal, monitoring, navigation, and force-stop recovery. It is not uploaded.
- Android arm64 APK/AAB report `versionName=0.6.10`, SDK 24/36, and arm64-only native code; APK v2 and AAB JAR signatures passed.
- The refreshed Linux AppImage/deb launched in a real Ubuntu 24 desktop session. Both passed process-lifetime, agent-linked Collector, SQLite `integrity_check=ok`, `+4` metrics, `0700/0700/0600` permissions, and zero-residue checks. Wayland did not expose a reliable `xdotool` visible-window delta, so that probe is not claimed as passed.

### SHA256
- `Creation-SSH_0.6.10_x64-setup.exe`: `5EA8FC3CD3CE08DA004B062DF28DFA4F86F656275338D84C963C114FD193E82E`
- `Creation-SSH_0.6.10_x64_en-US.msi`: `F1E41543BE522BAF6940073450873A99B2FD709243BD3C6F20673FB4EF57C750`
- `Creation-SSH_0.6.10_portable-Windows-x64.zip`: `DCC71D79C8EE681E1F79A7D53AEAADED251A97CC8AD3C511178692994AA21A66`
- `C-SSH_0.6.10_android-arm64.apk`: `5D347EDC629D09A6C683BF7B82E0F06DC75DA87EFBB43E73DF7663749C100E5C`
- `C-SSH_0.6.10_android-arm64.aab`: `B45101EBBB40BAF66BEC2237BACE4E32AE2B82696A51F91C5F843CD846522E84`
- `Creation-SSH_0.6.10_linux-x86_64.AppImage`: `3E7B299DBD639AB27EC16CC7E5BA34540FD8C696FF9C96CAD58D26D37E67FE55`
- `Creation-SSH_0.6.10_linux-amd64.deb`: `2A1FEE0CB982ED886131D1416613B4A99A8D8B92C86E6EF2F28AB68099F11179`

## v0.6.9 - Configurable Host Collection and First-Refresh Fix

### Downloads
- Windows installer: `Creation-SSH_0.6.9_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.9_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.9_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.9_android-arm64.apk`
- Android AAB: `C-SSH_0.6.9_android-arm64.aab`
- Linux desktop installers are deferred from this release.

### Added
- Background collection on Windows, Linux, and Android supports `1–10` cross-host workers, default `4`; the collection interval ranges from `1–3600` seconds, default `6`.
- Android now exposes interval, cross-host concurrency, and local retention settings, all persisted in local SQLite.
- The independent Linux client project and build gates are in place, but Linux installers are not included in this public Release.

### Fixed
- Fixed first-round metrics being stored in SQLite while the Hosts page continued to show stale values; the current page now refreshes as soon as the full round is committed.
- Unified Windows/Linux Hosts and MonitorList event/fallback scheduling, coalescing overlapping refreshes and preventing late responses from replacing newer data.
- Fixed collection settings being observed as mixed old/new values by storing all three atomically and reading them as one consistent snapshot.

### Verified
- The full Rust workspace tests and production frontend builds for Windows, Linux, and Android passed.
- The production Windows app was verified to refresh first-round metrics without leaving the Hosts page.
- The Android x86_64 `0.6.9` test package passed signature/version/ABI checks, installed, and launched on MuMu. It is not uploaded to the Release.
- The production Android arm64 APK/AAB build passed. The APK is `0.6.9 (6009)`, targets SDK `24–36`, contains only `arm64-v8a`, and passes v2 signature verification; the AAB also contains arm64 native libraries only.

### SHA256
- `Creation-SSH_0.6.9_x64-setup.exe`: `6ECF9CBB4A06440CE735C4EDD70F43F770DBBF774AEEC70FE74914D1FC19B3F1`
- `Creation-SSH_0.6.9_x64_en-US.msi`: `D9F4A11D8562093F5859530448EC4CD2CA317022391E1D504EB04B161661BF87`
- `Creation-SSH_0.6.9_portable-Windows-x64.zip`: `9B49B7D69F64E9FFC3386BA663962FFA7B06F8DEFCF14E8340795951713E0E09`
- `C-SSH_0.6.9_android-arm64.apk`: `4245852EAEB217AAC0F00F7731D30FDD011759D2F5BCB9811E49E383DFD9437F`
- `C-SSH_0.6.9_android-arm64.aab`: `FF5488C3547D1E42F83A6B5185BEDEF1BF03C370264CE1CDDCC4785158AB07DA`

## v0.6.8 - AI Workspaces, Monitoring Summaries, and Adaptive UI

### Downloads
- Windows installer: `Creation-SSH_0.6.8_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.8_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.8_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.8_android-arm64.apk`
- Android AAB: `C-SSH_0.6.8_android-arm64.aab`

### Added
- Unified history, host, permission, execution-profile, tool-loop, and context controls across the desktop and mobile AI workspaces. Desktop supports multiple independent AI pop-out windows for parallel host and task workflows.
- Persisted AI history, permission mode, execution profile, tool-loop limit, custom AI context window, and context preferences in local SQLite so user choices survive page changes and app restarts.
- Added Stable, Balanced, Fast, and Ultra presets for AI and other short agent requests, with 1/2/4/8 concurrency limits. SSH, channel, or streamlocal transport errors temporarily auto-downgrade the affected host to one concurrent request.
- Added cross-host monitoring summaries, 6-second automatic collection, and a collapsible detail sidebar for continuous checks and narrow layouts.
- Added Android system-language and system-theme following, including the mobile light-mode request from GitHub Issue #18.
- Synchronized Windows, Android, all five fixed public asset names, and the Linux agent to `0.6.8`; the agent handshake reports `0.6.8`. iOS and macOS are not part of this release.

### Fixed
- Fixed desktop and mobile AI workspaces losing history access, permissions, execution profiles, tool-loop limits, or context preferences after navigation, restart, or multi-window use.
- Fixed high-concurrency AI tool calls amplifying transport failures in short agent requests by enforcing shared concurrency protection and automatic downgrade recovery.
- Fixed agent-backed features becoming unavailable when older OpenSSH environments such as CentOS reject direct streamlocal; the client now falls back to the stdio bridge automatically.
- Fixed credential reuse and authentication fallback after adding a key-based host: the local encrypted-vault key is preferred, with saved-password or explicit password fallback when needed.
- Fixed global layout issues across narrow desktop windows, mobile portrait screens, and dynamic content; sidebars, toolbars, sheets, command-snippet execution/result containers, and primary work areas now adapt to available space.
- Fixed the Android soft keyboard covering the AI composer and the AI-response content overflowing its card boundary as tracked in GitHub Issue #16.
- Fixed Android system language and theme not refreshing consistently across some lifecycle and reopen paths, with clearer feedback when following the system.
- Fixed upload/download integrity handling at the completion of chunked and resumable transfers so incomplete results are not reported as successful.

### Verified
- The production Windows desktop build completed and was launched on a real machine for the affected feature checks.
- Real Ubuntu and CentOS paths verified agent communication and compatibility fallback. DeepSeek AI conversation and tool-loop E2E passed.
- The Android production build and static AndroidX SplashScreen C gate passed. A real Launcher cold start on the MuMu x86_64 emulator was verified frame by frame: the system C, frontend wordmark, large C, and main screen connect continuously without a solid-color gap or clipping.

### SHA256
- `Creation-SSH_0.6.8_x64-setup.exe`: `87FC035CF668A3DCC1F0DB9DC9F9DFD0762BFC4673B23E135253D6227B4C1A40`
- `Creation-SSH_0.6.8_x64_en-US.msi`: `B4F8611C3AF885378C2A908E61387381005138BC45CCBED0FC038DC08758CBE9`
- `Creation-SSH_0.6.8_portable-Windows-x64.zip`: `47629E99884378CBCD65CB0AD004C7AF6441492AA741F341F50F63C447842DB5`
- `C-SSH_0.6.8_android-arm64.apk`: `1EE2636F5004C4204FD48F58953819DC67D95F35C464FC420A102E243CE40753`
- `C-SSH_0.6.8_android-arm64.aab`: `B009C9739C3AE4CE42339639BDA45676D9C4DB1D3D7926244B28D27DAD2E889A`

## v0.6.7 - AI Assistant Pop-out Fix and Real Release Verification

### Downloads
- Windows installer: `Creation-SSH_0.6.7_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.7_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.7_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.7_android-arm64.apk`
- Android AAB: `C-SSH_0.6.7_android-arm64.aab`

### Added
- Added a hard pre-release functional verification gate: before a public Release, new tag, or installer upload, the final desktop app and mobile test package must be run and the changed features must be exercised for real.
- Extended the desktop AI window regression check to require the Windows-safe async Tauri command pattern for dynamic WebView creation.

### Fixed
- Fixed the desktop AI assistant pop-out opening as a blank white window in the Windows release build; window creation now uses Tauri's recommended async command pattern.
- Fixed corrupted or incorrect labels around the AI pop-out button, history entry, and agent performance presets in some locales.
- Fixed AI child windows lingering after the main window exits by closing `ai-*` windows when the main window closes.

### Verified
- Desktop `npm run test:ai-window` passed, covering the AI pop-out URL, `ai-*` permissions, async command requirement, history label, and localized strings.
- Rust `cargo fmt --check`, desktop Tauri workspace `cargo fmt --check`, and `cargo check` passed; desktop `npm run build` and `npm run tauri build` passed.
- The final Windows executable was run again and the AI assistant pop-out rendered normally, confirmed during the live release check.
- Android arm64 release APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, versionName `0.6.7`, versionCode `6007`, and ABI `arm64-v8a` only.
- Android x86_64 debug APK was installed and launched on the MuMu emulator for testing only; package info reports versionName `0.6.7` and ABI `x86_64`. This package is not uploaded to the public Release.
- SHA256 hashes were recalculated for all public assets, and public repository text plus release notes were checked for sanitized content.

### SHA256
- `Creation-SSH_0.6.7_portable-Windows-x64.zip`: `B80BC866177D5D9C82034E21BEB41C6B5100A6A0BD62039A5E7D31F8C8A0983F`
- `Creation-SSH_0.6.7_x64_en-US.msi`: `4EE2AF5FFA7CDEF55A5C44144D12B5B6CE30C0D5C28FC1BB06DD803AA4CC84E1`
- `Creation-SSH_0.6.7_x64-setup.exe`: `0447447C4DFFB35DFF48925134368B197AF8B7954033293567CE59A17C2B6D1E`
- `C-SSH_0.6.7_android-arm64.aab`: `0DE4EA9BF5D6B021E90CD8E9C889E73D6DF3E64CF775606B14247E0E4487153E`
- `C-SSH_0.6.7_android-arm64.apk`: `4EA626D63E709F6B1BF0D1A014C951C76153AA2BB141243A250F550DC5BFB402`

## v0.6.6 - Agent Bridge Fallback, Concurrency Guard, and Key-Based Hosts

### Downloads
- Windows installer: `Creation-SSH_0.6.6_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.6_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.6_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.6_android-arm64.apk`
- Android AAB: `C-SSH_0.6.6_android-arm64.aab`

### Added
- Added an agent `--stdio-bridge` mode. When sshd rejects `direct-streamlocal`, the client automatically falls back to a bridge channel that still reaches the local unix socket, keeping agent-backed features usable on older systems such as CentOS 7.9.
- Added Stable / Balanced / Fast / Ultra presets for short agent requests, with per-host persistence in local SQLite.
- Added a desktop AI assistant pop-out entry so the current AI assistant can open in an independent window, preparing the UI for multiple AI assistant windows.
- Added OpenSSH private-key authentication to desktop and mobile host creation; pasted keys are stored in the local encrypted vault and reused for later connections.
- Improved host grouping, delete confirmation, authentication error guidance, multilingual host text, and synchronized public versioning to `0.6.6`.

### Fixed
- Fixed agent features being unavailable on CentOS 7.9/OpenSSH 7.4 environments when sshd returned `AdministrativelyProhibited` for `direct-streamlocal`; the client now falls back to the bridge without asking users to re-enter SSH passwords.
- Fixed short agent requests lacking a shared concurrency limit under high-concurrency tool-call scenarios; all non-long-stream requests now go through the agent request guard and temporarily auto-downgrade to 1 concurrent request after SSH/channel/streamlocal transport errors.
- Fixed desktop and mobile deploy / files / monitor / appcenter / sysmgmt paths that could bypass the shared agent concurrency guard.
- Clarified the bridge failure message: users only see the fallback failure text when both direct streamlocal and bridge transport fail.
- Fixed some host-credential flows still prompting for passwords after credentials had already been saved; successful host creation now prefers the local vault credential on later connections.

### Verified
- Rust `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p client`, and `cargo check -p agent` passed.
- Desktop/mobile Tauri workspaces passed `cargo check`; desktop and mobile `npm run build` passed.
- x86_64 and aarch64 Linux musl agent releases were built with `cargo zigbuild`, both synchronized to version `0.6.6`.
- Desktop `npm run tauri build` produced Windows setup/MSI; the main executable reports file and product version `0.6.6`, and the portable zip was rebuilt.
- Android x86_64 debug APK was installed on the MuMu emulator; `aapt dump badging` reports `versionName=0.6.6` and `native-code='x86_64'`. This package is for testing only and is not uploaded to the public Release.
- Android arm64 release APK/AAB were generated; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, versionName `0.6.6`, versionCode `6006`, and ABI `arm64-v8a` only.
- Real CentOS 7.9 verification: native `direct-streamlocal` is still rejected by sshd, but the current client automatically uses `stdio-bridge` and completes `handshake`, `sysinfo`, `metrics`, and `mon` streaming.
- Source/docs, public repository text, and release notes passed sanitized scans for real credentials, tokens, private keys, and non-example IPs.

### SHA256
- `Creation-SSH_0.6.6_portable-Windows-x64.zip`: `68773A304D2C74C3BEE922F75FC3D0C5C61F3D077C57083132C4371E8F98EFBE`
- `Creation-SSH_0.6.6_x64_en-US.msi`: `D7A22FB19E46BD4CFD0A17DBB2DF847051E8246961460111180E1FE2228C2989`
- `Creation-SSH_0.6.6_x64-setup.exe`: `1D2EC3D2F9F22A52AAC94227AF68D2EACBAB4C93F7CD350B56F7C0A3E9C2D76C`
- `C-SSH_0.6.6_android-arm64.aab`: `67E0EB455CA9A92F0379F785D31486A002884D3FAED9EED75E6C055AC2BFBEB4`
- `C-SSH_0.6.6_android-arm64.apk`: `78990A2DB4F286E0E46C0FF0AD959A57A0CDE79A652DFF795028AE7AF504C8F7`

## v0.6.5 - AI Workspace, History Entry, and Sensitive Attachment Blocking

### Downloads
- Windows installer: `Creation-SSH_0.6.5_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.5_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.5_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.5_android-arm64.apk`
- Android AAB: `C-SSH_0.6.5_android-arm64.aab`

### Added
- Added a compact top workspace bar to the desktop AI assistant, moving history from the permanent side panel into a top popover and freeing the main chat area.
- Added the same top history entry to the mobile AI assistant for narrow screens and prepared a shared workspace structure for future multi-AI-window work.
- Added desktop/mobile AI workspace static regression scripts covering the top history entry, mobile keyboard anchoring, and sensitive attachment blocking.
- Continued enforcing versioned public asset names; Windows, Android, agent, and public documentation are synchronized to `0.6.5`.

### Fixed
- Fixed the desktop AI history panel occupying main layout width all the time; it now opens only when needed from the top workspace bar.
- Fixed the mobile AI input area being obscured by the soft keyboard by anchoring the composer to the visible bottom area.
- Fixed invalid history-session actions when no server is selected by adding disabled states and keyboard dismissal handling.
- Fixed attachment filtering relying too heavily on MIME/extension allowlists; filenames such as `.env`, private keys, and key/certificate files are now blocked before type checks.

### Verified
- Desktop `npm run test:ai-workspace`, `npm run test:locale-system`, and `npm run build` passed.
- Mobile `npm run test:ai-workspace`, `npm run test:ai-keyboard`, `npm run test:locale-system`, and `npm run build` passed.
- The x86_64 musl agent release was built with `cargo zigbuild -p agent --target x86_64-unknown-linux-musl --release`; the build log reports `agent v0.6.5`.
- Desktop `npx tauri build` was rerun after the agent 0.6.5 build and produced Windows setup/MSI; the Windows executable and setup metadata report `0.6.5`.
- Android arm64 release APK/AAB were rebuilt after the agent 0.6.5 build; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, versionName `0.6.5`, versionCode `6005`, and ABI `arm64-v8a` only. The AAB was inspected and only contains the `arm64-v8a` native lib.
- Source/docs and public release text passed sanitized scans for private keys, tokens, real credentials, and non-example IPs.

### SHA256
- `Creation-SSH_0.6.5_portable-Windows-x64.zip`: `96150F4C5128A33FFE558289FB2261BADF31A73B4508E1243783952C2401B36A`
- `Creation-SSH_0.6.5_x64_en-US.msi`: `47E0817CC3F69E8ADA7B9F364FCDE14B876E99E3425856D2D5C00FEBB808C153`
- `Creation-SSH_0.6.5_x64-setup.exe`: `FD532382E110769DCC3DA40362A6102008822005F4FD67E5ED215D21A88C889A`
- `C-SSH_0.6.5_android-arm64.aab`: `C217DA6561ED0DE2A79E684B6C95B6111F74CB96AE784B40379658C543217EBB`
- `C-SSH_0.6.5_android-arm64.apk`: `4F279E03184F0942C276F4B915129844856FAEDFFB5A72B9522E608540238F79`

## v0.6.4 - AI Run Recovery, System Language, and Mobile Input Fixes

### Downloads
- Windows installer: `Creation-SSH_0.6.4_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.4_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.4_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.4_android-arm64.apk`
- Android AAB: `C-SSH_0.6.4_android-arm64.aab`

### Added
- Added AI run-state persistence and page recovery, so active or recently finished AI turns can restore their event stream and status after switching away and coming back.
- Added the first ACP/Hermes protocol skeleton with capability descriptors and integration entry points; full Hermes workflow work remains staged for later releases.
- Added GitHub update checks on desktop and mobile; the app opens the GitHub Releases page instead of silently auto-installing updates.
- Added system-language following as the default language mode while keeping manual selection across 9 UI languages; preferences continue to be stored in SQLite.
- Replaced visible mobile AI emoji indicators with componentized SVG icons for more consistent rendering.

### Fixed
- Fixed crashes in high-concurrency AI tool-call scenarios by executing multiple tool calls from the same model turn through a controlled serial path and adding run-state guards.
- Fixed the mobile AI input area being covered by the Android soft keyboard: Activity now uses `adjustResize`, and the frontend adjusts bottom spacing from `visualViewport`.
- Fixed mobile AI permission mode, execution mode, and tool-loop preferences reverting after leaving the page or restarting.
- Added the missing PC-side custom tool-loop control, persisted the same way as mobile.
- This release also carries forward terminal sizing/status-bar fixes, the 80% default desktop transparency, synchronized agent versioning, and versioned public asset names.

### Verified
- Rust `cargo fmt --check`, `cargo build --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` all passed.
- Desktop/mobile frontend `npm run build` passed; system-language, mobile AI keyboard avoidance, and splash timeline regression checks passed.
- Desktop `npm run tauri build` produced Windows setup/MSI; the Windows executable and setup metadata report `0.6.4`; the portable zip was rebuilt.
- Android arm64 release APK/AAB were generated; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, versionName `0.6.4`, versionCode `6004`, and ABI `arm64-v8a` only. The AAB was inspected and only contains the `arm64-v8a` native lib.
- Source/docs and public release text passed sanitized scans for private keys, tokens, real credentials, and non-example IPs.

### SHA256
- `Creation-SSH_0.6.4_portable-Windows-x64.zip`: `CFC6D71575E26C1F2E84539505AF1AC5DB72B0B63F182EF3DA2C6191AE3AE799`
- `Creation-SSH_0.6.4_x64_en-US.msi`: `1B3D8364CED5BA4A6E53FD28F4024A53322C286D904C4AC95527A0827AF7BAF6`
- `Creation-SSH_0.6.4_x64-setup.exe`: `48A0318A3AFDC7679301C00F46DCA0C6BCC329BD3FF2A8184A90B1B6298A6131`
- `C-SSH_0.6.4_android-arm64.aab`: `891AC7731D7B592122294B28E508030DBEDB1792253F6DF8C09F296AAEF25001`
- `C-SSH_0.6.4_android-arm64.apk`: `5F8099C77DFD547DCF969937D02AA07C184F3C05B1A5A107EE6BFA43F26CA664`
## v0.6.3 - Version Sync, AI Layout, and Package Metadata Fixes

### Downloads
- Windows installer: `Creation-SSH_0.6.3_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.6.3_x64_en-US.msi`
- Windows portable: `Creation-SSH_0.6.3_portable-Windows-x64.zip`
- Android arm64: `C-SSH_0.6.3_android-arm64.apk`
- Android AAB: `C-SSH_0.6.3_android-arm64.aab`

### Added
- Synchronized the server agent package version to `0.6.3`; lazy deployment now checks both protocol version and packaged agent version, so same-protocol older agents are upgraded too.
- Formalized the version-sync rule for every release: source versions, Tauri config, npm package versions, Android versionName/versionCode, About page, agent version, public Release filenames, and docs must move together.
- The desktop AI assistant now expands to the full available width when the window is maximized, matching the rest of the desktop app.

### Fixed
- Fixed Windows installers, Android app metadata, and About pages still showing `0.1` / `0.1.0`; this release consistently reports `0.6.3`.
- Fixed the agent version staying at `0.0.1`; the runtime agent version now comes from the package version and changes with releases.
- Fixed the desktop transparency default not applying as requested; the default is now 80%, and older saved `0` values migrate to 80%.
- Fixed AI execution-mode selection not persisting like permission mode; reopening keeps the last selected option.
- Clarified public notes for AI permission persistence, custom context windows, and custom tool-loop limits so users can find the new controls.

### Verified
- `cargo fmt --check` and `cargo check -p agent` passed, with the agent build reporting `0.6.3`.
- Desktop/mobile `npm run build` passed; desktop/mobile Tauri workspaces passed `cargo check`.
- Desktop `npm run tauri build` produced the `0.6.3` Windows setup/MSI; the Windows executable reports version `0.6.3`.
- Android x86_64 debug APK was installed on the MuMu 12 emulator; system package info reports `versionName=0.6.3`, and the About page shows `0.6.3`.
- Android arm64 release APK/AAB were generated; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, SDK 24/36, and ABI `arm64-v8a` only.

## v0.6.2 - AI Settings Persistence and Custom Model UX Fixes

### Downloads
- Windows installer: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows portable: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### Added
- Added a tool-loop number control directly to the desktop AI assistant toolbar, so PC users no longer need to dig through settings to customize the loop limit.
- Added a context-window setting for custom AI providers, from 4,096 to 2,000,000 tokens with a 128k default; custom models now use this value for the context remaining indicator.
- Split AI configuration persistence into a dedicated `ai_config` backend module, keeping the execution-loop module below the file-size limit.

### Fixed
- Fixed the AI assistant permission mode reverting to the first read-only option after leaving the page and returning; the app now restores the in-session cached mode while still persisting to SQLite.
- Fixed permission-mode changes not being saved early enough when users immediately switch away after choosing a mode; the watcher now flushes synchronously.
- Fixed custom AI settings not carrying a context-window value through the desktop and mobile save paths.
- Fixed a desktop clippy blocker caused by lazy continuation in a deployment doc comment.

### Verified
- Desktop and mobile `npm run build` passed.
- Desktop and mobile Tauri workspaces passed `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings`.
- AI locale JSON files for all 9 languages parse successfully.
- Desktop `npm run tauri build` produced the Windows setup/MSI, and the portable zip was rebuilt.
- Android arm64 release APK/AAB were generated; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, SDK 24/36, and ABI `arm64-v8a` only.

## v0.6.1 - Terminal UX and Agent Compatibility Fixes

### Downloads
- Windows installer: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows portable: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### Added
- Raised the AI tool-loop default from 16 to 30 iterations, with a configurable custom limit that stays inside the safety range.
- Persisted all five AI assistant permission modes so reopening the app keeps the user's last selected mode.
- Added compatible login-key setup for CentOS/RHEL/FIPS/older OpenSSH environments: Ed25519 first, with RSA/ECDSA P-256 fallback when needed.
- Added stable classification for server-side `direct-streamlocal` policy rejection so it is not mistaken for a bad SSH password.
- Added shared authentication-error classification so Files, Monitoring, System Management, App Center, and related agent pages use the same clear messages.
- Split desktop/mobile deployment helpers for credentials and assets, with documentation and code indexes updated to the current structure.

### Fixed
- Fixed the desktop terminal CLI/xterm area being too small: the terminal view now uses the full available width, with tighter header controls and a more stable toolbar on narrow screens.
- Fixed the terminal cursor and last line being clipped at the bottom by adding proper xterm bottom padding and line-height constraints.
- Fixed persistent-terminal status/mode display when the agent channel is unavailable; the app now temporarily falls back to a plain SSH PTY for the current connection without overwriting the user's saved terminal mode.
- Fixed agent private-key setup failures seen on CentOS 7.9-like environments.
- Fixed repeated SSH password prompts when SSH works but the server rejects agent unix-socket forwarding; terminal now temporarily falls back to a plain SSH PTY.
- Fixed lazy deploy/key repair so saved SSH passwords are reused before interrupting the user for password input.
- Fixed Files, Monitoring, System Management, App Center, and other agent pages treating `direct-streamlocal` rejection as a bad SSH password; they now report the server-side agent-channel policy block clearly.

### Verified
- Rust `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` passed.
- Desktop `npm run tauri build` produced the Windows setup/MSI, and the portable zip was rebuilt.
- Android arm64 release APK/AAB were generated; the APK passed `apksigner verify --verbose --print-certs` and `aapt dump badging`, with package `com.creationssh.mobile`, SDK 24/36, and ABI `arm64-v8a` only.
- Sanitized real-server verification covered CentOS 7.9 and Ubuntu 24: CentOS key setup works and falls back to SSH when the agent channel is policy-blocked; Ubuntu agent handshake and command execution work normally.
- GitHub Issues #1, #2, #3, and #4 were each answered with the fix details and linked to v0.6.1.

## v0.6 - Smooth Splash and Icon Safe Area

### Downloads
- Windows installer: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows portable: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### Added
- Formalized the public release workflow: README, changelog, and Release notes stay bilingual, and every public package update uses a new tag instead of overwriting an existing release.
- Added the mobile splash timeline regression check `npm run test:splash` to cover both the large-C bridge and pre-rendering the app shell before splash fade-out.

### Fixed
- Fixed the visible gap during the Android splash transition from the small wordmark into the large C; the C stage now mounts during the word-collapse phase and fades in with overlap.
- Fixed the brief black frame after splash fade-out by pre-rendering the main route and bottom tabs while the splash overlay is still covering the screen.
- Fixed the Android icon safe area: the C is smaller, no longer clipped, and launcher/foreground/round icons all use the full source icon so the border remains visible.

### Verified
- agent x86_64 musl release, mobile tests/build, and desktop Tauri release build passed.
- Android x86_64 debug APK was rebuilt after clean and installed on the MuMu 12 emulator; 3500/3900/4300ms startup frames keep the large C/border visible, and 5000ms reaches the main screen.
- Android arm64 release APK passed signature, package name, SDK, and ABI checks; ABI is `arm64-v8a` only.

## v0.5 - Branding Refresh and Stable Packages

### Downloads
- Windows installer: `Creation-SSH_0.1.0_x64-setup.exe`
- Windows MSI: `Creation-SSH_0.1.0_x64_en-US.msi`
- Windows portable: `Creation-SSH-portable-Windows-x64.zip`
- Android arm64: `C-SSH-android-arm64.apk`
- Android AAB: `C-SSH-android-arm64.aab`

### Added
- New brand C icon: dark glass background with a metallic silver C, unified across desktop, Android launcher icons, and in-app branding.
- Public release notes are now bilingual, with GitHub Releases serving as the version update list.
- Expanded internal handoff, architecture, protocol, release, testing, code map, and code index documentation.

### Fixed
- Fixed Android release icon synchronization so launcher, foreground, and round icons no longer keep the old artwork.
- Fixed missing Android `splash_window_bg` resource so release and debug builds package correctly.
- Fixed clippy blocking the AI session entry point by replacing an oversized argument list with a structured `RunRequest`.
- Documented the boundary between the private source repository and the public promotion repository to avoid leaking APKs, promo archives, credentials, screenshots, or logs into the source repo.

### Verified
- Rust fmt, clippy, and workspace tests passed.
- Windows desktop release build passed.
- Android arm64 release APK passed signature, package name, SDK, and ABI checks.
- Android x86_64 debug build was smoke-tested on the emulator for cold start and main-screen rendering.
- Sanitization checked before publishing: no real IPs, credentials, or test-server details.

## v0.4 - Desktop and Android

### Added
- First public Windows desktop installers.
- Improved Android splash animation to reduce white flash and main-screen flicker.
- Fixed Android app icon to use a full dark-base C.
- AI assistant supports OpenAI-compatible APIs and Anthropic.

### Downloads
- `C-SSH-android-arm64.apk`
- `Creation-SSH-portable-Windows-x64.zip`
- `Creation-SSH_0.1.0_x64-setup.exe`
- `Creation-SSH_0.1.0_x64_en-US.msi`
