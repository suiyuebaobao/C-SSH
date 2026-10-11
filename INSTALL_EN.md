# C-SSH 0.9.6 installation and data preservation

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release uses notices and manual downloads only. No upgrade prompt or forced update is sent.

## Before installation

Use the official website or suiyuebaobao/C-SSH v0.9.6 release and verify SHA256. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64 / ARM64

Choose the package for your CPU; native ARM64 was validated on Windows 11 ARM64.

- Run the NSIS installer in the original installation directory. WebView2 Runtime must already be available; the installer has no Authenticode signature.
- For a new portable installation, extract and run C-SSH/C-SSH.exe.
- Portable upgrade from 0.9.0 or later: exit, back up the old EXE and old Setup in the adjacent Windows folder, preserve data in place, then adopt the new EXE and let it release matching Setup. A stale Setup is rejected rather than silently overwritten.
- Never replace a running EXE or overwrite/delete data. Handle historical MSI installations using their original installation method.

## Apple Silicon Mac

Open C-SSH_0.9.6_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. Mac automatic updates are deferred.

## Android arm64

Install C-SSH_0.9.6_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The 0.9.0 signer is retained, so 0.9.0, 0.9.1, 0.9.2 and 0.9.3 can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Linux x86_64 / ARM64

- Choose an amd64 or arm64 deb for your CPU and install through the Debian/Ubuntu package manager.
- Make the AppImage executable before launching; Ordinary mounted launch usually requires FUSE; this round verified the extracted AppImage entry point, not FUSE mounting. The aarch64 AppImage is the ARM64 build.
- Validated on Ubuntu 24.04.5 with X11 and a desktop Secret Service such as GNOME Keyring. Other distributions and Wayland remain unvalidated.
- Keep the existing data directory and system keyring during upgrades. Do not substitute copying encrypted databases from another device for migration. Linux uses manual downloads in this release.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.


## Data and verification

Mac is arm64-only, ad-hoc signed and unnotarized; Windows installers have no Authenticode signature. Android keeps the release signer; this round directly verified upgrades from 0.9.4. Do not uninstall or clear data to work around a signer difference. This schema22 build cannot open higher-schema test data; retain that test build and its separate data.

Cloud receives only permitted, explicitly synced client-encrypted ciphertext. AI conversations, raw records, Skills and long-term memory stay local; selected context is sent to the chosen model provider. This public repository contains documentation, binaries and a whitelisted Cloud production-source mirror. Client, shared-core, AI and Agent sources are not public.

- Ten artifacts total 583,386,396 bytes and are bound to the current build and package receipts. Existing Mac/Windows real-VM SSH/SFTP, monitor switching, navigation and deletion regressions are reused without repeating unchanged passing checks.
- Windows x64/ARM64 packages passed per-architecture package, install, 0.9.4→0.9.6 upgrade, uninstall and data-preservation checks in a Windows 11 ARM VM. x64 updater epoch3 signatures were cryptographically verified; these are not Authenticode signatures.
- Mac passed ordinary arm64 Release, ad-hoc strict-signature and DMG checks; it remains unnotarized. Android retained the release signer and passed actual Manifest/DEX 0.9.6/9006000, 16KiB alignment, and same-signer upgrade with settings and 21 protected tables preserved in an API36 arm64 emulator.
- Linux deb/AppImage packages passed per-architecture embedding and isolated startup, log entry and preference preservation on Ubuntu24.04 X11. AppImage used its extracted entry point; FUSE mounting and all remote workflows were not retested.
- iPhone received a Simulator build/identity check only and is not uploaded. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.
