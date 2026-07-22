//! 提供英文文档入口、发布状态、实操指南与安全参考内容。

use crate::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationPlatform, DocumentationScreenshot, DocumentationSection,
    PageContent, PageId,
};

use super::en::{action, page};

const RELEASE_HREF: &str = "https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.16";

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Documentation,
        "Creation-SSH Docs | Install, Connect, and Operate",
        "Verifiable Creation-SSH documentation for trusted installation, adding a host, agent deployment, persistent terminals, monitoring, files, and AI.",
        "DOCS / TASK GUIDES",
        "Start with your first Creation-SSH host",
        "Documentation and tutorials now share one task-oriented home. Complete one real workflow and use its expected result to verify the path.",
    )
    .with_actions(vec![
        action("Open the v0.6.16 release", RELEASE_HREF, "button button-secondary"),
        action("Download the client", "/downloads", "button button-primary"),
    ])
    .with_documentation_page(documentation())
}

fn documentation() -> DocumentationContent {
    DocumentationContent {
        release_label: "Current public release",
        release_version: "v0.6.16",
        release_date: "2026-07-18",
        release_href: RELEASE_HREF,
        release_action_label: "Open Release",
        index_label: "Documentation",
        mobile_index_label: "Open documentation index",
        search_label: "Filter titles on this page",
        search_placeholder: "Try host, terminal, or monitoring",
        search_help: "This filters titles already loaded on this page. It does not search article text or external sites.",
        search_empty: "No title on this page matches.",
        status: DocumentationNotice::new(
            "BEFORE YOU START",
            "Verify the source and host identity first",
            "Use only assets from the public Creation-SSH Release. Stop if SHA256 is missing or mismatched, the host key changes, or the architecture is unsupported.",
        ),
        groups: groups(),
        platform_code: "00 / PLATFORM MATRIX",
        platform_title: "Choose the asset for your platform and architecture",
        platform_lead: "v0.6.16 has seven application release assets. Windows, Linux, and Android are delivered; macOS and iOS remain planned.",
        platforms: platforms(),
        tutorials: super::en_tutorials::content(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "A standard PTY and a persistent terminal are separate paths",
            lead: "The image shows a direct standard PTY. Only persistent-terminal mode uses the client, agent, and tmux to provide a reconnectable session.",
            src: "/static/img/product-terminal.png",
            alt: "Redacted Creation-SSH demo terminal showing an example server and a standard SSH PTY",
            caption: "Redacted demo with an RFC 5737 example address. It explains the UI path only and is not persistent-session, release, or no-mock evidence.",
            width: 1650,
            height: 1080,
        },
        final_code: "NEXT / KEEP EVIDENCE",
        final_title: "Preserve the workspace before reporting a problem",
        final_body: "Never silently accept a changed host key, end an unauthorized remote session, or submit real addresses, passwords, private keys, tokens, or complete sensitive logs in feedback.",
    }
}

fn groups() -> Vec<DocumentationGroup> {
    vec![
        group(
            "Quick start",
            vec![
                link("platform-matrix", "00", "Platforms and release status"),
                link("getting-started", "01", "Download, verify, and install"),
            ],
        ),
        group(
            "Hands-on guides",
            vec![
                link("add-host", "02", "Add your first host"),
                link("deploy-agent", "03", "Deploy or repair the agent"),
                link(
                    "persistent-terminal",
                    "04",
                    "Create a reconnectable terminal",
                ),
                link("monitoring", "05", "Enable monitoring and history"),
                link("files", "06", "Browse and transfer files"),
                link("ai-assistant", "07", "Configure and run the AI assistant"),
            ],
        ),
        group(
            "Reference and safety",
            vec![
                link("port-forwarding", "08", "Local SSH forwarding"),
                link("cloud-security", "09", "Cloud and the data boundary"),
                link("troubleshooting", "10", "Safe stop conditions"),
            ],
        ),
    ]
}

fn platforms() -> Vec<DocumentationPlatform> {
    vec![
        DocumentationPlatform::released(
            "W",
            "Windows",
            "Desktop mainline",
            "v0.6.16 released",
            "Creation-SSH_0.6.16_x64-setup.exe · Creation-SSH_0.6.16_x64_en-US.msi · Creation-SSH_0.6.16_portable-Windows-x64.zip",
            "Choose one of NSIS, MSI, or portable; do not install multiple variants together.",
            RELEASE_HREF,
        ),
        DocumentationPlatform::released(
            "L",
            "Linux",
            "Independent desktop",
            "v0.6.16 released",
            "Creation-SSH_0.6.16_linux-amd64.deb · Creation-SSH_0.6.16_linux-x86_64.AppImage",
            "Use deb on Debian-family systems; other x86_64 desktops may use AppImage according to distribution policy.",
            RELEASE_HREF,
        ),
        DocumentationPlatform::released(
            "A",
            "Android",
            "Mobile companion",
            "v0.6.16 released",
            "C-SSH_0.6.16_android-arm64.apk · C-SSH_0.6.16_android-arm64.aab",
            "Most users install the signed APK. AAB is for store distribution and is not directly installable.",
            RELEASE_HREF,
        ),
        DocumentationPlatform::planned(
            "m",
            "macOS",
            "Future desktop",
            "Planned, no download",
            "No deliverable client or installer exists yet.",
        ),
        DocumentationPlatform::planned(
            "i",
            "iOS",
            "Future companion",
            "Planned, no download",
            "Development and real-device validation are not complete.",
        ),
    ]
}

