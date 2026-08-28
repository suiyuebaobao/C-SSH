use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AnnouncementLocale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl AnnouncementLocale {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnouncementStatus {
    Draft,
    Published,
    Hidden,
}

impl AnnouncementStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Hidden => "hidden",
        }
    }
}

impl fmt::Display for AnnouncementStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for AnnouncementStatus {
    type Err = AppError;

    fn from_str(value: &str) -> AppResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "hidden" => Ok(Self::Hidden),
            _ => Err(AppError::Internal("数据库中的公告状态无效".to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnouncementPriority {
    Normal,
    Important,
    Critical,
}

impl AnnouncementPriority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Important => "important",
            Self::Critical => "critical",
        }
    }
}

impl fmt::Display for AnnouncementPriority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for AnnouncementPriority {
    type Err = AppError;

    fn from_str(value: &str) -> AppResult<Self> {
        match value {
            "normal" => Ok(Self::Normal),
            "important" => Ok(Self::Important),
            "critical" => Ok(Self::Critical),
            _ => Err(AppError::Internal("数据库中的公告优先级无效".to_owned())),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Announcement {
    pub id: Uuid,
    pub title_zh_cn: String,
    pub body_zh_cn: String,
    pub title_en: String,
    pub body_en: String,
    pub priority: AnnouncementPriority,
    pub status: AnnouncementStatus,
    pub revision: i64,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct AnnouncementRow {
    pub id: Uuid,
    pub title_zh_cn: String,
    pub body_zh_cn: String,
    pub title_en: String,
    pub body_en: String,
    pub priority: String,
    pub status: String,
    pub revision: i64,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, FromRow)]
pub(crate) struct PublicationStateRow {
    pub public_revision: i64,
    pub current_announcement_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub(crate) struct CurrentPublication {
    pub public_revision: i64,
    pub announcement: Option<Announcement>,
}

impl TryFrom<AnnouncementRow> for Announcement {
    type Error = AppError;

    fn try_from(row: AnnouncementRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            title_zh_cn: row.title_zh_cn,
            body_zh_cn: row.body_zh_cn,
            title_en: row.title_en,
            body_en: row.body_en,
            priority: row.priority.parse()?,
            status: row.status.parse()?,
            revision: row.revision,
            created_by: row.created_by,
            updated_by: row.updated_by,
            published_at: row.published_at,
            hidden_at: row.hidden_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAnnouncementInput {
    pub title_zh_cn: String,
    pub body_zh_cn: String,
    pub title_en: String,
    pub body_en: String,
    pub priority: AnnouncementPriority,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplaceAnnouncementInput {
    pub expected_revision: i64,
    pub title_zh_cn: String,
    pub body_zh_cn: String,
    pub title_en: String,
    pub body_en: String,
    pub priority: AnnouncementPriority,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionAnnouncementInput {
    pub expected_revision: i64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentAnnouncementQuery {
    pub locale: AnnouncementLocale,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicAnnouncement {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub priority: AnnouncementPriority,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurrentAnnouncementResponse {
    pub revision: i64,
    pub announcement: Option<PublicAnnouncement>,
}

impl Announcement {
    pub(crate) fn localized(&self, locale: AnnouncementLocale) -> AppResult<PublicAnnouncement> {
        let published_at = self
            .published_at
            .ok_or_else(|| AppError::Internal("已发布公告缺少发布时间".to_owned()))?;
        let (title, content) = match locale {
            AnnouncementLocale::ZhCn => (&self.title_zh_cn, &self.body_zh_cn),
            AnnouncementLocale::En => (&self.title_en, &self.body_en),
        };
        Ok(PublicAnnouncement {
            id: self.id,
            title: title.clone(),
            content: content.clone(),
            priority: self.priority,
            published_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedAnnouncement {
    pub title_zh_cn: String,
    pub body_zh_cn: String,
    pub title_en: String,
    pub body_en: String,
    pub priority: AnnouncementPriority,
}
