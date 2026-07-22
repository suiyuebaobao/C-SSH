//! 提供英文工业制图首页的完整产品信息结构。

use crate::{
    HomeFaqItem, HomeItem, HomeLayout, HomePageContent, HomePlatform, HomeQrLabels, HomeQrWidget,
    HomeSection, HomeTone, PageContent, PageId,
};

use super::en::{action, page};

pub(super) fn page_content() -> PageContent {
    let platforms = platforms();
    let sections = sections(&platforms);
    let home_page = HomePageContent {
        status_note: "Local implementation · Not deployed · Not published",
        platform_label: "I/O MATRIX / PLATFORM",
        platform_note: "5 SLOTS",
        platforms,
        sections,
        faq_code: "08 / FAQ",
        faq_heading: "Answer the questions that decide adoption",
        faq_lead: "The home page keeps only high-value decision questions; the FAQ page carries the full explanation.",
        faqs: faqs(),
        final_code: "NEXT STEP / START HERE",
        final_heading: "Choose a platform, then start with the first tutorial",
        final_lead: "Capabilities, platform status, and downloads always follow real implementation, deployment, and release records.",
        qr_widget: HomeQrWidget::pending(HomeQrLabels {
            code: "MOBILE ACCESS / QR",
            title: "Scan entry",
            pending: "QR code awaiting admin upload",
            ready: "QR code published",
            note: "Click the card to toggle size",
            image_alt: "Creation-SSH QR code",
            open: "Enlarge QR code",
            close: "Close QR code",
        }),
    };

    page(
        PageId::Home,
        "Creation-SSH | SSH client for server operations",
        "Creation-SSH is a native SSH client and server operations tool combining SSH transport with a resident agent for persistent terminals, monitoring, files, and system tasks.",
        "CLIENT × RESIDENT AGENT",
        "Keep the SSH working context online",
        "A native client and resident agent keep persistent terminals, monitoring, files, and system tools in the context of the same host.",
    )
    .with_actions(vec![
        action(
            "Read the getting-started guide",
            "/docs/getting-started",
            "button button-primary",
        ),
        action("Read the changelog", "/changelog", "button button-secondary"),
        action("Start with docs", "/docs/getting-started#add-host", "text-link"),
    ])
    .with_home_page(home_page)
}

fn platforms() -> Vec<HomePlatform> {
    vec![
        HomePlatform::current(
            "W",
            "Windows",
            "Desktop client",
            "Complete desktop operations",
            "Independent native client",
            "See downloads for live records",
        ),
        HomePlatform::current(
            "L",
            "Linux",
            "Independent client",
            "Desktop operations and local builds",
            "Independent native client",
            "See downloads for live records",
        ),
        HomePlatform::current(
            "A",
            "Android",
            "Mobile companion",
            "Mobile inspection and lightweight actions",
            "Independent mobile client",
            "See downloads for live records",
        ),
        HomePlatform::planned(
            "m",
            "macOS",
            "Independent desktop client",
            "Future desktop client",
            "Reserved independent platform boundary",
            "Planned · Downloads not open",
        ),
        HomePlatform::planned(
            "i",
            "iOS",
            "Independent mobile companion",
            "Future mobile companion",
            "Reserved independent platform boundary",
            "Planned · Downloads not open",
        ),
    ]
}

