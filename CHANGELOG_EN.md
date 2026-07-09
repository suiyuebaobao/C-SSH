[中文](CHANGELOG.md) | **English**

# Changelog

Download complete installers from [GitHub Releases](../../releases). Each release includes binaries, release notes, and verification details.

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
