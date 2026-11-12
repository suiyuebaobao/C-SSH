# C-SSH 0.9.9 and self-hosted Creation Cloud

[Home](README_EN.md) · [中文](SELF_HOSTING.md) · [Installation and data](INSTALL_EN.md)

Creation Cloud is an independent control plane for accounts, devices, user-initiated encrypted configuration sync, model catalogs, notices, and downloads. It does not proxy client-to-server SSH, terminal, or Agent traffic. Local operations do not require Cloud sign-in.

## This release's update policy

**Version 0.9.9 only notifies users and directs them to a manual download. It does not force upgrades or download/install in the background.** The official update policy remains disabled; notices and Downloads provide the entry points. Android only supports version notices and manual downloads. iOS development and distribution remain paused.

Self-hosted operators may publish a notice linking to official downloads. When hosting official packages, preserve filenames, architectures, sizes, SHA256 values, and original signatures. Use new releases or revisions rather than replacing published records or assets.

## Download assets and signatures

0.9.9 publishes exactly three manual downloads: Windows x64 ZIP, Mac arm64 DMG and Android arm64 APK. The updated server accepts that version-specific complete set without making it eligible for automatic updates. Linux stays on 0.9.8; previous installers and releases remain. Register the original Windows ZIP updater signature through the normal admin upload flow; do not replace the official signing root. `GET /api/v1/downloads/releases` returns actual file identities and sources.

## Data and migrations

Signing in does not decrypt data or start sync automatically. Hosts, proxy routes, and AI configuration are synced manually with sensitive fields encrypted by the client. AI conversations, original memory, terminal output, and command history remain outside Cloud sync.

Account and data-protection passwords have separate purposes. A valid data-protection verification is reused within one client process and requested again after reopening. Preserve existing data and OS keys; do not clear storage or remove keys to address sync errors.

Clients use SQLite28 in 0.9.9. Cloud PostgreSQL migrations have their own version sequence. Do not treat the client release number or SQLite schema as the server schema version. Preserve your configuration, keys, and data, and use the normal forward migration process without rewriting old migrations or database checksums.

## Public source boundary

The repository's `creation-cloud/` directory contains the server production-source mirror. Use the current `main` directory and its corresponding commit. Client Release tags archive client distribution material; they do not imply that every tag contains the latest server source.

The mirror includes the production Cargo dependency closure, migrations, current Nuxt build manifests and source, and required sanitized public resources. Client, shared client core, AI/Agent source, production configuration, deployment tools, tests, runtime data, and secrets are excluded.

Operators supply HTTPS or explicitly enabled private-network HTTP, PostgreSQL, mail, and persistent storage. This is not a one-click deployment carrying the official production environment. Do not copy another operator's sessions, keys, databases, or environment files.

## Optional private-network HTTP in 0.9.8

In Settings → Cloud service → Self-hosted cloud, enter an IP and port, test the connection, review the protocol and confirm the switch. An address without a protocol is tested over HTTPS first. Only private IPs with a recognized protocol-unavailable error may fall back to HTTP. Explicit or saved HTTPS addresses and certificate errors never trigger an automatic downgrade.

The server defaults to HTTPS. To enable HTTP on a trusted private deployment, explicitly set the following options; replace the example address with your own private IP:

```text
CLOUD_INTRANET_MODE=1
CLOUD_INTRANET_TRANSPORT=http
CLOUD_PUBLIC_BASE_URL=http://192.168.1.50:8080
CLOUD_EMAIL_CODE_HMAC_KEY_FILE=/run/secrets/cloud-email-code-hmac-key
```

The key file must contain an independent 64-byte hexadecimal value with no newline and restricted read permissions. Never reuse another deployment's key. Mail is configured independently; missing SMTP does not bypass account verification requirements. HTTP uses a separate cookie namespace and retains HttpOnly, SameSite and CSRF checks, but credentials and sessions are not encrypted in transit. Use it only on a trusted private network.

This server update also fixes account deletion when hosts, AI configuration, proxy profiles or synchronization history reference its devices. Dependencies and the account are handled in one transaction; failures roll back and other accounts are preserved. Retain your database, keys and deployment configuration when upgrading.

## Device limits and server schema

The current mirror includes PostgreSQL schema 63, stable device identity and global/per-account administrator limits. The default is 16 and can be changed by administrators; unknown legacy devices are not guessed together. Official HTTPS policy remains; private-network HTTP requires explicit self-hosted configuration.
