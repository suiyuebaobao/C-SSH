[中文](CHANGELOG.md) | **English**

# Changelog

Download complete installers from [GitHub Releases](../../releases). Each release includes binaries, release notes, and verification details.

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
