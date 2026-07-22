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
        "Sync model configuration metadata",
        "Names, providers, model IDs, and defaults can sync; API keys remain vault ciphertext references.",
        vec![Metric::new(
            "—",
            "Model profiles",
            "Available after sign-in",
        )],
        vec![section(
            "model-list",
            "Model profiles",
            "Defaults, order, and enablement arrive through the model service.",
            vec![item(
                "Empty",
                "Models are not loaded",
                "The page never inserts sample keys or invented profiles.",
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
        "Find accounts by redacted identity, update status and role, and protect both the current and last active administrator.",
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

pub(super) fn admin_site() -> PageContent {
    admin_page(
        PageId::AdminSite,
        "Site media | Creation Cloud Admin",
        "Site media",
        "Manage the home QR publication slot",
        "Upload controlled bitmaps, review drafts, and publish same-origin media while the home page retains its honest empty state when none is live.",
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
        nav("Sync", PageId::Sync, current),
        nav("Models", PageId::Models, current),
        nav("Vault", PageId::Vault, current),
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
        nav("00 Overview", PageId::Admin, current),
        nav("10 Users", PageId::AdminUsers, current),
        nav("20 Devices", PageId::AdminDevices, current),
        nav("30 Releases", PageId::AdminReleases, current),
        nav("40 Assets", PageId::AdminAssets, current),
        nav("50 Site", PageId::AdminSite, current),
        nav("60 Audit", PageId::AdminAudit, current),
        nav("70 Feedback", PageId::AdminFeedback, current),
    ]
}
