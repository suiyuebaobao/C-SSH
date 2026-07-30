//! 定义可持久化的首页业务内容快照，并与运行时页面模型双向投影。

use serde::{Deserialize, Serialize};

use crate::{
    Action, HomePageContent, Locale, NavigationItem, PageId, RepositoryLink, SiteView,
    content_service,
};

pub const SITE_CONTENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditableLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditableSiteShell {
    pub schema_version: u32,
    pub brand_note: String,
    pub skip_label: String,
    pub menu_label: String,
    pub navigation: Vec<EditableLink>,
    pub console_label: String,
    pub login_label: String,
    pub language_label: String,
    pub utility_navigation_label: String,
    pub github: EditableLink,
    pub github_aria_label: String,
    pub footer_summary: String,
    pub footer_signature: String,
    pub footer_note: String,
    pub footer_navigation: Vec<EditableLink>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditableHomePage {
    pub schema_version: u32,
    pub meta_title: String,
    pub meta_description: String,
    pub eyebrow: String,
    pub heading: String,
    pub lead: String,
    pub actions: Vec<EditableLink>,
    pub content: HomePageContent,
}

#[must_use]
pub fn compiled_site_shell(locale: Locale) -> EditableSiteShell {
    let view = content_service().view(PageId::Home, locale);
    EditableSiteShell::from_view(&view)
}

#[must_use]
pub fn compiled_home_page(locale: Locale) -> EditableHomePage {
    let view = content_service().view(PageId::Home, locale);
    EditableHomePage::from_view(&view)
}

pub fn apply_site_shell(view: &mut SiteView, document: EditableSiteShell) {
    let current_path = view.page.id.localized_path(view.shell.locale);
    view.shell.brand_note = document.brand_note;
    view.shell.skip_label = document.skip_label;
    view.shell.menu_label = document.menu_label;
    view.shell.navigation = document
        .navigation
        .into_iter()
        .map(|item| navigation_item(item, &current_path))
        .collect();
    view.shell.console_link.label = document.console_label;
    view.shell.login_link.label = document.login_label;
    view.shell.language_label = document.language_label;
    view.shell.utility_navigation_label = document.utility_navigation_label;
    view.shell.github_repository = RepositoryLink {
        label: document.github.label,
        href: document.github.href,
        aria_label: document.github_aria_label,
    };
    view.shell.footer_summary = document.footer_summary;
    view.shell.footer_motto = document.footer_signature;
    view.shell.footer_note = document.footer_note;
    view.shell.footer_navigation = document
        .footer_navigation
        .into_iter()
        .map(|item| navigation_item(item, &current_path))
        .collect();
}

pub fn apply_home_page(view: &mut SiteView, document: EditableHomePage) {
    view.page.meta_title = document.meta_title;
    view.page.meta_description = document.meta_description;
    view.page.eyebrow = document.eyebrow;
    view.page.heading = document.heading;
    view.page.lead = document.lead;
    view.page.actions = document
        .actions
        .into_iter()
        .enumerate()
        .map(|(index, item)| Action {
            label: item.label,
            href: item.href,
            class_name: action_class(index).to_owned(),
        })
        .collect();
    view.page.home_page = Some(document.content);
}

impl EditableSiteShell {
    #[must_use]
    pub fn from_view(view: &SiteView) -> Self {
        Self {
            schema_version: SITE_CONTENT_SCHEMA_VERSION,
            brand_note: view.shell.brand_note.clone(),
            skip_label: view.shell.skip_label.clone(),
            menu_label: view.shell.menu_label.clone(),
            navigation: view
                .shell
                .navigation
                .iter()
                .map(EditableLink::from)
                .collect(),
            console_label: view.shell.console_link.label.clone(),
            login_label: view.shell.login_link.label.clone(),
            language_label: view.shell.language_label.clone(),
            utility_navigation_label: view.shell.utility_navigation_label.clone(),
            github: EditableLink {
                label: view.shell.github_repository.label.clone(),
                href: view.shell.github_repository.href.clone(),
            },
            github_aria_label: view.shell.github_repository.aria_label.clone(),
            footer_summary: view.shell.footer_summary.clone(),
            footer_signature: view.shell.footer_motto.clone(),
            footer_note: view.shell.footer_note.clone(),
            footer_navigation: view
                .shell
                .footer_navigation
                .iter()
                .map(EditableLink::from)
                .collect(),
        }
    }
}

impl EditableHomePage {
    #[must_use]
    pub fn from_view(view: &SiteView) -> Self {
        Self {
            schema_version: SITE_CONTENT_SCHEMA_VERSION,
            meta_title: view.page.meta_title.clone(),
            meta_description: view.page.meta_description.clone(),
            eyebrow: view.page.eyebrow.clone(),
            heading: view.page.heading.clone(),
            lead: view.page.lead.clone(),
            actions: view.page.actions.iter().map(EditableLink::from).collect(),
            content: view
                .page
                .home_page
                .clone()
                .expect("编译期首页必须包含首页结构"),
        }
    }
}

impl From<&NavigationItem> for EditableLink {
    fn from(value: &NavigationItem) -> Self {
        Self {
            label: value.label.clone(),
            href: value.href.clone(),
        }
    }
}

impl From<&Action> for EditableLink {
    fn from(value: &Action) -> Self {
        Self {
            label: value.label.clone(),
            href: value.href.clone(),
        }
    }
}

fn navigation_item(item: EditableLink, current_path: &str) -> NavigationItem {
    let comparable = item.href.split('#').next().unwrap_or(item.href.as_str());
    NavigationItem {
        label: item.label,
        active: comparable == current_path,
        href: item.href,
    }
}

const fn action_class(index: usize) -> &'static str {
    match index {
        0 => "button button-primary",
        1 => "button button-secondary",
        _ => "text-link",
    }
}
