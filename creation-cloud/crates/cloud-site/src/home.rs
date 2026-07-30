//! 定义首页专用的平台、系统区块、状态与常见问题模型。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::HomeQrWidget;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomePageContent {
    pub status_strip_label: String,
    pub status_note: String,
    pub hero_blueprint_label: String,
    pub platform_label: String,
    pub platform_note: String,
    pub platforms: Vec<HomePlatform>,
    pub sections: Vec<HomeSection>,
    pub faq_side_label: String,
    pub faq_item_prefix: String,
    pub faq_code: String,
    pub faq_heading: String,
    pub faq_lead: String,
    pub faqs: Vec<HomeFaqItem>,
    pub seo_code: String,
    pub seo_heading: String,
    pub seo_topics_label: String,
    pub final_code: String,
    pub final_heading: String,
    pub final_lead: String,
    pub qr_placeholder_code: String,
    pub qr_placeholder_waiting: String,
    pub media_slot: Option<String>,
    pub qr_widget: HomeQrWidget,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomePlatform {
    pub symbol: String,
    pub name: String,
    pub detail: String,
    pub position: String,
    pub shell: String,
    pub state: String,
    pub planned: bool,
}

impl HomePlatform {
    #[must_use]
    pub fn current(
        symbol: &str,
        name: &str,
        detail: &str,
        position: &str,
        shell: &str,
        state: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_owned(),
            name: name.to_owned(),
            detail: detail.to_owned(),
            position: position.to_owned(),
            shell: shell.to_owned(),
            state: state.to_owned(),
            planned: false,
        }
    }

    #[must_use]
    pub fn planned(
        symbol: &str,
        name: &str,
        detail: &str,
        position: &str,
        shell: &str,
        state: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_owned(),
            name: name.to_owned(),
            detail: detail.to_owned(),
            position: position.to_owned(),
            shell: shell.to_owned(),
            state: state.to_owned(),
            planned: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeSection {
    pub anchor: String,
    pub code: String,
    pub side_label: String,
    pub layout: HomeLayout,
    pub title: String,
    pub lead: String,
    pub items: Vec<HomeItem>,
}

impl HomeSection {
    #[must_use]
    pub fn new(
        anchor: &str,
        code: &str,
        side_label: &str,
        layout: HomeLayout,
        title: &str,
        lead: &str,
        items: Vec<HomeItem>,
    ) -> Self {
        Self {
            anchor: anchor.to_owned(),
            code: code.to_owned(),
            side_label: side_label.to_owned(),
            layout,
            title: title.to_owned(),
            lead: lead.to_owned(),
            items,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeItem {
    pub badge: String,
    pub title: String,
    pub body: String,
    pub meta: String,
    pub tone: HomeTone,
}

impl HomeItem {
    #[must_use]
    pub fn new(badge: &str, title: &str, body: &str, meta: &str, tone: HomeTone) -> Self {
        Self {
            badge: badge.to_owned(),
            title: title.to_owned(),
            body: body.to_owned(),
            meta: meta.to_owned(),
            tone,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeLayout {
    Workflow,
    Capabilities,
    Steps,
    Platforms,
    Security,
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
            Self::Cloud => "cloud",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeFaqItem {
    pub question: String,
    pub answer: String,
}

impl HomeFaqItem {
    #[must_use]
    pub fn new(question: &str, answer: &str) -> Self {
        Self {
            question: question.to_owned(),
            answer: answer.to_owned(),
        }
    }
}
