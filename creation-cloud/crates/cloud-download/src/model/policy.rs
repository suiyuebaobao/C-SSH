//! 定义版本策略草稿、追加式发布记录和管理员三步后台投影。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct UpdatePolicyDraft {
    pub revision: i64,
    pub enabled: bool,
    pub forced_versions: Vec<String>,
    pub target_release_id: Option<Uuid>,
    pub sha256_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublishedUpdatePolicy {
    pub revision: i64,
    pub enabled: bool,
    pub forced_versions: Vec<String>,
    pub target_release_id: Option<Uuid>,
    pub target_version: Option<String>,
    pub sha256_enabled: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<Uuid>,
}

impl PublishedUpdatePolicy {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            revision: 0,
            enabled: false,
            forced_versions: Vec::new(),
            target_release_id: None,
            target_version: None,
            sha256_enabled: true,
            published_at: None,
            published_by: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdatePolicyTargetRelease {
    pub id: Uuid,
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub eligible: bool,
    pub readiness: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminUpdatePolicySnapshot {
    pub draft: UpdatePolicyDraft,
    pub published: PublishedUpdatePolicy,
    pub target_releases: Vec<UpdatePolicyTargetRelease>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveUpdatePolicyDraftInput {
    pub expected_revision: i64,
    pub enabled: bool,
    pub forced_versions: Vec<String>,
    pub target_release_id: Option<Uuid>,
    pub sha256_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishUpdatePolicyInput {
    pub expected_draft_revision: i64,
    pub confirmation: String,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct UpdatePolicyDraftRow {
    pub revision: i64,
    pub enabled: bool,
    pub forced_versions: Vec<String>,
    pub target_release_id: Option<Uuid>,
    pub sha256_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct PublishedUpdatePolicyRow {
    pub revision: i64,
    pub enabled: bool,
    pub forced_versions: Vec<String>,
    pub target_release_id: Option<Uuid>,
    pub target_version: Option<String>,
    pub sha256_enabled: bool,
    pub published_at: DateTime<Utc>,
    pub published_by: Uuid,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct PolicyTargetRow {
    pub id: Uuid,
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub asset_count: i64,
    pub formal_asset_count: i64,
    pub required_signature_count: i64,
    pub local_source_count: i64,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct PolicyAssetRow {
    pub id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub updater_signature: Option<String>,
    pub source_id: Uuid,
    pub local_path: String,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct ForcedIdentityRow {
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub asset_sha256: String,
    pub installed_sha256: Option<String>,
}
