//! 定义与下载来源解耦的安装资产身份和管理输入。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct ReleaseAsset {
    pub id: Uuid,
    pub release_id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
}

pub(crate) type AssetRow = ReleaseAsset;

#[derive(Clone, Debug, Deserialize)]
pub struct CreateAssetInput {
    #[serde(skip)]
    pub release_id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateAssetInput {
    pub platform: Option<String>,
    pub architecture: Option<String>,
    pub package_kind: Option<String>,
    pub file_name: Option<String>,
    pub byte_size: Option<i64>,
    pub sha256: Option<String>,
}
