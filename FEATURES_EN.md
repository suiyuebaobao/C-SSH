# C-SSH 0.9.9 features

[Home](README_EN.md) · [简体中文](FEATURES.md) · [Installation](INSTALL_EN.md)

Current downloads cover Windows x64, Apple Silicon Mac, Linux x86_64, and Android arm64. Linux now moves to 0.9.9; the other three platforms retain their previously released 0.9.9 files. Desktop interfaces follow the Mac baseline, while mobile uses touch-friendly layouts. Installation is manual. iOS remains paused.

## AI assistant

- Ask questions across all hosts, a project, or one host; configure model accounts, endpoints, and context windows.
- Run independent conversations, switch hosts while preserving progress and results, and stop the selected task.
- See responses and progress as they arrive, with concise summaries and expandable details. Interjections, pause, and resume are supported.
- View cumulative tokens, context usage, and cache hit rate. Cumulative usage includes requests and subtasks; context represents one main-assistant input. Values remain unknown when the provider does not report them.
- Improved long-chat loading, saving, and storage use while keeping original records locally. Operations remain subject to permission modes and confirmation requirements.

## Hosts, terminals, and files

Manage hosts, groups, favorites, and proxy routes through Agent or plain SSH. Use IPv4, domains, and separate ports; existing IPv6 hosts retain their supported connection paths. Plain SSH mode does not install an Agent automatically.

Use ordinary or persistent terminals, browse and transfer files, and edit or manage remote files. Desktop clients provide multiple terminals, separate remote desktop windows, port forwarding, and command broadcasting. Mobile has no forwarding or broadcast pages. Android host cards open the selected host terminal, with ordinary mode used by default for new connections.

## Monitoring and server management

View CPU, memory, disks, networking, history, and processes. Collection concurrency can be set up to 1000, with remaining tasks queued. Actual speed depends on the client, servers, and network.

Process, service, firewall, and container management depends on the target system and permissions. Linux x86_64/ARM64 and Windows servers are supported, with additional compatibility work for Kylin Linux Advanced Server V10 ARM64. Clients can export the Windows server management installer for a server administrator to install manually; ordinary remote desktop access does not require it.

## Desktop server browser

Windows, Mac, and Linux use Google Chrome and prompt when it is missing. Browse through the selected server, with multiple environments per server that retain separate data and settings.

Configure OS presentation, timezone, language, and permissions. Websites may still request verification; complete browser-fingerprint concealment is not promised. The Android entry is currently hidden.

## Cloud accounts and data protection

Local features remain available without signing in. Choose official or self-hosted Cloud, sign in through a dialog, and manage uploaded resources from the client, including adding local items, editing, deleting, and manually downloading.

Hosts, proxy routes, and model configuration sync only through user action. Sensitive information is encrypted; AI conversations and long-term memory are not included in Cloud sync. Valid data-protection verification is reused within the running app and requested again after reopening.

## Interface and platform notes

Appearance and grouped settings are available. Android supports chat text-size controls and refined spacing around content and navigation. Monitoring health is shown separately from Agent deployment status.

Mac supports Apple Silicon and is currently not notarized. This Linux release provides an x86_64 AppImage; older builds for other architectures remain in past releases. See [Installation](INSTALL_EN.md) for requirements and data preservation. Validation is environment-specific and does not cover every system and network combination.
