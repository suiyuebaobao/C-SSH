# Connecting to a self-hosted Creation Cloud

[Home](README_EN.md) · [中文](SELF_HOSTING.md)

## Responsibilities

Creation Cloud provides accounts, devices, model catalogs, explicit encrypted sync, announcements, releases and downloads. It does not relay SSH, terminal, file or Agent traffic. The client defaults to the official service but can use your deployment of the same server; local features do not require login.

The repository's `creation-cloud/` directory is a filtered production-source mirror. The operator must configure and maintain the database, SMTP, HTTPS, administrators and runtime secrets. It does not include the official production configuration or secrets.

## Client setup

1. Deploy a working service with a trusted HTTPS certificate. Set `CLOUD_PUBLIC_BASE_URL` to its public root, such as `https://cloud.example.com`.
2. Select Self-hosted Cloud under Settings → Cloud Service and enter the same HTTPS root.
3. Test, confirm the switch, then sign in to that service. URLs may not contain credentials, paths, queries or fragments; certificate verification is not disabled.
4. The official service and separate self-hosted services have isolated login/sync state. Switching does not move local hosts, credentials or chats. Resolve pending sync as instructed before switching.
5. Sync still requires explicit preview, selection and confirmation. The data-protection password differs from the login password; AI conversations and long-term memory are excluded.

## Release distribution

**0.9.2 requires manual installation. Older versions cannot automatically upgrade to it.** Do not force this upgrade on clients with the previous signing identity.

For later compatible upgrades, download original official artifacts, preserve their bytes/signatures, upload them through your Cloud administration interface and configure release policy. Clients check and download from the currently selected Cloud rather than a separate official endpoint.

- Windows requires the original updater signature. The official `GET /api/v1/downloads/releases` manifest exposes `file_name`, `sha256` and `updater_signature`. Save the matching non-empty signature unchanged as a UTF-8 `.sig` and upload it with the package. Do not replace it with your own signing key.
- Upload the original Android APK and preserve its installation signature. Signing changes still require the official manual-installation and data-preservation procedure.
- Mac currently uses a manually downloaded DMG without an automatic-install contract. iOS is not distributed in this release.
- Verify version, platform, architecture, SHA256, source and real downloads. Never replace a published asset, hash or signature under the same identity.

Operators own certificate, backup, mail, availability and permission management. Clients still control end-to-end decryption; a server upgrade does not perform a user's decryption or migration.

The official 0.9.2 channel uses notices and manual downloads only, with no upgrade prompt or forced update. Self-hosted operators manage their own notices and releases.

The current Cloud updater asset contract covers Windows x64 and Android. Windows ARM64, Mac and Linux use manual download links in this release; ARM64 is never mapped to x64.
