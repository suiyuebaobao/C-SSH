# C-SSH 0.9.9 installation and data preservation

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release notifies users and directs them to download and install manually. There are no forced upgrades or background downloads or installations. Android build number: 9009000. iOS remains paused.

## Before installation

Use the official website or the [v0.9.9-r2 release](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.9-r2), select your platform, and verify SHA256. The Windows, Mac, and Android files are unchanged; users already running 0.9.9 do not need another download. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64

This release provides only a Portable ZIP. New client installers and native ARM64 packages are paused. An existing WebView2 Runtime is required.

- New installation: extract and run `C-SSH/C-SSH.exe`; do not run from inside the ZIP.
- From 0.9.8: exit normally, keep adjacent `data` and existing OS keys, and replace only the EXE. The application handles its supporting components; manual deletion is not required. If a warning appears, preserve existing data before addressing it.
- Do not replace a running executable or clear data. Published older installers remain available; no new NSIS/MSI is supplied.

## Apple Silicon Mac

Open C-SSH_0.9.9_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. For 0.9.9, all platforms use notifications and manual downloads only.

## Android arm64

Install C-SSH_0.9.9_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The existing release signer is retained; preserve data when replacing a compatible same-signer installation. Compatible earlier builds with the same signer can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Linux x86_64

This release provides C-SSH_0.9.9_amd64.AppImage.

- Make the downloaded file executable, then launch it. Normal mounted operation requires FUSE.
- This release was tested on an X11 desktop. A desktop Secret Service such as GNOME Keyring is required. Other distributions and Wayland remain unvalidated.
- Exit the old application before upgrading and retain the existing data directory and system keyring. Do not substitute copying encrypted databases from another device for normal migration.
- Older Linux ARM64 and deb packages remain in [past releases](https://github.com/suiyuebaobao/C-SSH/releases); they are older versions, not new 0.9.9 builds.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.



## Data compatibility

Version 0.9.9 updates the local data format during upgrade. Preserve the complete data directory and existing OS keychain/vault, and allow the normal forward migration. Do not delete data or keys to address startup problems. Do not open a migrated database with an older incompatible client. Cloud does not contain AI conversations or memory; rollback requires the matching old program and a complete pre-upgrade backup. See the [changelog](CHANGELOG_EN.md) for validation boundaries.
