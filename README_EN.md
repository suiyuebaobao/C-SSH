# C-SSH 0.9.9

[简体中文](README.md) · [Website](https://c-ssh.com/en) · [Install](INSTALL_EN.md) · [Features](FEATURES_EN.md) · [Self-hosting](SELF_HOSTING_EN.md) · [Changelog](CHANGELOG_EN.md)

This release provides a Windows x64 portable ZIP, a macOS Apple Silicon DMG, and an Android arm64 APK. Linux remains on 0.9.8, and iOS development remains paused. Updates are announced for manual download and installation; no automatic or forced installation is enabled.

## AI conversations and local storage

- Run independent AI conversations and switch between hosts while work continues. Each conversation retains its progress, results, and run state. Stop applies to the selected task.
- Improve scrolling and presentation during AI execution. Compact progress summaries keep the conversation readable, with parameters, results, and worker details available on demand.
- Store message deltas, reuse identical bodies through references, and apply selective lossless Zstd compression. This reduces repeated full-history writes and excessive growth in global and project conversations while preserving event identities, order, sources, and original content.
- Load recent history first and fetch earlier messages as needed. Display paging does not delete original memory or reduce the model's context window. Local compression does not automatically reduce billed tokens.
- Fix races between reading older history and saving new results. Stale page responses no longer report that the current conversation failed to save. Android now distinguishes history-read errors from actual write failures.
- Improve background conversation recovery and stop settlement without automatically replaying model requests or remote changes.

## Shared web search and page reading

- Provide one search entry across models. Use supplier search when supported by the current account and interface, then use website search when unavailable. The chat model and account are not switched automatically.
- Improve search-source handling and failure transitions, including Bing language negotiation and valid results with empty snippets. Page reading retains address, redirect, and private-network protections.
- Availability still depends on provider capabilities, network conditions, and the destination site. A successful test is not a guarantee of continued availability.

## Cloud accounts and resource management

- Align account entry points and themed login dialogs across Windows, Mac, and Android. Local features remain available without signing in.
- Manage uploaded hosts, AI accounts, and proxy profiles from the client, including adding local resources, editing, deleting, and explicitly downloading. Cloud and local changes remain separate and user initiated.
- Improve account initialization and state refresh to reduce repeated loading and redundant requests.
- Improve device counting across multiple versions on the same computer. Administrators can configure a global device limit and account-specific overrides. Installation data and synchronization state remain separate.
- Keep existing data protection. Plaintext keys are not sent to Creation Cloud; manual sync transfers protected ciphertext and excludes AI conversations and original memory.

## Startup, host access, and Windows upgrades

- Remove custom startup animations and decorative waits while retaining required initialization, migration, and version gates.
- Reuse verified Windows management resources without extracting them again. Replacing only the client EXE can safely advance known 0.9.7/0.9.8 management resources while preserving data. Unknown files and unsafe links are not overwritten.
- Improve diagnosis and controlled handling of rejected dedicated SSH keys and disabled account public-key authentication during Agent installation or repair. Existing credentials and server business data remain preserved.
- Bundle version 0.9.9 Linux x64/ARM64 Agents, Windows Agent, and Windows management Setup. Protocols and supported architectures remain unchanged. The server management Setup remains available even though new Windows client installers are paused.

## Android refinements

- Reduce excessive spacing above content and below navigation, and improve the login dialog layout.
- Select AI conversations through the existing host/scope selectors and history. Remove the separate conversation-tab row while retaining background conversations.
- Adjust and persist AI chat text size from settings.
- Open new terminals and newly selected hosts in ordinary terminal mode by default. Persistent mode remains an explicit choice.
- Derive host status dots from the same monitoring status as the text. A healthy monitoring result does not imply an installed Agent.
- Mobile server-browser entry points remain hidden.

## Downloads and installation

- Windows: x64 portable ZIP only. Exit the old client, extract the ZIP, replace C-SSH.exe in the existing application directory, and retain its data folder. Do not run directly from the archive.
- Mac: Apple Silicon arm64 DMG, retaining the application and data identities. The app remains ad-hoc signed and is not notarized; no Intel build is provided.
- Android: arm64 APK with the existing release signing identity. Install over the existing app without uninstalling or clearing data.
- This release advances local storage to SQLite schema 28. Allow first-run migration to finish; do not open upgraded data with an older client.
- Linux was not rebuilt for this release. The download page identifies its latest published version separately.

## Verification

Previously passed, unchanged functionality was reused. Windows checks covered replacing the 0.9.8 EXE, management-resource replacement, preservation of groups/settings/original history, new writes, and reopening; focused Win32 file tests passed. Android API 36 arm64 SDK Emulator checks covered same-signer installation, installation identity and existing data retention, new writes, and reopening. Mac checks covered the current build, signature, embedded resources, and DMG mounting, alongside prior functional evidence. These results do not claim that every physical device, operating-system version, or network was retested.

## Added

Independent AI sessions, client Cloud resource management, administrator device limits and Android chat font controls.

## Fixed

History/save races, repeated storage, account initialization, Bing search, Windows management-component upgrades, Android spacing and status display; see details above.

## Verified

See the validation scope and limitations above.

## Downloads

| File | Bytes | Download |
|---|---:|---|
| C-SSH_0.9.9_portable-Windows-x64.zip | 46,581,351 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9/C-SSH_0.9.9_portable-Windows-x64.zip) |
| C-SSH_0.9.9_macOS-arm64.dmg | 48,915,525 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9/C-SSH_0.9.9_macOS-arm64.dmg) |
| C-SSH_0.9.9_android-arm64.apk | 46,778,053 | [Download](https://github.com/suiyuebaobao/C-SSH/releases/download/v0.9.9/C-SSH_0.9.9_android-arm64.apk) |

Linux stays on [0.9.8](https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.9.8); no Linux client is released in this batch.

## SHA256

| File | SHA256 |
|---|---|
| C-SSH_0.9.9_portable-Windows-x64.zip | `f8e7b58ae96a6fcb603bd16af9d30090d0ce9d8defdd7f369ff4e9f157c93c34` |
| C-SSH_0.9.9_macOS-arm64.dmg | `6d6c5b49a1e3d052c41c63345ec874699b340073ba4035b4b148bedf7b6dc8ad` |
| C-SSH_0.9.9_android-arm64.apk | `960bba3cd79935f7bf46410d8ddd402dc70650d521c378616b8677c688d6bd02` |
