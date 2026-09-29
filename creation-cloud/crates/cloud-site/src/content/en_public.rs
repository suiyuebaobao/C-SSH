//! 提供英文安全、下载、更新与常见问题内容。

use crate::{FaqItem, PageContent, PageId};

use super::en::{item, page, section};

pub(super) fn security() -> PageContent {
    page(
        PageId::Security,
        "SSH tunnel, host key, and vault security | C-SSH",
        "Learn how C-SSH handles SSH tunnels, host-key verification, the agent local socket, cloud data boundaries, and client-side encrypted vaults.",
        "Set boundaries before adding capability",
        "A clear split between control and SSH data planes",
        "Connections remain direct end to end; the cloud accepts only classified, explicitly allowed data.",
    )
    .with_sections(vec![
        section(
            "transport",
            "Connections and host safety",
            "Important changes stay visible and unfamiliar resources are never modified automatically.",
            vec![
                item("SSH", "Local tunnel", "The client initiates the connection and the agent exposes no public port.", "Minimal exposure"),
                item("Identity", "Host key confirmation", "A changed host key requires explicit confirmation.", "Visible failure"),
                item("Sessions", "Preserve user work", "Remote SSH, tmux, and user processes are never ended without authorization.", "Keep context"),
            ],
        ),
        section(
            "cloud",
            "Cloud data classification",
            "Unknown fields are rejected and sensitive material does not enter the cloud in plaintext.",
            vec![
                item("Sync", "Host and AI manual sync", "Secrets upload only as opaque ciphertext encrypted by a trusted client.", "User confirmed"),
                item("Vault", "Client-side encryption", "The server stores versioned opaque ciphertext and required non-secret metadata only.", "Zero knowledge"),
                item("Logs", "Redacted records", "Passwords, tokens, cookies, ciphertext bodies, and SSH details stay out of logs.", "Minimum record"),
            ],
        ),
    ])
}

pub(super) fn downloads() -> PageContent {
    page(
        PageId::Downloads,
        "Download C-SSH for Windows, Apple Silicon Mac and Android",
        "Download the latest C-SSH SSH terminal and server operations client for Windows, Apple Silicon Mac and Android. The Linux client is discontinued and is not offered for download.",
        "DOWNLOADS / DIRECT",
        "Download C-SSH",
        "0.9.0 requires a manual download and installation. Automatic updates from older versions are unavailable. Read the data-preservation instructions first.",
    )
    .with_sections(vec![section(
        "builds",
        "Choose your platform",
        "Only actionable download buttons are shown; platforms not developed yet keep a short status.",
        vec![
            item("Desktop", "Windows", "Available package buttons start the download directly.", "Awaiting release data"),
            item("Mobile", "Android", "Available package buttons start the download directly.", "Awaiting release data"),
            item("Planned", "macOS", "DMG for Apple Silicon only; ad-hoc signed and not notarized. Manual installation required.", "Not published"),
            item("Planned", "iOS", "The iPhone client remains in development and is not included in this release.", "Not published"),
        ],
    )])
}

pub(super) fn changelog() -> PageContent {
    page(
        PageId::Changelog,
        "C-SSH changelog | Releases and feature updates",
        "Review C-SSH release dates, feature updates, and supported platforms; download files and SHA256 details stay on the download page.",
        "CHANGELOG / RELEASE HISTORY",
        "Changelog",
        "Track what changed, when each release shipped, and which platforms were included.",
    )
    .with_sections(vec![
        section("latest", "Recent releases", "This page does not hard-code a version that can go stale.", vec![item("Pending", "Release records are not loaded", "Published versions will be ordered by release time.", "No mock data")]),
        section(
            "policy",
            "Release policy",
            "Fixes ship as new versions instead of replacing public assets in place.",
            vec![
                item("Source", "Explicit sources", "First-party files and external mirrors remain clearly distinct.", "Traceable"),
                item("Integrity", "Verifiable hashes", "Every asset exposes its own SHA256 and architecture.", "Immutable"),
                item("Validation", "Validate before release", "Build, signature, and real-feature checks precede publication.", "Real path"),
            ],
        ),
    ])
}

pub(super) fn faq() -> PageContent {
    page(
        PageId::Faq,
        "SSH client and agent FAQ | C-SSH",
        "Answers about C-SSH connections, the resident agent, cloud sync, credential privacy, download verification, and mobile scope.",
        "Frequently asked questions",
        "Clear answers to the important boundaries",
        "A concise guide to connections, the agent, cloud sync, and downloads.",
    )
    .with_faqs(vec![
        FaqItem::new("Does Creation Cloud proxy SSH connections?", "No. The SSH data plane stays direct from the client to your server; the cloud only provides account, device, and optional sync controls."),
        FaqItem::new("Can I use it without the agent?", "A standard SSH terminal and port forwarding currently use native SSH. Jump hosts share that architectural exception but remain deferred. Persistent sessions, monitoring, and structured management use the agent."),
        FaqItem::new("Are host addresses and private keys synced?", "Host names, addresses, ports, tags, status, connection settings, and credentials sync only as client-encrypted ciphertext. AI keys, APIs, and model bindings use the same client-side protection boundary. known_hosts, terminal content, and command history never upload."),
        FaqItem::new("Is the vault password my account password?", "No. The account password signs you in. The vault password derives encryption keys only on trusted clients and is never uploaded."),
        FaqItem::new("How can I verify a download?", "Formal download entries show platform, architecture, file size, and SHA256 for verification before installation."),
        FaqItem::new("Is mobile a full copy of desktop?", "No. Android is a mobile companion focused on inspection, lightweight actions, and continuity with desktop workflows."),
    ])
}
