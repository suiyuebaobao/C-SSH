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
        status_strip_label: "OFFICIAL PRODUCT SURFACE / INDUSTRIAL SYSTEM".into(),
        status_note:
            "0.9.1 · Manual installation · No upgrade reminders or forced updates"
                .into(),
        hero_blueprint_label: "BRAND GEOMETRY / UNIT 01".into(),
        platform_label: "I/O MATRIX / PLATFORM".into(),
        platform_note: "3 RELEASED / iOS NOT PUBLISHED".into(),
        platforms,
        sections,
        faq_side_label: "FAQ / DECISION QUESTIONS".into(),
        faq_item_prefix: "FAQ".into(),
        faq_code: "08 / FAQ".into(),
        faq_heading: "Answer the questions that decide adoption".into(),
        faq_lead:
            "The home page keeps only high-value decision questions; the FAQ page carries the full explanation."
                .into(),
        faqs: faqs(),
        seo_code: "SEARCH TOPICS / CONTENT MAP".into(),
        seo_heading: "Related SSH topics".into(),
        seo_topics_label: "Published SEO topics".into(),
        final_code: "NEXT STEP / START HERE".into(),
        final_heading: "Preserve your data before installing".into(),
        final_lead:
            "This release is communicated by announcement only, without upgrade reminders or forced updates. Download and install it manually, and read the installation and data-preservation guide first. The Mac automatic updater is deferred."
                .into(),
        qr_placeholder_code: "QR".into(),
        qr_placeholder_waiting: "WAITING".into(),
        media_slot: None,
        qr_widget: HomeQrWidget::pending(HomeQrLabels {
            code: "QQ COMMUNITY / QR",
            title: "Join the QQ community",
            pending: "QQ community QR awaiting upload",
            ready: "QQ community QR published",
            note: "Scan to join C-SSH users · Click to enlarge",
            image_alt: "QR code to join the C-SSH QQ community",
            open: "Enlarge QQ community QR",
            close: "Close QQ community QR",
        }),
    };

    page(
        PageId::Home,
        "C-SSH 0.9.1 | Windows · Mac · Android",
        "C-SSH 0.9.1 for Windows, Apple Silicon Mac and Android: SSH, RDP, persistent terminals, monitoring, files, scoped AI and optional self-hosted Cloud. Announcement only; manual installation without upgrade reminders or forced updates.",
        "C-SSH 0.9.1 / MANUAL DOWNLOAD",
        "C-SSH 0.9.1: one workspace for server operations",
        "Windows, Apple Silicon Mac and Android bring hosts, persistent terminals, remote desktops, monitoring, files and AI together. This release is communicated by announcement only. Download and install it manually; no upgrade reminders or forced updates are sent. iOS is not published in this release.",
    )
    .with_actions(vec![
        action(
            "Download 0.9.1 manually",
            "/en/downloads",
            "button button-primary",
        ),
        action("Complete feature guide", "https://github.com/suiyuebaobao/C-SSH/blob/main/FEATURES_EN.md", "button button-secondary"),
        action("Full screenshot gallery", "https://github.com/suiyuebaobao/C-SSH/blob/main/screenshots/README.md", "text-link"),
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
            "A",
            "Android",
            "Mobile companion",
            "Mobile inspection and lightweight actions",
            "Independent mobile client",
            "See downloads for live records",
        ),
        HomePlatform::current(
            "m",
            "macOS",
            "Apple Silicon / arm64",
            "Apple Silicon desktop client",
            "Ad-hoc signed · Not notarized",
            "DMG · Manual download",
        ),
        HomePlatform::planned(
            "i",
            "iOS",
            "iPhone client in development",
            "No download in this release",
            "No IPA or TestFlight upload",
            "Not published",
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
                    "Three clients",
                    "Windows x64, Apple Silicon Mac arm64 and Android arm64 share core capabilities with desktop and phone layouts.",
                    "0.9.1 · Manual installation",
                ),
                item(
                    "LINK 02",
                    "SSH and RDP connections",
                    "Enter an IPv4 address or domain in one field; the format is detected automatically. The port remains separate, and existing IPv6 hosts remain compatible.",
                    "PURE SSH",
                ),
                item(
                    "CORE 03",
                    "Agent or standard SSH",
                    "Agent mode provides structured management and persistent services; standard SSH uses existing PTY, SFTP and related services.",
                    "AGENT",
                ),
                item(
                    "STATE 04",
                    "Persistent terminals",
                    "Linux uses tmux; Windows uses the matching Agent and Terminal2/psmux. Reconnect to an existing session after closing the client.",
                    "Not a guarantee for every failure scenario",
                ),
                item(
                    "TOOLS 05",
                    "Files and system tools",
                    "Browse, transfer, edit and verify files. Monitoring, processes, firewall and service controls depend on the target capabilities.",
                    "Linux and Windows capabilities are negotiated",
                ),
                item(
                    "PURE SSH",
                    "Built-in Windows setup package",
                    "When adding a Windows management host, save the built-in setup ZIP from the client or share it from Android. Transfer it to the server, extract it and run Setup manually.",
                    "Available offline · Manual server installation",
                ),
                item(
                    "RESIDENT AGENT",
                    "Local-first operation",
                    "Use local server tools without a Cloud login. Optional accounts provide device management and explicit end-to-end encrypted sync.",
                    "Cloud does not relay SSH traffic",
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
                    "Hosts and projects",
                    "Create, edit, search, favorite and group hosts. Use IPv4 or domains, separate ports, passwords or keys, and direct or proxy routes.",
                    "Hosts / Projects",
                ),
                item(
                    "MIXED",
                    "Terminals and remote desktops",
                    "Persistent and standard SSH terminals, desktop tabs and separate RDP windows, with platform-specific keyboard, clipboard and file interactions.",
                    "SSH / RDP",
                ),
                item(
                    "AGENT",
                    "Monitoring and history",
                    "CPU, memory, disk, load, network and disk IO, live charts, history ranges, top processes and collection settings.",
                    "Available fields come from the target",
                ),
                item(
                    "AGENT",
                    "File management",
                    "Browse, upload, download, create, rename, delete, edit text and verify hashes. Mobile transfers use system file pickers.",
                    "Confirm the destination before acting",
                ),
                item(
                    "PURE SSH",
                    "Forwarding and broadcasts",
                    "Windows and Mac provide SSH port forwarding, saved commands, batch execution and per-host results. Android has no forwarding or broadcast pages.",
                    "Desktop features",
                ),
                item(
                    "AGENT",
                    "AI assistant",
                    "Global can access all local scope histories; project is limited to that project and host to that host. Manage model accounts, bindings, permissions and confirmations.",
                    "Conversations and memory remain local",
                ),
                item(
                    "AGENT",
                    "System management",
                    "System information, processes and firewall tools. Windows Native and Linux Agent expose operations according to negotiated capabilities.",
                    "Unknown capability is not treated as supported",
                ),
                item(
                    "AGENT",
                    "Application center",
                    "Desktop management for supported Docker containers, images, systemd units and Windows services.",
                    "Availability depends on the server",
                ),
                item(
                    "PURE SSH",
                    "Grants and diagnostics",
                    "Desktop access-grant management. Diagnostic logging is off by default and can be enabled, queried and exported when needed.",
                    "Platform entry points follow the actual UI",
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
                    "Download manually",
                    "0.9.1 is communicated by announcement only. Download the matching package manually from the website or official GitHub release.",
                    "Check version and SHA256",
                ),
                item(
                    "STEP 02",
                    "Preserve existing data",
                    "Keep data when installing Windows or replacing the Mac app. An older Android build with a different signature cannot be replaced directly; preserve data before uninstalling.",
                    "Cloud does not sync AI history",
                ),
                item(
                    "STEP 03",
                    "Add your first host",
                    "Enter an IPv4 address or domain, a separate port and credentials, then select Agent, standard SSH or the appropriate Windows connection.",
                    "Verify server identity first",
                ),
                item(
                    "STEP 04",
                    "Open terminals and monitoring",
                    "Use persistent sessions, standard terminals, live metrics and collection history as needed.",
                    "Keep existing sessions and data",
                ),
                item(
                    "STEP 05",
                    "Organize files and projects",
                    "Manage remote files and group hosts by project. Desktop users can reuse saved commands and port forwards.",
                    "Local host deletion does not uninstall the remote Agent",
                ),
                item(
                    "STEP 06",
                    "AI assistant",
                    "Configure model accounts and permissions. The official Cloud is the default; a compatible self-hosted Creation Cloud can be selected in settings.",
                    "Sync is started explicitly by the user",
                ),
            ],
        ),
        section(
            "platforms",
            HomeLayout::Platforms,
            "Windows, Mac and Android; iOS is not published",
            "0.9.1 ships for Windows, Apple Silicon Mac and Android and requires manual installation. iOS is not published; the Linux client remains frozen.",
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
                    "Direct connections and identity checks",
                    "SSH host-key and RDP certificate checks remain in place. Proxy failure never silently falls back to a direct connection. Local deletion does not contact or clean the server.",
                    "Separate control and data planes",
                ),
                item(
                    "CLOUD CONTROL PLANE",
                    "Local keys and encrypted sync",
                    "System device keys protect local credentials; account data protection and a separate key protect Cloud sync. AI history and memory stay out of Cloud, while selected context is sent to the chosen model provider.",
                    "Cloud login is optional",
                ),
            ],
        ),
        section(
            "cloud",
            HomeLayout::Cloud,
            "Accounts support devices and sync, never SSH",
            "The Creation Cloud control plane is deployed on the production server with accounts, devices, sync, model metadata, encrypted vault envelopes, releases, downloads, and administration. Production client integration ships separately.",
            vec![
                item(
                    "ACCOUNT",
                    "Optional account",
                    "Registration, login, device management and logout. Logging in neither decrypts data nor automatically starts a sync.",
                    "Official Cloud by default",
                ),
                item(
                    "DEVICE",
                    "Official or self-hosted Cloud",
                    "Enter the HTTPS URL of your own deployment of the same Creation Cloud server. Login and sync state are isolated when switching sources.",
                    "Operated by the third-party administrator",
                ),
                item(
                    "SYNC",
                    "Explicit encrypted sync",
                    "Sync hosts, proxy profiles and model-account settings. Passwords, private keys, RDP credentials and model keys are encrypted by the client before upload.",
                    "AI conversations and long-term memory are excluded",
                ),
                item(
                    "MODEL",
                    "Model accounts and bindings",
                    "Manage model accounts, custom endpoints, bindings and context settings; use tools within the chosen scope and permission level.",
                    "Use your own model service account",
                ),
                item(
                    "VAULT",
                    "Data protection",
                    "The Cloud data-protection password and local device key have separate roles. Changing Cloud does not move local hosts or chats automatically.",
                    "Confirm the target Cloud before syncing",
                ),
                item(
                    "RELEASE",
                    "Manual download and update source",
                    "0.9.1 must be downloaded manually. With a self-hosted Cloud, later checks use that source; its administrator uploads official downloads and original signatures.",
                    "Announcement only · No upgrade reminders or forced updates",
                ),
            ],
        ),
    ]
}

