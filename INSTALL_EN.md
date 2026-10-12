# C-SSH 0.9.6 Revision 3 installation and data preservation


[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release uses notices and manual downloads only. No upgrade prompt or forced update is sent.

Announcements and manual downloads only: no upgrade reminder or forced update. The product version remains 0.9.6; revision 3 uses the new tag `v0.9.6-r3`. Ten assets cover four platforms; iOS is not uploaded. All earlier releases and original 0.9.6 assets remain intact. Android/iOS build number is 9006000; wire and signing identities are unchanged.

## Before installation

Use the official website or suiyuebaobao/C-SSH v0.9.6-r3 release and verify SHA256. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64 / ARM64

Choose the package for your CPU; native ARM64 was validated on Windows 11 ARM64.

- Run the NSIS installer in the original installation directory. WebView2 Runtime must already be available; the installer has no Authenticode signature.
- For a new portable installation, extract and run C-SSH/C-SSH.exe.
- Portable upgrade from 0.9.0 or later: exit, back up the old EXE and old Setup in the adjacent Windows folder, preserve data in place, then adopt the new EXE and let it release matching Setup. A stale Setup is rejected rather than silently overwritten.
- Never replace a running EXE or overwrite/delete data. Handle historical MSI installations using their original installation method.

## Apple Silicon Mac

Open C-SSH_0.9.6_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. Mac automatic updates are deferred.

## Android arm64

Install C-SSH_0.9.6_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The existing release signer is retained; The final revision3 APK retains the existing release signer; preserve data when replacing a compatible same-signer installation. Compatible earlier builds with the same signer can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Linux x86_64 / ARM64

- Choose an amd64 or arm64 deb for your CPU and install through the Debian/Ubuntu package manager.
- Make the AppImage executable before launching; Mounted launch needs FUSE; Current package verification is described below; direct AppImage startup requires FUSE. The aarch64 AppImage is the ARM64 build.
- Validated on Ubuntu 24.04.5 with X11 and a desktop Secret Service such as GNOME Keyring. Other distributions and Wayland remain unvalidated.
- Keep the existing data directory and system keyring during upgrades. Do not substitute copying encrypted databases from another device for migration. Linux uses manual downloads in this release.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.


## Data and verification

The product version remains 0.9.6 with SQLite 26. Update in place with the matching platform package while preserving existing data and OS Keychain/keystore entries. Mac is ad-hoc signed and not notarized. Announcement and manual download only, with no upgrade reminders or forced updates.

- Direct frontend verification completed 170 checks across 38 layouts for classic/workbench search buttons, the existing panel, shortcuts and narrow windows on Windows, Mac and Linux. All three frontend builds passed.
- All ten release packages have matching source and package receipts. The nine refreshed desktop packages contain the final search interface; all four Windows packages have new actual installed identities and valid signatures. Mac is arm64, ad-hoc signed and not notarized.
- Agent acceptance covers three Linux nodes through Mac, native ARM64 and compatible x64 execution on Windows 11 ARM64, and both Linux VM architectures. The search-only change reuses the unchanged Rust-flow evidence; it does not imply a new full-function test of every package or physical Windows x64 acceptance.
- The Android arm64 release APK retains the existing signer. Its normal UI completed existing-Agent reuse, readiness, credential persistence and monitor loading on the SDK Emulator. This does not imply physical-device or ordinary host-deletion acceptance. iOS source and frontend are synchronized only and are not uploaded.
