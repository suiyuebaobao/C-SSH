//! 定义站点媒体管理模型、状态值和公开首页二维码投影。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMediaSlot {
    HomeQr,
}

impl SiteMediaSlot {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HomeQr => "home_qr",
        }
    }
}

impl TryFrom<&str> for SiteMediaSlot {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "home_qr" => Ok(Self::HomeQr),
            _ => Err(AppError::Internal("数据库中的站点媒体槽位无效".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMediaState {
    Draft,
    Published,
    Revoked,
}

impl SiteMediaState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Revoked => "revoked",
        }
    }
}

impl TryFrom<&str> for SiteMediaState {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "revoked" => Ok(Self::Revoked),
            _ => Err(AppError::Internal("数据库中的站点媒体状态无效".into())),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteMedia {
    pub id: Uuid,
    pub slot: SiteMediaSlot,
    pub state: SiteMediaState,
    pub storage_key: String,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub width: i32,
    pub height: i32,
    pub alt_zh: String,
    pub alt_en: String,
    pub created_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct SiteMediaRow {
    pub id: Uuid,
    pub slot: String,
    pub state: String,
    pub storage_key: String,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub width: i32,
    pub height: i32,
    pub alt_zh: String,
    pub alt_en: String,
    pub created_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) struct NewSiteMedia {
    pub id: Uuid,
    pub storage_key: String,
    pub byte_size: i64,
    pub sha256: String,
    pub width: i32,
    pub height: i32,
    pub alt_zh: String,
    pub alt_en: String,
    pub created_by: Uuid,
}

pub(crate) struct PublicMediaContent {
    pub bytes: Vec<u8>,
    pub sha256: String,
}

impl TryFrom<SiteMediaRow> for SiteMedia {
    type Error = AppError;

    fn try_from(row: SiteMediaRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            slot: SiteMediaSlot::try_from(row.slot.as_str())?,
            state: SiteMediaState::try_from(row.state.as_str())?,
            storage_key: row.storage_key,
            content_type: row.content_type,
            byte_size: row.byte_size,
            sha256: row.sha256,
            width: row.width,
            height: row.height,
            alt_zh: row.alt_zh,
            alt_en: row.alt_en,
            created_by: row.created_by,
            published_at: row.published_at,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CreateSiteMediaInput {
    pub declared_content_type: String,
    pub bytes: Vec<u8>,
    pub alt_zh: String,
    pub alt_en: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSiteMediaInput {
    pub alt_zh: Option<String>,
    pub alt_en: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicHomeQr {
    pub id: Uuid,
    pub content_url: String,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub width: i32,
    pub height: i32,
    pub alt_zh: String,
    pub alt_en: String,
    pub published_at: DateTime<Utc>,
}

impl TryFrom<SiteMedia> for PublicHomeQr {
    type Error = AppError;

    fn try_from(media: SiteMedia) -> AppResult<Self> {
        if media.slot != SiteMediaSlot::HomeQr || media.state != SiteMediaState::Published {
            return Err(AppError::Internal("公开站点媒体投影状态无效".into()));
        }
        let published_at = media
            .published_at
            .ok_or_else(|| AppError::Internal("公开站点媒体缺少发布时间".into()))?;
        Ok(Self {
            id: media.id,
            content_url: format!("/api/v1/site-media/{}/content", media.id),
            content_type: media.content_type,
            byte_size: media.byte_size,
            sha256: media.sha256,
            width: media.width,
            height: media.height,
            alt_zh: media.alt_zh,
            alt_en: media.alt_en,
            published_at,
        })
    }
}
