//! 定义兼容现有客户端的匿名更新响应，并追加强制策略与签名元数据。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SourceKind;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCheckQuery {
    pub policy_version: Option<u8>,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub current_version: String,
    pub current_install_sha256: Option<String>,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateCheckResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    pub update_available: bool,
    pub current_version: String,
    pub latest: Option<LatestUpdate>,
    pub required: bool,
    pub policy_revision: Option<u64>,
    pub sha256_enabled: bool,
    pub identity_status: UpdateIdentityStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateIdentityStatus {
    NotApplicable,
    Verified,
    Missing,
    Mismatch,
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
    pub updater_signature: Option<String>,
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