fn faqs() -> Vec<HomeFaqItem> {
    vec![
        HomeFaqItem::new(
            "Can older versions automatically update to 0.9.1?",
            "This release is communicated by announcement only, without upgrade reminders or forced updates. Download and install it manually, and read the installation and data-preservation guide first. The Mac automatic updater is deferred.",
        ),
        HomeFaqItem::new(
            "Which platforms are included?",
            "Windows x64 installer and portable ZIP, an arm64 DMG for Apple Silicon Mac only, and an Android arm64 APK. iOS is not published. The Linux client remains frozen; Linux server Agents remain available.",
        ),
        HomeFaqItem::new(
            "What happens to old Android data?",
            "An older installation with a different signature cannot be replaced directly. Uninstalling clears local data. Preserve needed data first; Cloud sync excludes AI conversations and memory. Keep the old app if you cannot confirm a backup.",
        ),
        HomeFaqItem::new(
            "Is the official Cloud required?",
            "No. Local features work without a Cloud login. Select a compatible self-hosted Creation Cloud in settings; resolve pending sync before switching and sign in to the selected service afterwards.",
        ),
        HomeFaqItem::new(
            "How do the three AI scopes differ?",
            "Global can access all scope histories stored on this device, project only its own project and host only its own host. It does not gain chats from another device automatically; conversations and memory are not synced through Cloud.",
        ),
        HomeFaqItem::new(
            "What should Mac users know?",
            "The DMG targets Apple Silicon M-series Macs and macOS 13 or later. It is ad-hoc signed and not notarized, so system confirmation may be required. The Mac automatic updater is deferred; install this release manually.",
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
        "PLANNED" | "iOS" => HomeTone::Planned,
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
                &platform.state,
                &platform.name,
                &platform.position,
                &platform.shell,
                if platform.planned {
                    HomeTone::Planned
                } else {
                    HomeTone::Default
                },
            )
        })
        .collect()
}
