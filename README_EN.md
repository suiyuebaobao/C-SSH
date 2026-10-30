# C-SSH 0.9.8

[中文](README.md) · [Website](https://c-ssh.com) · [Installation](INSTALL_EN.md) · [Features](FEATURES_EN.md) · [Self-hosting](SELF_HOSTING_EN.md) · [Changelog](CHANGELOG_EN.md)

This release updates Windows, macOS, Linux and Android, together with the C-SSH Agents. It focuses on host connection errors, self-hosted Cloud access, Android session recovery, host deletion and diagnostic overhead.

## 1. Clearer host connection errors

When adding a host, connection failures now appear in a dialog that matches the C-SSH interface. Messages distinguish address resolution, network connectivity, SSH handshake, authentication and proxy authentication failures so that users can identify which part of the connection needs attention.

Closing the dialog preserves the entered connection details. An authentication rejection is not automatically labeled an incorrect password: the username, key and server authentication policy may also be responsible.

## 2. Separate hosts at the same IP address

Fixes hosts with the same address but different SSH ports being treated as duplicates during creation or synchronization. Connection endpoints are identified by both address and SSH port; their host records, credentials, trust records and Cloud mappings remain separate.

Existing host identities are preserved. Adding a different port does not require deleting the original host or replacing its credentials.

## 3. Automatic protocol detection for self-hosted Cloud

In Settings → Cloud service → Self-hosted cloud, enter an IP address, domain or address with a port. Test the connection to see the detected protocol, then save and switch. A protocol prefix is optional for a new address.

Detection tries HTTPS first. An explicitly configured, trusted private-network deployment may use HTTP without importing certificates on every client. The interface displays the selected protocol and warns that HTTP traffic is not encrypted.

Explicit or previously saved HTTPS addresses, certificate verification failures and public-network addresses are never silently downgraded to HTTP. Detection sends no existing account credentials or session cookies. The official Cloud continues to use HTTPS. Self-hosted servers must use a compatible server version and explicitly enable the private-network HTTP mode.

## 4. Android AI session recovery after interruptions

Improves recovery when the phone is locked, the network disconnects, connectivity returns or the app returns to the foreground. Addresses blocked follow-up messages, unrecognized task errors and save state that did not recover promptly.

Recovery reconciles the original task with locally saved state, preserves content already received and allows the conversation to continue. It does not automatically replay model requests or remote modification commands. Related save acknowledgements are handled in the correct order to reduce discrepancies between stored conversation content and the displayed state.

## 5. Less unnecessary waiting when deleting hosts

Fixes host deletion waiting on a resource lock already held by the same cleanup operation when the host has server-browser environments. This could leave the interface showing “Deleting” for an unnecessarily long time.

The deletion path now releases the relevant resources before completing browser-environment cleanup. The remote Agent uninstall attempt remains time limited; a failure does not indefinitely block deletion of the local host record.

## 6. Lower Agent diagnostic overhead

When diagnostics are disabled, recording work is skipped earlier. The Linux Agent summarizes normal, short monitoring operations while retaining the necessary details for errors, slow operations and other activity that needs diagnosis.

Log maintenance uses bounded periodic work instead of heavier cleanup on request paths. The Windows Agent receives the corresponding maintenance scheduling changes. These changes target diagnostic recording and maintenance costs; they do not reduce monitoring frequency or change logging settings for other server programs.

## 7. Cloud administrator account deletion

Fixes deletion being blocked by device references when an account contains hosts, AI configurations, proxy profiles or synchronization history.

The account and its dependent synchronization records are handled in one database transaction. Failures roll back completely, and other accounts remain unchanged. Administrator protections, global history and audit requirements remain in place. The official Cloud has received this backend fix; self-hosted deployments need the updated server implementation.

## Versions and installation

- Windows, macOS, Linux and Android clients, and C-SSH Agents: 0.9.8.
- This release announces the update and directs users to download and install it manually. It does not force updates or download and install them automatically.
- Keep the existing installation identity and local data. Do not uninstall, clear app data or remove keys as an update step.
- macOS remains available for Apple Silicon. The app is not notarized with an Apple Developer ID; this installation limitation is unchanged.
- iOS development and releases remain paused. The mobile server-browser entry remains hidden.

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.8_arm64-setup.exe | 55,120,724 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_arm64-setup.exe) |
| C-SSH_0.9.8_portable-Windows-arm64.zip | 50,865,532 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_portable-Windows-arm64.zip) |
| C-SSH_0.9.8_portable-Windows-x64.zip | 52,359,569 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_portable-Windows-x64.zip) |
| C-SSH_0.9.8_x64-setup.exe | 57,113,796 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_x64-setup.exe) |
| C-SSH_0.9.8_macOS-arm64.dmg | 55,033,590 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_macOS-arm64.dmg) |
| C-SSH_0.9.8_android-arm64.apk | 53,086,313 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_android-arm64.apk) |
| C-SSH_0.9.8_aarch64.AppImage | 127,085,064 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_aarch64.AppImage) |
| C-SSH_0.9.8_amd64.AppImage | 128,088,568 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_amd64.AppImage) |
| C-SSH_0.9.8_amd64.deb | 54,934,216 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_amd64.deb) |
| C-SSH_0.9.8_arm64.deb | 54,041,416 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.8/C-SSH_0.9.8_arm64.deb) |

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.8_arm64-setup.exe | `e79dc574d0278dc419bdc130eaf4201afd33b31029f40dbccf0fe222f475cbcb` |
| C-SSH_0.9.8_portable-Windows-arm64.zip | `c499806e7d5bef2a48691e083fb24ec789235ae2e939da24e3b3bffd77cd00cd` |
| C-SSH_0.9.8_portable-Windows-x64.zip | `9b926ad08286e5c7fefbbb9eed40293ad45f8a0f036af7de0922201610a40941` |
| C-SSH_0.9.8_x64-setup.exe | `e526564486f14f84aec0ab265cd7eb92c892d5dfbc759e62461548eefccc8e82` |
| C-SSH_0.9.8_macOS-arm64.dmg | `73a2d6136fb6f4d34aa245984eae008e732152807878349479b25b947e9a1af1` |
| C-SSH_0.9.8_android-arm64.apk | `add46254cfdfc322d575da5f36328822e3b3b97b9c10b1d41f4f5a603f41bda1` |
| C-SSH_0.9.8_aarch64.AppImage | `2873949dfbfda78456b19e1aecdeb8b8a58d134651adfc71b2e4536a32ccff20` |
| C-SSH_0.9.8_amd64.AppImage | `f936a01b9621173f66be8c43ce77c53d6fe9e5206f1b9d52a5453ac5f39cba3b` |
| C-SSH_0.9.8_amd64.deb | `00630c2cc0cb1fdf1d23d80bb83242c85a13e09f977b25b3cb29952ee2c41a20` |
| C-SSH_0.9.8_arm64.deb | `97105e890023f27d8b69f300745f09e248e2eb5542c68668ef2802e4a38ba2e4` |

AI conversations and original memory remain local. Cloud synchronizes selected encrypted configuration. This repository contains documentation, downloads and the Creation Cloud production-source mirror; client and Agent source code is not public.
