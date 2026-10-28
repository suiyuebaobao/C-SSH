# C-SSH 0.9.7 and self-hosted Creation Cloud

[Home](README_EN.md) · [中文](SELF_HOSTING.md) · [Installation and data](INSTALL_EN.md)

Creation Cloud is an independent control plane for accounts, devices, user-initiated encrypted configuration sync, model catalogs, notices, and downloads. It does not proxy client-to-server SSH, terminal, or Agent traffic. Local operations do not require Cloud sign-in.

## This release's update policy

**Version 0.9.7 only notifies users and directs them to a manual download. It does not force upgrades or download/install in the background.** The official update policy remains disabled; notices and Downloads provide the entry points. Android only supports version notices and manual downloads. iOS development and distribution remain paused.

Self-hosted operators may publish a notice linking to official downloads. When hosting official packages, preserve filenames, architectures, sizes, SHA256 values, and original signatures. Use new releases or revisions rather than replacing published records or assets.

## Assets and signatures

- The manual set has ten assets: Windows x64/ARM64 NSIS and portable ZIP, Mac arm64 DMG, Android arm64 APK, and Linux x86_64/ARM64 deb and AppImage. There are no new MSI, AAB, or iOS assets.
- Starting with 0.9.7, the current server supports a six-asset canonical distribution set: Windows x64 EXE/ZIP, Android APK, the Mac update archive, and both Linux AppImages. It also accepts the complete manual set plus the Mac update archive. Storing that archive does not activate automatic updates.
- The official `GET /api/v1/downloads/releases` manifest provides filenames, targets, sizes, hashes, and `updater_signature` for applicable assets. Save signatures unchanged as UTF-8 `.sig` files and upload them with the matching packages. Do not replace official signatures with your own key.
- Do not label Windows ARM64, Mac DMG, or Linux deb as a different update target. Android does not gain in-app installation capability.
- Keep the 0.9.7 update policy disabled, without forced or disabled versions, and use a notice to let users decide when to download. Uploading files is not authorization to enable a rollout.

## Data and migrations

Signing in does not decrypt data or start sync automatically. Hosts, proxy routes, and AI configuration are synced manually with sensitive fields encrypted by the client. AI conversations, original memory, terminal output, and command history remain outside Cloud sync.

Account and data-protection passwords have separate purposes. A valid data-protection verification is reused within one client process and requested again after reopening. Preserve existing data and OS keys; do not clear storage or remove keys to address sync errors.

Clients use SQLite27 in 0.9.7. Cloud PostgreSQL migrations have their own version sequence. Do not treat the client release number or SQLite schema as the server schema version. Preserve your configuration, keys, and data, and use the normal forward migration process without rewriting old migrations or database checksums.

## Public source boundary

The repository's `creation-cloud/` directory contains the server production-source mirror. Use the current `main` directory and its corresponding commit. Client Release tags archive client distribution material; they do not imply that every tag contains the latest server source.

The mirror includes the production Cargo dependency closure, migrations, current Nuxt build manifests and source, and required sanitized public resources. Client, shared client core, AI/Agent source, production configuration, deployment tools, tests, runtime data, and secrets are excluded.

Operators supply HTTPS, PostgreSQL, mail, and persistent storage. This is not a one-click deployment carrying the official production environment. Do not copy another operator's sessions, keys, databases, or environment files.
