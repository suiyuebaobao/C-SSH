//! 提供英文网站壳并装配分层内容目录。

use crate::{
    Action, ContentItem, ContentSection, Locale, NavigationItem, PageContent, PageId,
    RepositoryLink, SiteShell, SiteView,
};

const LOCALE: Locale = Locale::En;

pub(super) fn view(page: PageId) -> SiteView {
    SiteView {
        shell: shell(page),
        page: match page {
            PageId::Home => super::en_home::page_content(),
            PageId::Tutorials => super::en_tutorials::page_content(),
            PageId::Documentation => super::en_documentation::page_content(),
            PageId::Security => super::en_public::security(),
            PageId::Downloads => super::en_public::downloads(),
            PageId::Changelog => super::en_public::changelog(),
            PageId::Faq => super::en_public::faq(),
            PageId::Feedback => super::en_feedback::page_content(),
            PageId::Login => super::en_account::login(),
            PageId::Register => super::en_account::register(),
            PageId::Console => super::en_workspace::console_overview(),
            PageId::Devices => super::en_workspace::devices(),
            PageId::Sync => super::en_workspace::sync(),
            PageId::Models => super::en_workspace::models(),
            PageId::Vault => super::en_workspace::vault(),
            PageId::Admin => super::en_workspace::admin(),
            PageId::AdminUsers => super::en_workspace::admin_users(),
            PageId::AdminDevices => super::en_workspace::admin_devices(),
            PageId::AdminReleases => super::en_workspace::admin_releases(),
            PageId::AdminAssets => super::en_workspace::admin_assets(),
            PageId::AdminSite => super::en_workspace::admin_site(),
            PageId::AdminAudit => super::en_workspace::admin_audit(),
            PageId::AdminFeedback => super::en_workspace::admin_feedback(),
        },
    }
}

fn shell(current: PageId) -> SiteShell {
    let mut console_link = nav("Console", PageId::Console, current);
    console_link.active = matches!(
        current,
        PageId::Console | PageId::Devices | PageId::Sync | PageId::Models | PageId::Vault
    );
    let mut login_link = nav("Sign in", PageId::Login, current);
    login_link.active = matches!(current, PageId::Login | PageId::Register);
    SiteShell {
        locale: LOCALE,
        html_lang: LOCALE.code(),
        brand: "Creation-SSH",
        brand_note: "Native SSH operations workspace",
        home_href: PageId::Home.localized_path(LOCALE),
        skip_label: "Skip to main content",
        menu_label: "Menu",
        navigation: vec![
            nav("Home", PageId::Home, current),
            nav("Docs", PageId::Documentation, current),
            nav("Changelog", PageId::Changelog, current),
            nav("Tutorials", PageId::Tutorials, current),
            nav("Security", PageId::Security, current),
            nav("Downloads", PageId::Downloads, current),
            nav("FAQ", PageId::Faq, current),
            nav("Feedback", PageId::Feedback, current),
        ],
        console_link,
        login_link,
        language_label: "中文",
        language_href: current.path().to_owned(),
        alternate_lang: Locale::ZhCn.code(),
        utility_navigation_label: "Utility",
        github_repository: RepositoryLink::github(
            "Open the Creation-SSH GitHub repository in a new tab",
        ),
        footer_summary: "Cross-platform SSH operations built around a native client and resident agent.",
        footer_motto: "岁月牵涉年华，流年纠结浮生",
        footer_note: "Creation Cloud is a control plane and never proxies the SSH data plane.",
        footer_navigation: vec![
            nav("Security", PageId::Security, current),
            nav("Changelog", PageId::Changelog, current),
            nav("FAQ", PageId::Faq, current),
            nav("Feedback", PageId::Feedback, current),
        ],
    }
}

pub(super) fn page(
    id: PageId,
    meta_title: &'static str,
    meta_description: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    lead: &'static str,
) -> PageContent {
    PageContent::new(id, meta_title, meta_description, eyebrow, heading, lead)
}

pub(super) fn action(label: &'static str, href: &'static str, class_name: &'static str) -> Action {
    Action::new(label, href, class_name, LOCALE)
}

pub(super) fn nav(label: &'static str, target: PageId, current: PageId) -> NavigationItem {
    NavigationItem::new(label, target, current, LOCALE)
}

pub(super) const fn item(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    meta: &'static str,
) -> ContentItem {
    ContentItem::new(badge, title, body, meta)
}

pub(super) fn section(
    anchor: &'static str,
    title: &'static str,
    lead: &'static str,
    items: Vec<ContentItem>,
) -> ContentSection {
    ContentSection::new(anchor, title, lead, items)
}
