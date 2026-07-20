//! 定义版本状态机、渠道、持久化行和管理输入。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Nightly,
}

impl ReleaseChannel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Nightly => "nightly",
        }
    }
}

impl TryFrom<&str> for ReleaseChannel {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "nightly" => Ok(Self::Nightly),
            _ => Err(AppError::Internal("数据库中的发布渠道无效".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseStatus {
    Draft,
    Validating,
    Published,
    Revoked,
    Hidden,
}

impl ReleaseStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Validating => "validating",
            Self::Published => "published",
            Self::Revoked => "revoked",
            Self::Hidden => "hidden",
        }
    }

    #[must_use]
    pub const fn allows_asset_mutation(self) -> bool {
        matches!(self, Self::Draft | Self::Validating)
    }
}

impl TryFrom<&str> for ReleaseStatus {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "validating" => Ok(Self::Validating),
            "published" => Ok(Self::Published),
            "revoked" => Ok(Self::Revoked),
            "hidden" => Ok(Self::Hidden),
            _ => Err(AppError::Internal("数据库中的发布状态无效".into())),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Release {
    pub id: Uuid,
    pub version: String,
    pub channel: ReleaseChannel,
    pub status: ReleaseStatus,
    pub title_zh: String,
    pub title_en: String,
    pub notes_zh: String,
    pub notes_en: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct ReleaseRow {
    pub id: Uuid,
    pub version: String,
    pub channel: String,
    pub status: String,
    pub title_zh: String,
    pub title_en: String,
    pub notes_zh: String,
    pub notes_en: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ReleaseRow> for Release {
    type Error = AppError;

    fn try_from(row: ReleaseRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            version: row.version,
            channel: ReleaseChannel::try_from(row.channel.as_str())?,
            status: ReleaseStatus::try_from(row.status.as_str())?,
            title_zh: row.title_zh,
            title_en: row.title_en,
            notes_zh: row.notes_zh,
            notes_en: row.notes_en,
            published_at: row.published_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateReleaseInput {
    pub version: String,
    pub channel: ReleaseChannel,
    pub title_zh: String,
    pub title_en: String,
    pub notes_zh: String,
    pub notes_en: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateReleaseInput {
    pub title_zh: Option<String>,
    pub title_en: Option<String>,
    pub notes_zh: Option<String>,
    pub notes_en: Option<String>,
    pub status: Option<ReleaseStatus>,
}
