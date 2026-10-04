# C-SSH 0.9.4 installation and data preservation

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release uses notices and manual downloads only. No upgrade prompt or forced update is sent.

## Before installation

Use the official website or suiyuebaobao/C-SSH v0.9.4 release and verify SHA256. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64 / ARM64

Choose the package for your CPU; native ARM64 was validated on Windows 11 ARM64.

- Run the NSIS installer in the original installation directory. WebView2 Runtime must already be available; the installer has no Authenticode signature.
- For a new portable installation, extract and run C-SSH/C-SSH.exe.
- Portable upgrade from 0.9.0 or later: exit, back up the old EXE and old Setup in the adjacent Windows folder, preserve data in place, then adopt the new EXE and let it release matching Setup. A stale Setup is rejected rather than silently overwritten.
- Never replace a running EXE or overwrite/delete data. Handle historical MSI installations using their original installation method.

## Apple Silicon Mac

Open C-SSH_0.9.4_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. Mac automatic updates are deferred.

## Android arm64

Install C-SSH_0.9.4_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The 0.9.0 signer is retained, so 0.9.0, 0.9.1, 0.9.2 and 0.9.3 can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Linux x86_64 / ARM64

- Choose an amd64 or arm64 deb for your CPU and install through the Debian/Ubuntu package manager.
- Make the AppImage executable before launching; Ordinary mounted launch usually requires FUSE; this round verified the extracted AppImage entry point, not FUSE mounting. The aarch64 AppImage is the ARM64 build.
- Validated on Ubuntu 24.04.5 with X11 and a desktop Secret Service such as GNOME Keyring. Other distributions and Wayland remain unvalidated.
- Keep the existing data directory and system keyring during upgrades. Do not substitute copying encrypted databases from another device for migration. Linux uses manual downloads in this release.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.

## Data and first use

0.9.4 uses client SQLite schema 22 and the existing migration path from 21. Existing data does not force onboarding; a new-profile guide can be skipped and reopened in Settings. Bulk import creates pending connections before authentication creates a host. Automatic Skills and operation logs do not replace backups. AI conversations, raw records, Skills and long-term memory are not uploaded to Cloud.

- All ten formal artifacts have names, byte counts and SHA256 values bound to their platform receipts, totaling 580,358,528 bytes. Windows has current Release and package gates on both architectures; Portable contains only `C-SSH/C-SSH.exe`. Installation lifecycle and ordinary user-profile startup were not repeated in this packaging round.
- Mac is the ordinary arm64 Release/DMG with checked ad-hoc signing and no notarization. Same-source Mac business evidence is reused without opening the user's normal profile. Android checks cover 0.9.4/9004000, the original signer, 16KB alignment, a fresh SDK Emulator cold start and a normal operation-log row.
- Both Linux architectures passed deb/AppImage package checks and isolated minimal startup; AppImage used its extracted entry point. This round adds no FUSE-mount, upgrade, other-distribution or physical-machine claim.
- Import, onboarding and automatic Skills reuse the October 2–3 native evidence: Mac arm64; Windows ARM64 and x64 within an ARM VM; Linux ARM64, with x64 native differences checked separately; Android SDK Emulator and iPhone Simulator. SSH/Windows management and real-provider learning/cold-restart reuse retain their individual scopes. A complete matrix was not rerun on every architecture.
- Operation-log evidence includes same-source Mac business operations and a normal Android row. Other platforms have the current shared implementation, frontend/package checks and their minimal-startup evidence; this does not claim every business-log path was rerun on every platform.
- No claim covers physical Android/iPhone devices, every input method, every Linux distribution/Wayland, every recovery scenario or untested features in this round.
