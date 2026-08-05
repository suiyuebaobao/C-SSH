//! 提供英文用户中心与管理后台内容。

use crate::{ContentSection, Metric, NavigationItem, PageContent, PageId};

use super::en::{item, nav, page, section};

pub(super) fn console_overview() -> PageContent {
    console_page(
        PageId::Console,
        "Console | Creation Cloud",
        "Overview",
        "Your Creation Cloud control plane",
        "Device state, sync revisions, and vault wrappers appear within explicit boundaries.",
        vec![
            Metric::new("—", "Registered devices", "Available after sign-in"),
            Metric::new("—", "Sync revision", "Available after sign-in"),
            Metric::new("—", "Vault entries", "Ciphertext metadata only"),
        ],
        vec![section(
            "overview",
            "Account overview",
            "Real account state replaces every placeholder once business services are connected.",
            vec![
                item(
                    "Devices",
                    "Device state",
                    "Review registered devices and recent activity.",
                    "Awaiting session",
                ),
                item(
                    "Sync",
                    "Sync state",
                    "Review revisions, conflicts, and recent sync.",
                    "Awaiting session",
                ),
                item(
                    "Vault",
                    "Ciphertext state",
                    "Review entry versions and device wrappers without plaintext.",
                    "Awaiting session",
                ),
            ],
        )],
    )
}

