# C-SSH 0.9.6 Revision 2 installation and data preservation

[Downloads](README_EN.md#downloads) · [中文](INSTALL.md)

This release uses notices and manual downloads only. No upgrade prompt or forced update is sent.

Announcements and manual downloads only: no upgrade reminder or forced update. The product version remains 0.9.6; revision 2 uses the new tag `v0.9.6-r2`. Ten assets cover four platforms; iOS is not uploaded. All earlier releases and original 0.9.6 assets remain intact. Android/iOS build number is 9006000; wire and signing identities are unchanged.

## Before installation

Use the official website or suiyuebaobao/C-SSH v0.9.6-r2 release and verify SHA256. Exit C-SSH and preserve important data. Cloud explicitly syncs encrypted configuration, not AI conversations or long-term memory; retain your data-protection password.

## Windows x64 / ARM64

Choose the package for your CPU; native ARM64 was validated on Windows 11 ARM64.

- Run the NSIS installer in the original installation directory. WebView2 Runtime must already be available; the installer has no Authenticode signature.
- For a new portable installation, extract and run C-SSH/C-SSH.exe.
- Portable upgrade from 0.9.0 or later: exit, back up the old EXE and old Setup in the adjacent Windows folder, preserve data in place, then adopt the new EXE and let it release matching Setup. A stale Setup is rejected rather than silently overwritten.
- Never replace a running EXE or overwrite/delete data. Handle historical MSI installations using their original installation method.

## Apple Silicon Mac

Open C-SSH_0.9.6_macOS-arm64.dmg and drag C-SSH.app to Applications. Preserve Application Support data and Keychain entries when replacing the app. Only Apple Silicon is provided; minimum target macOS13. The app is ad-hoc signed and not notarized; follow [Apple guidance](https://support.apple.com/102445) for its system permission. Mac automatic updates are deferred.

## Android arm64

Install C-SSH_0.9.6_android-arm64.apk with the system installer. Only arm64 is supplied; minimum target Android7/API24. The existing release signer is retained; this round directly tested original 0.9.6 same-version/same-code replacement. Compatible earlier builds with the same signer can be replaced while keeping data, without uninstalling. Earlier builds with a different signer cannot be replaced directly. Uninstalling clears local data, including AI history and memory not covered by Cloud sync; preserve it before considering uninstall.

## Linux x86_64 / ARM64

- Choose an amd64 or arm64 deb for your CPU and install through the Debian/Ubuntu package manager.
- Make the AppImage executable before launching; Mounted launch needs FUSE; this round exercised direct FUSE AppImage cold startup on both Ubuntu VM architectures. The aarch64 AppImage is the ARM64 build.
- Validated on Ubuntu 24.04.5 with X11 and a desktop Secret Service such as GNOME Keyring. Other distributions and Wayland remain unvalidated.
- Keep the existing data directory and system keyring during upgrades. Do not substitute copying encrypted databases from another device for migration. Linux uses manual downloads in this release.

## Windows server management Setup

The DVC host form in every released client exports embedded Setup ZIP; Android also offers system sharing. Extract on the Windows server and manually run C-SSH-Windows-Setup.exe with normal UAC confirmation. The client never executes the EXE or exports host/account/password data. Existing components can be used directly; RDP-only mode needs no Setup.

Enter an IPv4 address or domain directly, with a separate port. See [self-hosted Cloud](SELF_HOSTING_EN.md). No public iOS download is supplied.


## Data and verification

This revision uses SQLite 26. Exit, preserve complete data and keep existing OS Keychain/keystore entries before replacing the program in place. Do not manually move, delete or export raw keys, clear data, or uninstall to work around startup/signing issues. Normal 22→26 and the validated known 25 data were exercised. Other 23–25 data is accepted only for strictly known shapes with trust preserved. Once data adopts 26, do not open it with original 0.9.6 schema 22 binaries. A rollback requires the matching older program and a complete pre-upgrade backup.

- Ten artifacts total 583,805,859 bytes. Sizes and SHA256 come from current artifact and transfer receipts. Unchanged Mac r26/Windows r11 SSH/SFTP, monitor switching, navigation and UI-R57 deletion evidence is reused within its original scope.
- Mac arm64 reuses the first candidate’s actual 25→26 migration evidence with unchanged Store inputs. Final revision 2 starts under the ordinary identity on data already migrated to schema 26 and passes the existing Cloud page, window controls and close. A real-model text follow-up retained prior information; stopping occurred before provider context with tool=0, so incomplete tool rounds were not tested against a real model. Mac remains ad-hoc signed and unnotarized.
- Windows 11 ARM VM: native ARM64 and emulated x64. Release binaries passed 22→26 preservation, workbench and same-conversation stop/follow-up with an HTTP fixture. A new ordinary-user installer-only lane passed original 0.9.6 same-version replacement, actual EXE identities, data and registration preservation. All four updater epoch3 signatures were verified; no Authenticode. New-identity Session0 close/exit_app natural shutdown failures remain recorded; ARM64 raw separately passed Session1 close/destroyed. This is not a full formal-runtime pass.
- The signed arm64 APK passed same-signer/same build number 9006000 replacement, 22→26 preservation of 51 Required tables and preferences/wrapping data, plus fresh workbench/system appearance on a real Android SDK Emulator. AI entry only; no physical phone or general CleanShutdown validation.
- Both Linux architectures passed 22→26 preservation of 57 tables and 12 old rows, followed by direct FUSE AppImage cold startup after deb installation and real business views in Ubuntu VMs. Native Linux AI follow-up was not tested without independent model credentials.
- Incomplete tool rounds, unique complete results and exact coverage watermarks passed 3 shared integration tests / 4 real local HTTP scenarios; Windows native validation used a controlled HTTP fixture. This does not imply real-model or remote-tool validation on every platform.
- iPhone Simulator version/entry, fresh workbench/system and the two notification categories were exercised; no public upload. Physical devices, minimum-OS execution, other distributions/Wayland, weak networks and untested remote workflows are not implied.
