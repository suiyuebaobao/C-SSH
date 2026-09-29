# Installing C-SSH 0.9.0 and preserving data

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

> **Download and install this release manually. Automatic upgrades from older versions are unavailable.**

## Before installing

- Download only from the website or the `suiyuebaobao/C-SSH` v0.9.0 release and compare SHA256 with the README.
- Exit C-SSH before installing or replacing its executable.
- Preserve important data first. Cloud explicitly syncs encrypted host, proxy and model-account settings; **AI conversations and long-term memory are excluded**.
- Keep the data-protection password. It is separate from the website login password and local device key.

## Windows x64

1. Use `C-SSH_0.9.0_x64-setup.exe` for installation, or extract the portable ZIP and run `C-SSH/C-SSH.exe`.
2. WebView2 Runtime must already be installed. The installer has no Authenticode signature, so Windows may show an unknown-publisher warning.
3. Use the existing installation directory or retain the portable program's adjacent `data` directory. Never overwrite data with an empty directory or replace an executable while it is running.
4. This release has a different updater signing identity from older official builds. Run the new installer manually or replace the portable program; the old automatic updater cannot perform this upgrade.
5. Older MSI installations need their original installation procedure and data backup before switching package type. Do not mix installers in place.

To manage a Windows server for the first time, obtain `Windows/C-SSH-Windows-Setup.exe` from the Windows client directory, copy it to the server and run it manually with the required system confirmation. Then connect through the client's Windows-account workflow. The portable program materializes Setup from its embedded resources on first launch. Each client does not need to reinstall the server components.

## Apple Silicon Mac

1. Download `C-SSH_0.9.0_macOS-arm64.dmg`, open it and drag `C-SSH.app` into Applications.
2. Only M-series Apple Silicon is provided; the minimum target is macOS 13. There is no Intel/Universal build.
3. The app is ad-hoc signed and **not notarized by Apple**. After confirming the official source and hash, use [Apple's guidance](https://support.apple.com/en-us/102445) to handle the app-specific opening permission in System Settings → Privacy & Security.
4. Application Support holds local data and Keychain protects keys. Do not remove them when replacing the app. Handle any Keychain permission prompt locally.
5. The Mac client has no automatic updater in this release.

## Android arm64

1. Download `C-SSH_0.9.0_android-arm64.apk` and install it through Android's installer, granting the downloading app installation permission if requested.
2. The APK is arm64 only and targets Android 7.0/API 24 or later. No AAB or x86 release build is provided.
3. **The signature differs from older official builds, so 0.9.0 cannot replace them directly.** Do not repeatedly reinstall or clear data when Android reports a conflict.
4. **Uninstalling the old app clears its local data.** Confirm independent backups first. Cloud excludes AI conversations and memory; keep the old app if you cannot verify that needed data is preserved.
5. Same-signer test-build upgrades were checked in SDK Emulator. That result does not apply to the older official signing identity.

## First use

Select IPv4 or Domain when adding a host, omit URL schemes and ports from the address, and enter the port separately. Confirm SSH host keys or RDP certificates. Cloud login is optional; configure a self-hosted service under Settings → Cloud Service. See the [self-hosting guide](SELF_HOSTING_EN.md).

iOS has no IPA, TestFlight or download in this release. Use [Issues](https://github.com/suiyuebaobao/C-SSH/issues) or [website feedback](https://c-ssh.com/en/feedback), without passwords, private keys, tokens or real conversation contents.
