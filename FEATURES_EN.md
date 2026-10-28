# C-SSH 0.9.7 features

[Home](README_EN.md) · [中文](FEATURES.md) · [Installation](INSTALL_EN.md)

Windows x64/ARM64, Apple Silicon Mac, Linux x86_64/ARM64, and Android arm64 share the core runtime, with desktop and mobile layouts. Version 0.9.7 uses notifications and manual downloads only. iOS remains paused.

## AI workspace

Global, project, and host scopes; provider accounts and custom endpoints; model-window settings; permission modes and operation confirmation. Independent read-only tasks can run concurrently, including on the same host, with additional work queued.

Responses, subtask progress, and available results appear incrementally. A concise activity summary expands into tool, command, and file details. Interjections, pause, stop, and resume have clearer states. Context follows the configured model window, while long-history and tool-result handling preserve original local records.

The usage panel shows cumulative tokens, context usage, and cache hit rate. Cumulative usage includes requests and subtasks; context represents one main-assistant input. Missing provider reports remain unknown, and cached input still occupies context.

## Desktop server browser

Windows, Mac, and Linux use local Google Chrome with traffic and DNS handled through the host Agent. Chrome availability is checked before launch. Multiple environments per host retain separate browsing data, stable seeds, and settings.

Configure OS presentation, timezone, language, and permissions while retaining normal browsing features. Launch, verification-page stability, and recovery have improved. Websites may still require verification; complete fingerprint concealment is not promised. The Android entry remains hidden while mobile validation is paused.

## Hosts, connections, and terminals

Add, edit, search, favorite, and group hosts using IPv4, domains, separate ports, passwords or keys, and proxy routes. Existing IPv6 hosts retain their supported connection paths. Agent and plain SSH modes remain distinct; SSH-only hosts do not receive automatic Agent installation.

Linux persistent terminals use tmux, and Windows uses Agent Terminal2/psmux. Desktop clients provide multiple tabs and independent RDP windows. Android host cards open the selected host terminal, allow switching during connection waits, and reject late results from the previous target.

## Monitoring, files, and administration

CPU, memory, disks, networking, history charts, and top processes. Collection concurrency is configurable up to 1000 with queuing and independent scheduling. Database improvements and selective compression reduce repeated work and traffic; throughput depends on the computer and network.

Browse, transfer, edit, create, rename, delete, and verify files. System, process, firewall, container, and service operations follow host capabilities and permissions. Port forwarding, command libraries, broadcast execution, and access grants are desktop features; Android has no forwarding or broadcast pages.

## Agent and Windows management access

Improved Agent reuse, multi-device authorization, readiness, and update messages. Linux x86_64/ARM64 and Windows servers use their corresponding protocol backends. Improves Agent connectivity and administration on Kylin Linux Advanced Server V10 ARM64 with 64KiB memory pages, including initial independent-channel failures in specific SSH configurations.

Clients can save the embedded Windows Setup ZIP, with system sharing on Android. Install it manually on the Windows server and then connect using a Windows account. Plain RDP does not require Setup; there is no OCR-driven or unattended installation.

## Cloud and data protection

Local operations work without signing in. Choose official or self-hosted HTTPS Cloud, with separate sign-in, source, and sync state. Sync is manual and transfers selected hosts, proxy routes, and model configuration with sensitive fields encrypted by the client. AI conversations and long-term memory stay local.

Valid data-protection verification is reused within the current process and required again after reopening. Select-all and cloud-default choices simplify sync. AI deletion records without a corresponding local account no longer clutter the list; existing local conflicts retain an explicit choice.

## Interface and platform boundaries

Classic and workbench views retain appearance preferences, with settings organized into four groups. Desktop window controls, themes, and dialogs follow the Mac design baseline. Android improves safe areas, keyboard layout, and usage displays.

Mac supports Apple Silicon and remains ad-hoc signed without notarization. Mobile and desktop entry points differ; iOS is paused. Diagnostics are disabled by default. Validation is environment-specific and does not guarantee every failure or distribution combination. See the [changelog](CHANGELOG_EN.md) for details.
