//! 定义首页专用的平台、系统区块、状态与常见问题模型。

use std::fmt;

use crate::HomeQrWidget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomePageContent {
    pub status_note: &'static str,
    pub platform_label: &'static str,
    pub platform_note: &'static str,
    pub platforms: Vec<HomePlatform>,
    pub sections: Vec<HomeSection>,
    pub faq_code: &'static str,
    pub faq_heading: &'static str,
    pub faq_lead: &'static str,
    pub faqs: Vec<HomeFaqItem>,
    pub final_code: &'static str,
    pub final_heading: &'static str,
    pub final_lead: &'static str,
    pub qr_widget: HomeQrWidget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomePlatform {
    pub symbol: &'static str,
    pub name: &'static str,
    pub detail: &'static str,
    pub position: &'static str,
    pub shell: &'static str,
    pub state: &'static str,
    pub planned: bool,
}

impl HomePlatform {
    #[must_use]
    pub const fn current(
        symbol: &'static str,
        name: &'static str,
        detail: &'static str,
        position: &'static str,
        shell: &'static str,
        state: &'static str,
    ) -> Self {
        Self {
            symbol,
            name,
            detail,
            position,
            shell,
            state,
            planned: false,
        }
    }

    #[must_use]
    pub const fn planned(
        symbol: &'static str,
        name: &'static str,
        detail: &'static str,
        position: &'static str,
        shell: &'static str,
        state: &'static str,
    ) -> Self {
        Self {
            symbol,
            name,
            detail,
            position,
            shell,
            state,
            planned: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeSection {
    pub anchor: &'static str,
    pub code: &'static str,
    pub side_label: &'static str,
    pub layout: HomeLayout,
    pub title: &'static str,
    pub lead: &'static str,
    pub items: Vec<HomeItem>,
}

impl HomeSection {
    #[must_use]
    pub fn new(
        anchor: &'static str,
        code: &'static str,
        side_label: &'static str,
        layout: HomeLayout,
        title: &'static str,
        lead: &'static str,
        items: Vec<HomeItem>,
    ) -> Self {
        Self {
            anchor,
            code,
            side_label,
            layout,
            title,
            lead,
            items,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeItem {
    pub badge: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub meta: &'static str,
    pub tone: HomeTone,
}

impl HomeItem {
    #[must_use]
    pub const fn new(
        badge: &'static str,
        title: &'static str,
        body: &'static str,
        meta: &'static str,
        tone: HomeTone,
    ) -> Self {
        Self {
            badge,
            title,
            body,
            meta,
            tone,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeLayout {
    Workflow,
    Capabilities,
    Steps,
    Platforms,
    Security,
    Downloads,
    Cloud,
}

impl fmt::Display for HomeLayout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Workflow => "workflow",
            Self::Capabilities => "capabilities",
            Self::Steps => "steps",
            Self::Platforms => "platforms",
            Self::Security => "security",
            Self::Downloads => "downloads",
            Self::Cloud => "cloud",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeTone {
    Default,
    Dark,
    Accent,
    Planned,
}

impl fmt::Display for HomeTone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Default => "default",
            Self::Dark => "dark",
            Self::Accent => "accent",
            Self::Planned => "planned",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeFaqItem {
    pub question: &'static str,
    pub answer: &'static str,
}

impl HomeFaqItem {
    #[must_use]
    pub const fn new(question: &'static str, answer: &'static str) -> Self {
        Self { question, answer }
    }
}
