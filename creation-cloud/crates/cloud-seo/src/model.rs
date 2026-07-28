use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SeoLocale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl SeoLocale {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }
}

impl TryFrom<&str> for SeoLocale {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "zh-CN" => Ok(Self::ZhCn),
            "en" => Ok(Self::En),
            _ => Err(AppError::Validation(
                "SEO 主题词语言仅支持 zh-CN 或 en".to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SeoTopic {
    pub id: Uuid,
    pub locale: SeoLocale,
    pub phrase: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct SeoTopicRow {
    pub id: Uuid,
    pub locale: String,
    pub phrase: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<SeoTopicRow> for SeoTopic {
    type Error = AppError;

    fn try_from(row: SeoTopicRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            locale: SeoLocale::try_from(row.locale.as_str())
                .map_err(|_| AppError::Internal("数据库中的 SEO 主题词语言无效".to_owned()))?,
            phrase: row.phrase,
            sort_order: row.sort_order,
            enabled: row.enabled,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSeoTopicInput {
    pub locale: SeoLocale,
    pub phrase: String,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSeoTopicInput {
    pub locale: Option<SeoLocale>,
    pub phrase: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
}

const fn enabled_by_default() -> bool {
    true
}
