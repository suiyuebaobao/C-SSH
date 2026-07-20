//! 提供英文入门文档、平台发布状态与当前产品边界。

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
        "Install and First Connection Docs | Creation-SSH",
        "A bilingual Creation-SSH guide from trusted downloads and host-key verification to agent deployment and persistent-terminal reconnection.",
        "DOCS / GETTING STARTED",
        "Install, connect, and keep operating",
        "Follow one complete path from a trusted package to a workspace you can reconnect to. Verify the release and host identity before enabling agent capabilities.",
    )
    .with_actions(vec![
        action("Open the v0.6.16 release", RELEASE_HREF, "button button-primary"),
        action("Continue to tutorials", "/tutorials", "button button-secondary"),
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
        index_label: "On this page",
        mobile_index_label: "Open this-page index",
        search_label: "Filter section titles on this page",
        search_placeholder: "Try terminal, monitoring, or Cloud",
        search_help: "This filters section titles in the current-page index only. It does not search article text or external sites.",
        search_empty: "No section title on this page matches.",
        status: DocumentationNotice::new(
            "STATUS / SAFETY",
            "Verify the source and host identity before installing or deploying",
            "Use only assets from the public Creation-SSH Release. Stop if SHA256 is missing or mismatched, the host key changes, or the architecture is unsupported; never guess past a security check.",
        ),
        groups: groups(),
        platform_code: "00 / PLATFORM MATRIX",
        platform_title: "Choose the release asset for your platform and architecture",
        platform_lead: "v0.6.16 was published on 2026-07-18 with seven application release assets, excluding GitHub-generated source archives. Windows, Linux, and Android are delivered; macOS and iOS remain planned.",
        platforms: platforms(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "The terminal UI separates a standard PTY from persistent sessions",
            lead: "The image shows the selected direct standard PTY. Only after switching to a persistent terminal do the client, agent, and tmux provide a reconnectable session.",
            src: "/static/img/product-terminal.png",
            alt: "Redacted Creation-SSH demo terminal showing an example server and the selected standard SSH PTY",
            caption: "Redacted demo: the selected view is a direct standard PTY and the address is an RFC 5737 example. It explains the path distinction, not persistent-session, release, or no-mock evidence.",
            width: 1650,
            height: 1080,
        },
        final_code: "NEXT / OPERATE WITH EVIDENCE",
        final_title: "After reconnection succeeds, enable monitoring, files, and system tools in stages",
        final_body: "Keep failures visible: never silently accept a changed host key, end an unauthorized remote session, or upload plaintext secrets to Cloud.",
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
            "Connection and auth",
            vec![link(
                "connection-auth",
                "02",
                "First connection and host keys",
            )],
        ),
        group(
            "Agent and terminal",
            vec![link(
                "agent-terminal",
                "03",
                "Deployment and persistent terminal",
            )],
        ),
        group(
            "Monitoring",
            vec![link(
                "monitoring",
                "04",
                "Snapshots, live data, and history",
            )],
        ),
        group(
            "Files and system",
            vec![link(
                "files-system",
                "05",
                "Structured file and system actions",
            )],
        ),
        group(
            "Port forwarding",
            vec![link("port-forwarding", "06", "Local SSH forwarding")],
        ),
        group(
            "AI",
            vec![link("ai", "07", "Bring your model and confirm tools")],
        ),
        group(
            "Cloud and security",
            vec![link(
                "cloud-security",
                "08",
                "Optional account and data boundary",
            )],
        ),
        group(
            "Troubleshooting",
            vec![link("troubleshooting", "09", "Safe stop conditions")],
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
            "Most users install the signed APK. AAB is a store-distribution bundle, not a directly installable package.",
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
                    "Open the v0.6.16 release and select one of its seven application release assets, excluding GitHub-generated source archives, from the platform matrix. Do not install copies from chat attachments, re-uploaded drives, or unknown mirrors.",
                    true,
                ),
                command(
                    "SHA256",
                    "Compare the checksum character for character",
                    "Compute SHA256 for the downloaded file and compare it with the same asset in that version's Release notes. If the notes do not provide SHA256 for the exact asset, or any character differs, stop.",
                    "Windows: Get-FileHash .\\<asset> -Algorithm SHA256\nLinux:   sha256sum ./<asset>",
                    true,
                ),
                text(
                    "INSTALL",
                    "Install one matching variant",
                    "On Windows choose NSIS, MSI, or portable; on Linux choose deb or AppImage; for normal Android installation use the signed APK. macOS and iOS have no downloads.",
                    false,
                ),
            ],
        ),
        section(
            "connection-auth",
            "02 / CONNECTION & AUTH",
            "First connection and host keys",
            "The values below document fields only. You must verify the identity of a real server.",
            vec![
                command(
                    "EXAMPLE",
                    "Add an example host",
                    "Use example.com or the RFC 5737 address 192.0.2.10, keep port 22 unless your server differs, then select password or private-key authentication. Give real credentials only to the local client; never put them in the website, documentation, or agent.",
                    "Host: example.com\nAddress: 192.0.2.10\nPort: 22\nAuth: password or private key",
                    false,
                ),
                text(
                    "TRUST",
                    "Confirm the first host key explicitly",
                    "Compare the fingerprint through an independent trusted channel, then accept it explicitly. If it changes later, establish whether the server, DNS, or proxy changed; never accept silently.",
                    true,
                ),
                text(
                    "FALLBACK",
                    "A standard PTY works without the agent",
                    "The standard terminal uses a native SSH PTY, and port forwarding is native SSH too. Persistent terminals, structured monitoring, files, system actions, application operations, and AI tools require the agent.",
                    false,
                ),
            ],
        ),
        section(
            "agent-terminal",
            "03 / AGENT & TERMINAL",
            "Deploy the agent and verify a persistent terminal",
            "The client detects architecture and uploads only the matching agent plus static tmux pair.",
            vec![
                text(
                    "DETECT",
                    "Read first, then upload the matching pair",
                    "Authenticated SSH runs read-only uname -m first. Both resources must exist for x86_64 or aarch64, and unsupported or incomplete pairs must fail before directories, uploads, or old-service stops. The aarch64 build and automatic-selection foundation is verified, but without a real ARM64 test server this is not a claim of complete ARM64 hardware support.",
                    true,
                ),
                text(
                    "SESSION",
                    "Create the first persistent terminal",
                    "Choose the persistent terminal so the agent drives a tmux session. Standard PTY and persistent session are explicit paths; do not treat an ordinary shell as recoverable.",
                    false,
                ),
                text(
                    "RECONNECT",
                    "Disconnect the client and return",
                    "Leave a non-sensitive marker in the session, disconnect normally without ending tmux, then reconnect and confirm capture-pane restores the same context. Only a real disconnect and reconnect completes this check.",
                    false,
                ),
            ],
        ),
        section(
            "monitoring",
            "04 / MONITORING",
            "Move from snapshots to live data and history",
            "Each background round requests one structured MetricsSnapshot per host; the detail view opens a separate on-demand stream.",
            vec![
                text(
                    "SNAPSHOT",
                    "Start with the host overview",
                    "Confirm CPU, memory, disk, load, and agent state form one consistent snapshot. Cross-host scheduling and per-host short-request protection are separate limits.",
                    false,
                ),
                text(
                    "HISTORY",
                    "Then inspect live and historical data",
                    "Use the on-demand long stream on the detail view; the agent stores history locally in redb. Do not imitate a subscription with repeated short polling.",
                    false,
                ),
            ],
        ),
        section(
            "files-system",
            "05 / FILES & SYSTEM",
            "Use structured actions in the current host context",
            "Files, processes, firewall, systemd, and application actions should return structured agent results instead of client-built shell commands.",
            vec![
                text(
                    "FILES",
                    "Begin with read-only browsing",
                    "Verify the path, permissions, and target host before editing, uploading, or downloading. High-risk overwrites must preserve confirmation and visible failures; resumable transfer follows its own session rules.",
                    true,
                ),
                text(
                    "SYSTEM",
                    "Confirm high-risk actions one at a time",
                    "Check ownership before stopping a process or changing firewall or service state. Never end an unknown process, user SSH session, or unauthorized tmux workload.",
                    true,
                ),
            ],
        ),
        section(
            "port-forwarding",
            "06 / PORT FORWARDING",
            "Use native SSH local forwarding",
            "Port forwarding is an intentional pure-SSH exception and does not depend on the agent.",
            vec![command(
                "LOOPBACK",
                "Bind to loopback by default",
                "Map a server-reachable target to 127.0.0.1 on the client. Change the listen address only when you explicitly understand the exposure.",
                "127.0.0.1:<local-port> -> <remote-host>:<remote-port>",
                true,
            )],
        ),
        section(
            "ai",
            "07 / AI ASSISTANT",
            "Bring your own model and keep tool confirmation",
            "AI orchestrates structured agent capabilities; it must not bypass permissions, confirmation, or audit.",
            vec![
                text(
                    "MODEL",
                    "Keep model configuration local",
                    "Choose a compatible bring-your-own-model provider. Keys must never appear in logs, documentation, or on the agent. First test the connection with a question that calls no tools.",
                    true,
                ),
                text(
                    "TOOLS",
                    "Review the target and arguments",
                    "Before host reads or changes, verify the host, tool name, and parameters, then confirm explicitly. Do not let AI use free-form shell to bypass structured boundaries.",
                    true,
                ),
            ],
        ),
        section(
            "cloud-security",
            "08 / CLOUD & SECURITY",
            "A Cloud account is optional and SSH remains the data plane",
            "The current local implementation is not deployed or connected to an online service. A page existing does not mean a cloud capability is delivered.",
            vec![
                text(
                    "OPTIONAL",
                    "Manage local hosts without a Cloud account",
                    "SSH connections, standard terminals, and local workflows do not require Cloud sign-in. Cloud is intended as the control plane for accounts, devices, sync, model configuration, vault envelopes, versions, and downloads.",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud never proxies the SSH data plane",
                    "Private keys, passwords, and plaintext secrets do not go to Cloud. Only client-side encrypted vault envelopes defined by the dedicated plan may be stored; the current local Cloud implementation is neither deployed nor production-connected.",
                    true,
                ),
            ],
        ),
        section(
            "troubleshooting",
            "09 / TROUBLESHOOTING",
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
                    "Run real uname -m detection again; the last architecture in SQLite is only a record. Do not upload another architecture or send both sets when the matching agent or tmux is missing.",
                    true,
                ),
                text(
                    "SESSION",
                    "The session did not return",
                    "Confirm you used a persistent terminal rather than a standard PTY, then inspect agent and tmux ownership. Do not kill remote sessions or remove unknown sockets without authorization.",
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
