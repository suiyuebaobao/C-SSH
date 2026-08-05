//! 定义匿名更新检查的查询与稳定公开响应。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SourceKind;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCheckQuery {
    pub platform: String,
    pub architecture: String,
    pub current_version: String,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateCheckResponse {
    pub update_available: bool,
    pub current_version: String,
    pub latest: Option<LatestUpdate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatestUpdate {
    pub version: String,
    pub channel: String,
    pub title: String,
    pub notes: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<UpdateAsset>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateAsset {
    pub id: Uuid,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub sources: Vec<UpdateSource>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateSource {
    pub source_kind: SourceKind,
    pub provider_name: String,
    pub download_url: String,
}

fn default_channel() -> String {
    "stable".to_owned()
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}
