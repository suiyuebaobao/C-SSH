use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_site::{EditableHomePage, EditableSiteShell, Locale};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteContentDocumentKey {
    SiteShell,
    Home,
}

impl SiteContentDocumentKey {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SiteShell => "site_shell",
            Self::Home => "home",
        }
    }
}

impl fmt::Display for SiteContentDocumentKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for SiteContentDocumentKey {
    type Err = AppError;

    fn from_str(value: &str) -> AppResult<Self> {
        match value {
            "site_shell" => Ok(Self::SiteShell),
            "home" => Ok(Self::Home),
            _ => Err(AppError::Validation("站点内容文档类型无效".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SiteContentState {
    Draft,
    Published,
    Revoked,
}

impl SiteContentState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Revoked => "revoked",
        }
    }
}

impl FromStr for SiteContentState {
    type Err = AppError;

    fn from_str(value: &str) -> AppResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "revoked" => Ok(Self::Revoked),
            _ => Err(AppError::Internal("数据库中的站点内容状态无效".into())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "document_key", content = "content", rename_all = "snake_case")]
pub enum SiteContentPayload {
    SiteShell(Box<EditableSiteShell>),
    Home(Box<EditableHomePage>),
}

#[derive(Clone, Debug)]
pub enum PublicSiteContent {
    LegacyFallback,
    Published(SiteContentPayload),
    Unavailable,
}

impl SiteContentPayload {
    #[must_use]
    pub const fn key(&self) -> SiteContentDocumentKey {
        match self {
            Self::SiteShell(_) => SiteContentDocumentKey::SiteShell,
            Self::Home(_) => SiteContentDocumentKey::Home,
        }
    }

    #[must_use]
    pub fn compiled(key: SiteContentDocumentKey, locale: Locale) -> Self {
        match key {
            SiteContentDocumentKey::SiteShell => {
                Self::SiteShell(Box::new(cloud_site::compiled_site_shell(locale)))
            }
            SiteContentDocumentKey::Home => {
                Self::Home(Box::new(cloud_site::compiled_home_page(locale)))
            }
        }
    }

    pub fn apply(self, view: &mut cloud_site::SiteView) {
        match self {
            Self::SiteShell(document) => cloud_site::apply_site_shell(view, *document),
            Self::Home(document) => cloud_site::apply_home_page(view, *document),
        }
    }

    #[must_use]
    pub fn media_slot(&self) -> Option<&str> {
        match self {
            Self::Home(document) => document.content.media_slot.as_deref(),
            Self::SiteShell(_) => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteContentRevision {
    pub id: Uuid,
    pub document_key: SiteContentDocumentKey,
    pub locale: Locale,
    pub state: SiteContentState,
    pub revision: i64,
    pub content: SiteContentPayload,
    pub created_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct SiteContentRow {
    pub id: Uuid,
    pub document_key: String,
    pub locale: String,
    pub state: String,
    pub revision: i64,
    pub content_json: Value,
    pub created_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<SiteContentRow> for SiteContentRevision {
    type Error = AppError;

    fn try_from(row: SiteContentRow) -> AppResult<Self> {
        let document_key = row.document_key.parse()?;
        let content: SiteContentPayload = serde_json::from_value(row.content_json)
            .map_err(|_| AppError::Internal("数据库中的站点内容结构无效".into()))?;
        if content.key() != document_key {
            return Err(AppError::Internal("站点内容类型与数据库身份不一致".into()));
        }
        Ok(Self {
            id: row.id,
            document_key,
            locale: locale_from_db(&row.locale)?,
            state: row.state.parse()?,
            revision: row.revision,
            content,
            created_by: row.created_by,
            published_at: row.published_at,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSiteContentInput {
    pub document_key: SiteContentDocumentKey,
    pub locale: Locale,
    pub content: Option<SiteContentPayload>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSiteContentInput {
    pub expected_revision: i64,
    pub content: SiteContentPayload,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteContentTransitionInput {
    pub expected_revision: i64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteContentListQuery {
    pub document_key: Option<SiteContentDocumentKey>,
    pub locale: Option<Locale>,
}

pub(crate) fn locale_from_db(value: &str) -> AppResult<Locale> {
    match value {
        "zh-CN" => Ok(Locale::ZhCn),
        "en" => Ok(Locale::En),
        _ => Err(AppError::Internal("数据库中的站点内容语种无效".into())),
    }
}
