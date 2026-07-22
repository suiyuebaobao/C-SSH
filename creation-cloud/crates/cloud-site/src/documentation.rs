//! 定义官网文档中心的目录、平台状态、实操指南、正文与脱敏界面证据模型。

use crate::TutorialContent;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationContent {
    pub release_label: &'static str,
    pub release_version: &'static str,
    pub release_date: &'static str,
    pub release_href: &'static str,
    pub release_action_label: &'static str,
    pub index_label: &'static str,
    pub mobile_index_label: &'static str,
    pub search_label: &'static str,
    pub search_placeholder: &'static str,
    pub search_help: &'static str,
    pub search_empty: &'static str,
    pub status: DocumentationNotice,
    pub groups: Vec<DocumentationGroup>,
    pub platform_code: &'static str,
    pub platform_title: &'static str,
    pub platform_lead: &'static str,
    pub platforms: Vec<DocumentationPlatform>,
    pub tutorials: TutorialContent,
    pub sections: Vec<DocumentationSection>,
    pub screenshot: DocumentationScreenshot,
    pub final_code: &'static str,
    pub final_title: &'static str,
    pub final_body: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationNotice {
    pub label: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

impl DocumentationNotice {
    #[must_use]
    pub const fn new(label: &'static str, title: &'static str, body: &'static str) -> Self {
        Self { label, title, body }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationGroup {
    pub title: &'static str,
    pub links: Vec<DocumentationLink>,
}

impl DocumentationGroup {
    #[must_use]
    pub fn new(title: &'static str, links: Vec<DocumentationLink>) -> Self {
        Self { title, links }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationLink {
    pub anchor: &'static str,
    pub code: &'static str,
    pub title: &'static str,
}

impl DocumentationLink {
    #[must_use]
    pub const fn new(anchor: &'static str, code: &'static str, title: &'static str) -> Self {
        Self {
            anchor,
            code,
            title,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationPlatform {
    pub symbol: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub state: &'static str,
    pub assets: &'static str,
    pub detail: &'static str,
    pub release_href: &'static str,
    pub planned: bool,
}

impl DocumentationPlatform {
    #[must_use]
    pub const fn released(
        symbol: &'static str,
        name: &'static str,
        role: &'static str,
        state: &'static str,
        assets: &'static str,
        detail: &'static str,
        release_href: &'static str,
    ) -> Self {
        Self {
            symbol,
            name,
            role,
            state,
            assets,
            detail,
            release_href,
            planned: false,
        }
    }

    #[must_use]
    pub const fn planned(
        symbol: &'static str,
        name: &'static str,
        role: &'static str,
        state: &'static str,
        detail: &'static str,
    ) -> Self {
        Self {
            symbol,
            name,
            role,
            state,
            assets: "",
            detail,
            release_href: "",
            planned: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationSection {
    pub anchor: &'static str,
    pub code: &'static str,
    pub title: &'static str,
    pub lead: &'static str,
    pub items: Vec<DocumentationItem>,
}

impl DocumentationSection {
    #[must_use]
    pub fn new(
        anchor: &'static str,
        code: &'static str,
        title: &'static str,
        lead: &'static str,
        items: Vec<DocumentationItem>,
    ) -> Self {
        Self {
            anchor,
            code,
            title,
            lead,
            items,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationItem {
    pub badge: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub command: &'static str,
    pub has_command: bool,
    pub caution: bool,
}

impl DocumentationItem {
    #[must_use]
    pub const fn text(
        badge: &'static str,
        title: &'static str,
        body: &'static str,
        caution: bool,
    ) -> Self {
        Self {
            badge,
            title,
            body,
            command: "",
            has_command: false,
            caution,
        }
    }

    #[must_use]
    pub const fn command(
        badge: &'static str,
        title: &'static str,
        body: &'static str,
        command: &'static str,
        caution: bool,
    ) -> Self {
        Self {
            badge,
            title,
            body,
            command,
            has_command: true,
            caution,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationScreenshot {
    pub code: &'static str,
    pub title: &'static str,
    pub lead: &'static str,
    pub src: &'static str,
    pub alt: &'static str,
    pub caption: &'static str,
    pub width: u16,
    pub height: u16,
}
