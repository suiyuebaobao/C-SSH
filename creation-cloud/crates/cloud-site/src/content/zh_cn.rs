//! 提供简体中文网站壳并装配分层内容目录。

use crate::{
    Action, ContentItem, ContentSection, Locale, NavigationItem, PageContent, PageId,
    RepositoryLink, SiteShell, SiteView,
};

const LOCALE: Locale = Locale::ZhCn;

pub(super) fn view(page: PageId) -> SiteView {
    SiteView {
        shell: shell(page),
        page: match page {
            PageId::Home => super::zh_cn_home::page_content(),
            PageId::Tutorials => super::zh_cn_tutorials::page_content(),
            PageId::Documentation => super::zh_cn_documentation::page_content(),
            PageId::Security => super::zh_cn_public::security(),
            PageId::Downloads => super::zh_cn_public::downloads(),
            PageId::Changelog => super::zh_cn_public::changelog(),
            PageId::Faq => super::zh_cn_public::faq(),
            PageId::Feedback => super::zh_cn_feedback::page_content(),
            PageId::Login => super::zh_cn_account::login(),
            PageId::Register => super::zh_cn_account::register(),
            PageId::Console => super::zh_cn_workspace::console_overview(),
            PageId::Devices => super::zh_cn_workspace::devices(),
            PageId::Sync => super::zh_cn_workspace::sync(),
            PageId::Models => super::zh_cn_workspace::models(),
            PageId::Vault => super::zh_cn_workspace::vault(),
            PageId::Admin => super::zh_cn_workspace::admin(),
            PageId::AdminUsers => super::zh_cn_workspace::admin_users(),
            PageId::AdminDevices => super::zh_cn_workspace::admin_devices(),
            PageId::AdminReleases => super::zh_cn_workspace::admin_releases(),
            PageId::AdminAssets => super::zh_cn_workspace::admin_assets(),
            PageId::AdminSite => super::zh_cn_workspace::admin_site(),
            PageId::AdminAudit => super::zh_cn_workspace::admin_audit(),
            PageId::AdminFeedback => super::zh_cn_workspace::admin_feedback(),
        },
    }
}

fn shell(current: PageId) -> SiteShell {
    let mut console_link = nav("用户中心", PageId::Console, current);
    console_link.active = matches!(
        current,
        PageId::Console | PageId::Devices | PageId::Sync | PageId::Models | PageId::Vault
    );
    let mut login_link = nav("登录", PageId::Login, current);
    login_link.active = matches!(current, PageId::Login | PageId::Register);
    SiteShell {
        locale: LOCALE,
        html_lang: LOCALE.code(),
        brand: "Creation-SSH",
        brand_note: "原生 SSH 运维工作台",
        home_href: PageId::Home.path().to_owned(),
        skip_label: "跳到主要内容",
        menu_label: "菜单",
        navigation: vec![
            nav("首页", PageId::Home, current),
            nav("文档", PageId::Documentation, current),
            nav("更新日志", PageId::Changelog, current),
            nav("教程", PageId::Tutorials, current),
            nav("安全", PageId::Security, current),
            nav("下载", PageId::Downloads, current),
            nav("常见问题", PageId::Faq, current),
            nav("问题反馈", PageId::Feedback, current),
        ],
        console_link,
        login_link,
        language_label: "English",
        language_href: current.localized_path(Locale::En),
        alternate_lang: Locale::En.code(),
        utility_navigation_label: "快捷入口",
        github_repository: RepositoryLink::github("在新标签页打开 Creation-SSH GitHub 仓库"),
        footer_summary: "客户端与常驻 agent 协作的跨平台 SSH 运维产品。",
        footer_motto: "岁月牵涉年华，流年纠结浮生",
        footer_note: "Creation Cloud 只做控制面，不代理 SSH 数据面。",
        footer_navigation: vec![
            nav("安全", PageId::Security, current),
            nav("更新记录", PageId::Changelog, current),
            nav("常见问题", PageId::Faq, current),
            nav("问题反馈", PageId::Feedback, current),
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