fn sections(platforms: &[HomePlatform]) -> Vec<HomeSection> {
    vec![
        section(
            "workflow",
            HomeLayout::Workflow,
            "Make the client and agent boundary obvious",
            "The client authenticates and initiates, SSH is the only entry point, and the resident agent exposes structured capabilities through a server-local socket.",
            vec![
                item(
                    "NODE 01",
                    "Native client",
                    "Hosts, credentials, trust, and operation entry points stay local.",
                    "CLIENT",
                ),
                item(
                    "LINK 02",
                    "SSH tunnel",
                    "Reuse SSH authentication and encryption without a new public port.",
                    "PURE SSH",
                ),
                item(
                    "CORE 03",
                    "Resident agent",
                    "Listen only on the server-local socket and provide structured capabilities.",
                    "AGENT",
                ),
                item(
                    "STATE 04",
                    "tmux / redb",
                    "Terminal context and monitoring history persist on the server.",
                    "SERVER STATE",
                ),
                item(
                    "TOOLS 05",
                    "Files and systems",
                    "Reliable transfers and system actions use explicit protocols.",
                    "STRUCTURED",
                ),
                item(
                    "PURE SSH",
                    "Works without the agent",
                    "Standard terminals, port forwarding, and access grants keep a native SSH path.",
                    "Fallback boundary",
                ),
                item(
                    "RESIDENT AGENT",
                    "Client × agent integration",
                    "Persistent terminals, monitoring history, files, AI, systems, and apps use the structured protocol.",
                    "Primary path",
                ),
            ],
        ),
        section(
            "capabilities",
            HomeLayout::Capabilities,
            "One workspace, nine clearly bounded modules",
            "Every module states its dependency and result. The product docs continue with architecture, boundaries, interfaces, and verification.",
            vec![
                item(
                    "MIXED",
                    "Host management",
                    "Connections, authentication, host keys, and real reachability results.",
                    "Connection entry",
                ),
                item(
                    "MIXED",
                    "Terminal",
                    "Choose persistent tmux or a standard SSH PTY by task.",
                    "Resume context",
                ),
                item(
                    "AGENT",
                    "Monitoring",
                    "Snapshots, live subscriptions, history, and top processes.",
                    "Structured status",
                ),
                item(
                    "AGENT",
                    "Files",
                    "Browse, search, edit, and transfer with integrity checks.",
                    "Same host context",
                ),
                item(
                    "PURE SSH",
                    "Port forwarding",
                    "Map server-reachable ports safely to the client machine.",
                    "Native SSH",
                ),
                item(
                    "AGENT",
                    "AI assistant",
                    "Invoke structured tools behind permission, confirmation, and audit gates.",
                    "Bring your model",
                ),
                item(
                    "AGENT",
                    "System management",
                    "Read system facts and manage processes or firewalls explicitly.",
                    "Visible actions",
                ),
                item(
                    "AGENT",
                    "Application center",
                    "Focus on Docker, common apps, and systemd services.",
                    "Capability modules",
                ),
                item(
                    "PURE SSH",
                    "Access grants",
                    "Issue isolated access keys and revoke them by stable marker.",
                    "Least access",
                ),
            ],
        ),
        section(
            "first-run",
            HomeLayout::Steps,
            "From the first host to a complete workflow",
            "Tutorials form an executable path from first connection to daily operations instead of a loose article collection.",
            vec![
                item(
                    "STEP 01",
                    "Add a host",
                    "Complete SSH authentication and host-key trust.",
                    "Verify connection",
                ),
                item(
                    "STEP 02",
                    "Deploy the agent",
                    "Probe server architecture and select the matching agent and tmux pair.",
                    "Paired deployment",
                ),
                item(
                    "STEP 03",
                    "Persistent terminal",
                    "Disconnect deliberately, then verify task and screen recovery.",
                    "tmux",
                ),
                item(
                    "STEP 04",
                    "Monitoring history",
                    "Inspect live detail, historical ranges, and processes.",
                    "redb",
                ),
                item(
                    "STEP 05",
                    "File transfer",
                    "Upload or download and verify integrity results.",
                    "Chunk verification",
                ),
                item(
                    "STEP 06",
                    "AI assistant",
                    "Use your own model and begin with read-only diagnosis.",
                    "Permission gate",
                ),
            ],
        ),
        section(
            "platforms",
            HomeLayout::Platforms,
            "Five platforms with independent positions",
            "Public Windows, Linux, and Android status comes from real release records. macOS and iOS keep separate, explicitly planned positions.",
            platform_items(platforms),
        ),
        section(
            "security",
            HomeLayout::Security,
            "A direct SSH data plane and restrained cloud control plane",
            "The two paths stay separate: Creation Cloud never enters the SSH data plane between the client and your server.",
            vec![
                item(
                    "SSH DATA PLANE",
                    "Client to user server",
                    "The agent uses a local socket, host-key changes require confirmation, and SSH, tmux, or user processes are never ended without authorization.",
                    "Cloud is not in this path",
                ),
                item(
                    "CLOUD CONTROL PLANE",
                    "Accounts, devices, and optional sync",
                    "Only a non-sensitive allowlist can sync. Vault encryption stays on trusted clients and the service stores versioned ciphertext envelopes only.",
                    "No plaintext sensitive data",
                ),
            ],
        ),
        section(
            "downloads",
            HomeLayout::Downloads,
            "Confirm source and checksum before downloading",
            "The home page reads the latest real release. Until release data is connected, it keeps an explicit empty state without fake versions or placeholder links.",
            vec![
                item(
                    "WINDOWS",
                    "Awaiting release data",
                    "Architecture, source, size, and release time appear after integration.",
                    "SHA256: —",
                ),
                item(
                    "LINUX",
                    "Awaiting release data",
                    "AppImage and deb entries require real local-build records.",
                    "SHA256: —",
                ),
                item(
                    "ANDROID",
                    "Awaiting release data",
                    "Formal arm64 assets stay separate from test-only x86_64 packages.",
                    "SHA256: —",
                ),
                item(
                    "macOS",
                    "Planned",
                    "No download action or support implication.",
                    "SHA256: —",
                ),
                item(
                    "iOS",
                    "Planned",
                    "No download action or support implication.",
                    "SHA256: —",
                ),
            ],
        ),
        section(
            "cloud",
            HomeLayout::Cloud,
            "Accounts support devices and sync, never SSH",
            "Creation Cloud control plane status: implemented locally, not deployed, not published, and not integrated into clients. The page states both value and delivery status.",
            vec![
                item(
                    "ACCOUNT",
                    "Profile and security",
                    "Account status, security settings, and password boundaries.",
                    "Account password ≠ vault password",
                ),
                item(
                    "DEVICE",
                    "Devices",
                    "Register, rename, and revoke trusted devices.",
                    "No SSH host records",
                ),
                item(
                    "SYNC",
                    "Sync",
                    "Revisions, conflicts, and non-sensitive allowlist results.",
                    "Unknown fields denied",
                ),
                item(
                    "MODEL",
                    "Models",
                    "Sync credential-free model metadata, defaults, and order.",
                    "API keys reference ciphertext",
                ),
                item(
                    "VAULT",
                    "Vault",
                    "Show ciphertext versions and device-wrapper status only.",
                    "Server cannot decrypt",
                ),
                item(
                    "RELEASE",
                    "Downloads",
                    "Expose compatible releases, sources, and download history.",
                    "Real records",
                ),
            ],
        ),
    ]
}

