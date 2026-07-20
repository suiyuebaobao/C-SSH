//! 提供英文安全、下载、更新与常见问题内容。

use crate::{FaqItem, PageContent, PageId};

use super::en::{action, item, page, section};

pub(super) fn security() -> PageContent {
    page(
        PageId::Security,
        "SSH tunnel, host key, and vault security | Creation-SSH",
        "Learn how Creation-SSH handles SSH tunnels, host-key verification, the agent local socket, cloud data boundaries, and client-side encrypted vaults.",
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
                item("Sync", "Non-sensitive allowlist", "Only explicitly allowed preferences and model metadata can sync.", "Default deny"),
                item("Vault", "Client-side encryption", "The server stores ciphertext envelopes and key-wrapper metadata only.", "Zero knowledge"),
                item("Logs", "Redacted records", "Passwords, tokens, cookies, ciphertext bodies, and SSH details stay out of logs.", "Minimum record"),
            ],
        ),
    ])
}

pub(super) fn downloads() -> PageContent {
    page(
        PageId::Downloads,
        "Client releases and platform status | Creation-SSH",
        "Review Creation-SSH status for Windows, Linux, Android, macOS, and iOS; verify source, architecture, and SHA256 after real assets are published.",
        "Release records are authoritative",
        "Choose the right build for your platform",
        "Once assets are connected, this page will distinguish first-party files from clearly labeled external mirrors.",
    )
    .with_actions(vec![action("Read the changelog", "/changelog", "button button-secondary")])
    .with_sections(vec![section(
        "builds",
        "Platform builds",
        "Placeholder links never masquerade as downloadable assets.",
        vec![
            item("Desktop", "Windows", "Installer, MSI, and portable builds will list size and SHA256 separately.", "Awaiting release data"),
            item("Desktop", "Linux", "AppImage and deb appear only after local builds and real validation.", "Awaiting release data"),
            item("Mobile", "Android", "Release arm64 assets remain distinct from test-only x86_64 builds.", "Awaiting release data"),
            item("Planned", "macOS", "The independent macOS client has not been developed yet; no download is offered.", "Not developed yet"),
            item("Planned", "iOS", "The independent iOS companion has not been developed yet; no download is offered.", "Not developed yet"),
        ],
    )])
}

pub(super) fn changelog() -> PageContent {
    page(
        PageId::Changelog,
        "Release records and publication policy | Creation-SSH",
        "Review Creation-SSH release records, asset sources, SHA256, and real-validation policy without fabricated version data.",
        "An immutable release history",
        "Every formal version keeps its own record",
        "Release notes, sources, validation, and SHA256 will appear together once the release service is connected.",
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
        "SSH client and agent FAQ | Creation-SSH",
        "Answers about Creation-SSH connections, the resident agent, cloud sync, credential privacy, download verification, and mobile scope.",
        "Frequently asked questions",
        "Clear answers to the important boundaries",
        "A concise guide to connections, the agent, cloud sync, and downloads.",
    )
    .with_faqs(vec![
        FaqItem::new("Does Creation Cloud proxy SSH connections?", "No. The SSH data plane stays direct from the client to your server; the cloud only provides account, device, and optional sync controls."),
        FaqItem::new("Can I use it without the agent?", "A standard SSH terminal and port forwarding currently use native SSH. Jump hosts share that architectural exception but remain deferred. Persistent sessions, monitoring, and structured management use the agent."),
        FaqItem::new("Are host addresses and private keys synced?", "Not in plaintext. Host details, passwords, private keys, known_hosts, terminal content, and command history are outside the cloud allowlist."),
        FaqItem::new("Is the vault password my account password?", "No. The account password signs you in. The vault password derives encryption keys only on trusted clients and is never uploaded."),
        FaqItem::new("How can I verify a download?", "Formal download entries show platform, architecture, file size, and SHA256 for verification before installation."),
        FaqItem::new("Is mobile a full copy of desktop?", "No. Android is a mobile companion focused on inspection, lightweight actions, and continuity with desktop workflows."),
    ])
}
