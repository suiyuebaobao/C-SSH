# C-SSH 0.9.1 installation and data preservation

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release uses notices and manual downloads only. No upgrade prompt or forced update is sent.

## Before installation

Use the official website or suiyuebaobao/C-SSH v0.9.1 release and verify SHA256. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64

- Run the NSIS installer in the original installation directory. WebView2 Runtime must already be available; the installer has no Authenticode signature.
- For a new portable installation, extract and run C-SSH/C-SSH.exe.
- Portable 0.9.0 upgrade: exit, back up the old EXE and old Setup in the adjacent Windows folder, preserve data in place, then adopt the new EXE and let it release matching Setup. A stale Setup is rejected rather than silently overwritten.
- Never replace a running EXE or overwrite/delete data. Handle historical MSI installations using their original installation method.

## Apple Silicon Mac

Open C-SSH_0.9.1_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. Mac automatic updates are deferred.

## Android arm64

Install C-SSH_0.9.1_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The 0.9.0 signer is retained, so 0.9.0 can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.