fn faqs() -> Vec<HomeFaqItem> {
    vec![
        HomeFaqItem::new(
            "Does Creation Cloud proxy SSH?",
            "No. The SSH data plane remains direct from the client to your server. Cloud only provides account, device, and optional sync controls.",
        ),
        HomeFaqItem::new(
            "Can I use Creation-SSH without the agent?",
            "A standard SSH terminal, port forwarding, and access grants remain available. Persistent sessions, monitoring, and structured management depend on the agent.",
        ),
        HomeFaqItem::new(
            "Are host addresses and private keys uploaded?",
            "Not in plaintext. Host records, passwords, private keys, known_hosts, terminal content, and command history are outside the sync allowlist.",
        ),
        HomeFaqItem::new(
            "Is the vault password the account password?",
            "No. The account password signs you in. The vault password derives encryption keys only on trusted clients and is never uploaded.",
        ),
        HomeFaqItem::new(
            "How can I verify a download?",
            "Formal download entries expose platform, architecture, source, file size, and SHA256 for verification before installation.",
        ),
        HomeFaqItem::new(
            "Is Android a full desktop copy?",
            "No. Android is a mobile companion focused on inspection, lightweight actions, and continuity with desktop workflows.",
        ),
    ]
}

fn section(
    anchor: &'static str,
    layout: HomeLayout,
    title: &'static str,
    lead: &'static str,
    items: Vec<HomeItem>,
) -> HomeSection {
    let (code, side_label) = match layout {
        HomeLayout::Workflow => ("01 / HOW IT WORKS", "SYSTEM FLOW / DATA PATH"),
        HomeLayout::Capabilities => ("02 / CAPABILITIES", "FUNCTION MODULE / 09 UNITS"),
        HomeLayout::Steps => ("03 / FIRST RUN", "OPERATION SEQUENCE / 06 STEPS"),
        HomeLayout::Platforms => ("04 / PLATFORMS", "PLATFORM MATRIX / 05 SLOTS"),
        HomeLayout::Security => ("05 / SECURITY BOUNDARY", "SEPARATED PLANES"),
        HomeLayout::Downloads => ("06 / DOWNLOADS", "SOURCE AND CHECKSUM"),
        HomeLayout::Cloud => ("07 / CREATION CLOUD", "CONTROL SURFACE"),
    };
    HomeSection::new(anchor, code, side_label, layout, title, lead, items)
}

fn item(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    meta: &'static str,
) -> HomeItem {
    let tone = match badge {
        "CORE 03" | "SSH DATA PLANE" => HomeTone::Dark,
        "PLANNED" | "macOS" | "iOS" => HomeTone::Planned,
        "AGENT" | "CLOUD CONTROL PLANE" | "RESIDENT AGENT" => HomeTone::Accent,
        _ => HomeTone::Default,
    };
    HomeItem::new(badge, title, body, meta, tone)
}

fn platform_items(platforms: &[HomePlatform]) -> Vec<HomeItem> {
    platforms
        .iter()
        .map(|platform| {
            HomeItem::new(
                platform.state,
                platform.name,
                platform.position,
                platform.shell,
                if platform.planned {
                    HomeTone::Planned
                } else {
                    HomeTone::Default
                },
            )
        })
        .collect()
}
