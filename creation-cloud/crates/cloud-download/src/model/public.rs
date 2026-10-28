//! 定义不暴露本站路径或外链凭据的公开下载清单。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::SourceKind;

#[derive(Clone, Debug, Serialize)]
pub struct PublicRelease {
    pub id: Uuid,
    pub version: String,
    pub revision: i32,
    pub channel: String,
    pub title_zh: String,
    pub title_en: String,
    pub notes_zh: String,
    pub notes_en: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<PublicAsset>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicAsset {
    pub id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub updater_signature: Option<String>,
    pub sources: Vec<PublicSource>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicSource {
    pub id: Uuid,
    pub source_kind: SourceKind,
    pub provider_name: String,
    pub sort_order: i32,
    pub download_path: String,
}

#[derive(Debug, FromRow)]
pub(crate) struct PublicCatalogRow {
    pub release_id: Uuid,
    pub version: String,
    pub revision: i32,
    pub channel: String,
    pub title_zh: String,
    pub title_en: String,
    pub notes_zh: String,
    pub notes_en: String,
    pub published_at: DateTime<Utc>,
    pub asset_id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub updater_signature: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_kind: Option<String>,
    pub provider_name: Option<String>,
    pub sort_order: Option<i32>,
}

impl PublicCatalogRow {
    pub(crate) fn source(&self) -> AppResult<Option<PublicSource>> {
        match (
            self.source_id,
            self.source_kind.as_deref(),
            self.provider_name.as_deref(),
            self.sort_order,
        ) {
            (None, None, None, None) => Ok(None),
            (Some(id), Some(kind), Some(provider_name), Some(sort_order)) => {
                Ok(Some(PublicSource {
                    id,
                    source_kind: SourceKind::try_from(kind)?,
                    provider_name: provider_name.into(),
                    sort_order,
                    download_path: format!("assets/{}/sources/{id}", self.asset_id),
                }))
            }
            _ => Err(AppError::Storage("公开下载来源字段不完整".into())),
        }
    }
}