pub(super) fn profile() -> PageContent {
    console_page(
        PageId::Profile,
        "Profile and security | Creation Cloud",
        "Profile",
        "Manage profile and account security",
        "Profile data belongs to the current account; password changes never access the vault master key.",
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn devices() -> PageContent {
    console_page(
        PageId::Devices,
        "Devices | Creation Cloud",
        "Devices",
        "Manage registered devices",
        "Device records identify clients and never contain SSH host details.",
        vec![Metric::new("—", "Devices", "Available after sign-in")],
        vec![section(
            "devices",
            "Device list",
            "Registration, rename, and revocation come from the device service.",
            vec![item(
                "Empty",
                "Devices are not loaded",
                "Sign in to view devices owned by the current account.",
                "No mock data",
            )],
        )],
    )
}

pub(super) fn sync() -> PageContent {
    console_page(
        PageId::Sync,
        "Sync | Creation Cloud",
        "Sync",
        "Inspect revisions and conflicts",
        "Only allowlisted, non-sensitive preferences sync; unknown fields are rejected.",
        vec![
            Metric::new("—", "Current revision", "Available after sign-in"),
            Metric::new("—", "Open conflicts", "Available after sign-in"),
        ],
        vec![section(
            "sync-state",
            "Recent sync",
            "Show namespace, revision, and outcome without sensitive bodies.",
            vec![item(
                "Empty",
                "Sync state is not loaded",
                "Connect an account to see real sync records.",
                "No mock data",
            )],
        )],
    )
}

pub(super) fn models() -> PageContent {
    console_page(
        PageId::Models,
        "Models | Creation Cloud",
        "Models",
        "Read the global model catalog",
        "Names, vendors, model IDs, API formats, and API URLs are administered globally. AI provider API keys and tokens stay only in local secure client storage; Cloud never uploads, stores, or displays their status.",
        vec![Metric::new(
            "—",
            "Available models",
            "Available after sign-in",
        )],
        vec![section(
            "model-list",
            "Model catalog",
            "The model service returns administrator-enabled global catalog entries and never accepts personal credentials.",
            vec![item(
                "Empty",
                "Models are not loaded",
                "The page never inserts sample keys, ciphertext status, or invented profiles.",
                "No mock data",
            )],
        )],
    )
}

pub(super) fn vault() -> PageContent {
    console_page(
        PageId::Vault,
        "Vault | Creation Cloud",
        "Vault",
        "Manage versioned ciphertext only",
        "Encryption and decryption happen on trusted clients; the server cannot read vault content or passwords.",
        vec![
            Metric::new("—", "Ciphertext entries", "Available after sign-in"),
            Metric::new("—", "Wrapped devices", "Available after sign-in"),
        ],
        vec![section(
            "vault-state",
            "Vault state",
            "Show only entry count, version, and device wrapping state.",
            vec![item(
                "Zero knowledge",
                "Vault state is not loaded",
                "Sign in to view ciphertext metadata for your account.",
                "No plaintext",
            )],
        )],
    )
}

pub(super) fn downloads() -> PageContent {
    console_page(
        PageId::ConsoleDownloads,
        "Downloads | Creation Cloud",
        "Downloads",
        "Review compatible releases and download history",
        "Versions and checksums come only from published records; account history shows only attributed events.",
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn admin() -> PageContent {
    admin_page(
        PageId::Admin,
        "Admin | Creation Cloud",
        "Control plane overview",
        "Verify real system state at a glance",
        "Review process, database, controlled storage, users, devices, releases, and audit totals from live services.",
    )
}

pub(super) fn admin_users() -> PageContent {
    admin_page(
        PageId::AdminUsers,
        "Users | Creation Cloud Admin",
        "Account governance",
        "Manage users and authorization boundaries",
        "Find accounts by full email or administrator login, update status and role, and protect both the current and last active administrator.",
    )
}

pub(super) fn admin_devices() -> PageContent {
    admin_page(
        PageId::AdminDevices,
        "Devices | Creation Cloud Admin",
        "Device governance",
        "Manage client device metadata only",
        "Review platform, version, and revocation state without exposing or storing any SSH host data.",
    )
}

pub(super) fn admin_releases() -> PageContent {
    admin_page(
        PageId::AdminReleases,
        "Releases | Creation Cloud Admin",
        "Release control",
        "Move releases through verified states",
        "Create releases, maintain bilingual notes, and move from draft through validation, publication, revocation, or hiding.",
    )
}

pub(super) fn admin_assets() -> PageContent {
    admin_page(
        PageId::AdminAssets,
        "Assets | Creation Cloud Admin",
        "Delivery assets",
        "Keep file identity, sources, and verification aligned",
        "Register platform assets, complete quarantined SHA256-verified uploads, and manage local or HTTPS external sources.",
    )
}

pub(super) fn admin_models() -> PageContent {
    admin_page(
        PageId::AdminModels,
        "Models | Creation Cloud Admin",
        "Models",
        "Manage the global model catalog for clients",
        "Add, edit, enable, and order global models. The admin service never receives, stores, or displays any user's AI provider API key or token status.",
    )
}

pub(super) fn admin_announcements() -> PageContent {
    admin_page(
        PageId::AdminAnnouncements,
        "Announcements | Creation Cloud Admin",
        "Announcements",
        "Edit the current announcement available to clients",
        "Maintain drafts and publish the current announcement. The anonymous API returns published content only.",
    )
}

pub(super) fn admin_settings() -> PageContent {
    admin_page(
        PageId::AdminSettings,
        "System settings | Creation Cloud Admin",
        "System settings",
        "Manage global platform settings",
        "Manage sign-in verification and other global options in one place.",
    )
}

pub(super) fn admin_site() -> PageContent {
    admin_page(
        PageId::AdminSite,
        "Home content | Creation Cloud Admin",
        "Home content",
        "Manage home-page content and QR media",
        "Edit and publish Chinese and English home-page content and controlled QR media.",
    )
}

pub(super) fn admin_seo() -> PageContent {
    admin_page(
        PageId::AdminSeo,
        "SEO topics | Creation Cloud Admin",
        "SEO topics",
        "Maintain visible search themes for public pages",
        "Manage Chinese and English topics, visibility, and ordering. Topics appear naturally in crawlable copy; meta keywords remain a compatibility-only projection.",
    )
}

pub(super) fn admin_audit() -> PageContent {
    admin_page(
        PageId::AdminAudit,
        "Audit | Creation Cloud Admin",
        "Security audit",
        "Make every administrative action traceable",
        "Review server-generated actor, action, resource, outcome, and redacted request identifiers in chronological order.",
    )
}

pub(super) fn admin_feedback() -> PageContent {
    admin_page(
        PageId::AdminFeedback,
        "Feedback | Creation Cloud Admin",
        "Feedback handling",
        "Review and advance website feedback",
        "The list exposes only a minimal summary. Full plain-text content appears only after an administrator opens a record explicitly.",
    )
}

fn console_page(
    id: PageId,
    meta_title: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    lead: &'static str,
    metrics: Vec<Metric>,
    sections: Vec<ContentSection>,
) -> PageContent {
    page(id, meta_title, lead, eyebrow, heading, lead)
        .with_metrics(metrics)
        .with_sections(sections)
        .with_local_navigation(console_navigation(id))
}

fn console_navigation(current: PageId) -> Vec<NavigationItem> {
    vec![
        nav("Overview", PageId::Console, current),
        nav("Profile", PageId::Profile, current),
        nav("Devices", PageId::Devices, current),
        nav("Hosts", PageId::Sync, current),
        nav("Models", PageId::Models, current),
        nav("Downloads", PageId::ConsoleDownloads, current),
    ]
}

fn admin_page(
    id: PageId,
    meta_title: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    lead: &'static str,
) -> PageContent {
    page(id, meta_title, lead, eyebrow, heading, lead).with_local_navigation(admin_navigation(id))
}

fn admin_navigation(current: PageId) -> Vec<NavigationItem> {
    vec![
        nav("Users", PageId::AdminUsers, current),
        nav("Home content", PageId::AdminSite, current),
        nav("Client updates", PageId::AdminReleases, current),
        nav("Downloads", PageId::AdminAssets, current),
        nav("Models", PageId::AdminModels, current),
        nav("Announcements", PageId::AdminAnnouncements, current),
        nav("Feedback", PageId::AdminFeedback, current),
        nav("SEO", PageId::AdminSeo, current),
        nav("System settings", PageId::AdminSettings, current),
        nav("Activity", PageId::AdminAudit, current),
    ]
}
