//! 定义本地与外部下载来源及管理输入。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Local,
    External,
}

impl SourceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::External => "external",
        }
    }
}

impl TryFrom<&str> for SourceKind {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "local" => Ok(Self::Local),
            "external" => Ok(Self::External),
            _ => Err(AppError::Internal("数据库中的下载来源类型无效".into())),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReleaseSource {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub source_kind: SourceKind,
    pub provider_name: String,
    pub local_path: Option<String>,
    pub external_url: Option<String>,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct SourceRow {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub source_kind: String,
    pub provider_name: String,
    pub local_path: Option<String>,
    pub external_url: Option<String>,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<SourceRow> for ReleaseSource {
    type Error = AppError;

    fn try_from(row: SourceRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            asset_id: row.asset_id,
            source_kind: SourceKind::try_from(row.source_kind.as_str())?,
            provider_name: row.provider_name,
            local_path: row.local_path,
            external_url: row.external_url,
            sort_order: row.sort_order,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateSourceInput {
    #[serde(skip)]
    pub asset_id: Uuid,
    pub source_kind: SourceKind,
    pub provider_name: String,
    pub local_path: Option<String>,
    pub external_url: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateSourceInput {
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
}

const fn enabled_by_default() -> bool {
    true
}