fn sections() -> Vec<DocumentationSection> {
    vec![
        section(
            "getting-started",
            "01 / QUICK START",
            "Download, verify, and install",
            "Establish a trusted software source before connecting to a server.",
            vec![
                text(
                    "SOURCE",
                    "Use the public Release only",
                    "Open the v0.6.16 release and select one matching application asset from the platform matrix. Do not install copies from chat attachments, re-uploaded drives, or unknown mirrors.",
                    true,
                ),
                command(
                    "SHA256",
                    "Compare the checksum character for character",
                    "Compute SHA256 and compare it with the same asset in the Release notes. Stop if the checksum is missing or any character differs.",
                    "Windows: Get-FileHash .\\<asset> -Algorithm SHA256\nLinux:   sha256sum ./<asset>",
                    true,
                ),
                text(
                    "INSTALL",
                    "Install one matching variant",
                    "On Windows choose NSIS, MSI, or portable; on Linux choose deb or AppImage; for Android use the signed APK. macOS and iOS have no downloads.",
                    false,
                ),
            ],
        ),
        section(
            "port-forwarding",
            "08 / PORT FORWARDING",
            "Use native SSH local forwarding",
            "Port forwarding is an intentional pure-SSH exception and does not depend on the agent.",
            vec![command(
                "LOOPBACK",
                "Bind to loopback by default",
                "Map a server-reachable target to 127.0.0.1 on the client. Change the listen address only when you understand the exposure.",
                "127.0.0.1:<local-port> -> <remote-host>:<remote-port>",
                true,
            )],
        ),
        section(
            "cloud-security",
            "09 / CLOUD & SECURITY",
            "A Cloud account is optional and SSH remains the data plane",
            "The current local implementation is neither deployed nor connected online. A page existing does not mean the cloud capability is delivered.",
            vec![
                text(
                    "OPTIONAL",
                    "Manage local hosts without a Cloud account",
                    "SSH connections, standard terminals, and local workflows do not require Cloud sign-in. Cloud is planned only as the control plane for accounts, devices, sync, models, vault envelopes, versions, and downloads.",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud never proxies the SSH data plane",
                    "Private keys, passwords, and plaintext secrets do not go to Cloud. Only client-side encrypted vault envelopes defined by the dedicated plan may be stored.",
                    true,
                ),
            ],
        ),
        section(
            "troubleshooting",
            "10 / TROUBLESHOOTING",
            "Stop safely when something is wrong",
            "Protect identity and remote work first, then diagnose network, architecture, resources, and permissions.",
            vec![
                text(
                    "HOST KEY",
                    "The host key changed",
                    "Stop connecting and verify the new fingerprint and reason through a trusted channel. Do not delete known_hosts entries to skip confirmation.",
                    true,
                ),
                text(
                    "DEPLOY",
                    "Architecture or paired resources are missing",
                    "Run real uname -m detection again; the last architecture in SQLite is only a record. Do not upload another architecture or send both sets.",
                    true,
                ),
                text(
                    "SESSION",
                    "The session did not return",
                    "Confirm you used a persistent terminal rather than a standard PTY, then inspect agent and tmux ownership. Do not kill sessions or remove unknown sockets without authorization.",
                    true,
                ),
            ],
        ),
    ]
}

fn group(title: &'static str, links: Vec<DocumentationLink>) -> DocumentationGroup {
    DocumentationGroup::new(title, links)
}

const fn link(anchor: &'static str, code: &'static str, title: &'static str) -> DocumentationLink {
    DocumentationLink::new(anchor, code, title)
}

fn section(
    anchor: &'static str,
    code: &'static str,
    title: &'static str,
    lead: &'static str,
    items: Vec<DocumentationItem>,
) -> DocumentationSection {
    DocumentationSection::new(anchor, code, title, lead, items)
}

const fn text(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::text(badge, title, body, caution)
}

const fn command(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    value: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::command(badge, title, body, value, caution)
}
