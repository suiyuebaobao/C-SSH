//! 定义来源校验和实际分发所需的内部只读投影。

use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(crate) struct AssetRecord {
    pub release_id: Uuid,
    pub byte_size: i64,
    pub sha256: String,
    pub release_status: String,
}

#[derive(Debug, FromRow)]
pub(crate) struct LockedAssetRecord {
    pub release_id: Uuid,
    pub byte_size: i64,
    pub sha256: String,
}

#[derive(Debug, FromRow)]
pub(crate) struct DownloadTarget {
    pub asset_id: Uuid,
    pub source_id: Uuid,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub source_kind: String,
    pub local_path: Option<String>,
    pub external_url: Option<String>,
}
