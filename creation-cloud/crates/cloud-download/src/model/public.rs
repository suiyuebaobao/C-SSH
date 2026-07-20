//! 定义不暴露本站路径或外链凭据的公开下载清单。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::SourceKind;

#[derive(Clone, Debug, Serialize)]
pub struct PublicRelease {
    pub id: Uuid,
    pub version: String,
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
    pub source_id: Uuid,
    pub source_kind: String,
    pub provider_name: String,
    pub sort_order: i32,
}

impl PublicCatalogRow {
    pub(crate) fn source(&self) -> AppResult<PublicSource> {
        Ok(PublicSource {
            id: self.source_id,
            source_kind: SourceKind::try_from(self.source_kind.as_str())?,
            provider_name: self.provider_name.clone(),
            sort_order: self.sort_order,
            download_path: format!("assets/{}/sources/{}", self.asset_id, self.source_id),
        })
    }
}
