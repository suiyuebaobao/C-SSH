# C-SSH 0.9.6 — Revision 3

[中文](README.md) · [Website](https://c-ssh.com) · [Installation](INSTALL_EN.md)


A server-operations client for Windows, Apple Silicon Mac, Android and Linux. Local SSH/Agent operations do not require Cloud login.

Announcements and manual downloads only, with no upgrade reminder or forced update. Version0.9.6, SQLite26 and build9006000 are unchanged. Revision3 uses the independent tag `v0.9.6-r3`; ten packages cover four platforms, iOS is not uploaded, and original0.9.6, revision2 and historical downloads remain available.

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.6_x64-setup.exe | 47,912,113 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_x64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-x64.zip | 43,180,565 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-x64.zip) |
| C-SSH_0.9.6_arm64-setup.exe | 46,045,545 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64-setup.exe) |
| C-SSH_0.9.6_portable-Windows-arm64.zip | 41,861,101 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_portable-Windows-arm64.zip) |
| C-SSH_0.9.6_macOS-arm64.dmg | 42,822,796 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_macOS-arm64.dmg) |
| C-SSH_0.9.6_android-arm64.apk | 39,899,610 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_android-arm64.apk) |
| C-SSH_0.9.6_amd64.deb | 44,271,530 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.deb) |
| C-SSH_0.9.6_amd64.AppImage | 117,750,264 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_amd64.AppImage) |
| C-SSH_0.9.6_arm64.deb | 43,422,970 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_arm64.deb) |
| C-SSH_0.9.6_aarch64.AppImage | 116,779,528 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.6-r3/C-SSH_0.9.6_aarch64.AppImage) |

## Added

- A separate revision-3 download record preserves original 0.9.6, revision 2 and historical downloads.

## Fixed

- Adding an Agent host checks an existing Agent for compatibility first. Compatible Agents are reused without uploading, reinstalling or restarting. Missing Agents are installed; confirmed older versions follow the safe upgrade path. Unknown probe outcomes or future versions do not overwrite an installation.
- Identity adoption, READY confirmation and credential persistence appear as separate real stages. A readiness failure is no longer reported as an initial SSH connection failure, and does not automatically replay installation.
- Deployment progress is scoped to the current invocation, host and operation; late events after completion or leaving the view are rejected.
- Windows and Mac workbench close buttons match the adjacent window controls in outline, background and size, while preserving danger hover and the existing exit action. Linux retains its normal system title bar.
- Classic and workbench search entry points on Windows, Mac and Linux default to a magnifying-glass icon. Clicking opens the existing search panel, and the existing shortcuts are retained. The terminal toolbar wraps in narrow windows to keep search and connection controls from overlapping.

## Verified

- Direct frontend verification completed 170 checks across 38 layouts for classic/workbench search buttons, the existing panel, shortcuts and narrow windows on Windows, Mac and Linux. All three frontend builds passed.
- All ten release packages have matching source and package receipts. The nine refreshed desktop packages contain the final search interface; all four Windows packages have new actual installed identities and valid signatures. Mac is arm64, ad-hoc signed and not notarized.
- Agent acceptance covers three Linux nodes through Mac, native ARM64 and compatible x64 execution on Windows 11 ARM64, and both Linux VM architectures. The search-only change reuses the unchanged Rust-flow evidence; it does not imply a new full-function test of every package or physical Windows x64 acceptance.
- The Android arm64 release APK retains the existing signer. Its normal UI completed existing-Agent reuse, readiness, credential persistence and monitor loading on the SDK Emulator. This does not imply physical-device or ordinary host-deletion acceptance. iOS source and frontend are synchronized only and are not uploaded.

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.6_x64-setup.exe | `68c108d74ce246513b87d66a10fb7bd88c2bc729d150394f09b1a1cfdad41de8` |
| C-SSH_0.9.6_portable-Windows-x64.zip | `67e46eae24524e4478c014aaea660e8ad938dafe5269b9059cc8ed73327c95f2` |
| C-SSH_0.9.6_arm64-setup.exe | `b3287cbcb86daac82963015d5abedb2786541c2c1f3e60bac6b2c125831eff0c` |
| C-SSH_0.9.6_portable-Windows-arm64.zip | `ffee1c505cbd0883a20babeb51d9cada3173510c48f3247b897b26bbd85f13f5` |
| C-SSH_0.9.6_macOS-arm64.dmg | `28d6962f39104739030dc192d513ae00f088e1bf32930a979ec887d5c05bf000` |
| C-SSH_0.9.6_android-arm64.apk | `a0245a283b45ed9f1a5b4c35091ff419e1cc9bdf3054d4cef3f2488357c5d239` |
| C-SSH_0.9.6_amd64.deb | `a482d1d7c9e452533c4980790b02f88a6d71bf350bd0a231e47216eeaff76a22` |
| C-SSH_0.9.6_amd64.AppImage | `60ef49c4a3e9ac2abada6ad5f2baa0a6a43cb4a513ed5cc3f472fd8b2e42f218` |
| C-SSH_0.9.6_arm64.deb | `117b9915c92af9eecbac46c9c51b37d5414a751fcfe85339f5f32e6be30c4794` |
| C-SSH_0.9.6_aarch64.AppImage | `6528806c24497e6e696a1a038c0025819295c5cc68b9db959d1376d5f9abff14` |

## Data and source

The product version remains 0.9.6 with SQLite 26. Update in place with the matching platform package while preserving existing data and OS Keychain/keystore entries. Mac is ad-hoc signed and not notarized. Announcement and manual download only, with no upgrade reminders or forced updates.

AI conversations, raw records, Skills and long-term memory remain local and are not Cloud-sync data. Selected model context is sent to the chosen provider. This public repository contains docs, release binaries and the existing whitelisted Cloud source; client/shared-core/AI/Agent sources are not public.
