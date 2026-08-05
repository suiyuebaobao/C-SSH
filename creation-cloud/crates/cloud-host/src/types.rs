//! 定义主机元数据、原样密文、账号同步代次和手动同步的稳定 API 形状。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostStatus {
    Active,
    Disabled,
    Archived,
}

impl HostStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Archived => "archived",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostMetadataInput {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub platform: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: HostStatus,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostView {
    pub id: Uuid,
    pub address: String,
    pub port: u16,
    pub name: String,
    pub platform: String,
    pub tags: Vec<String>,
    pub status: HostStatus,
    pub revision: i64,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub secret_present: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOperation {
    Insert,
    Update,
    Delete,
}

impl HostOperation {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "insert" => Some(Self::Insert),
            "update" => Some(Self::Update),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostChange {
    pub host_id: Uuid,
    pub operation: HostOperation,
    #[serde(default)]
    pub metadata: Option<HostMetadataInput>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_ciphertext",
        skip_serializing_if = "Option::is_none"
    )]
    pub ciphertext: Option<Option<String>>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushRequest {
    pub sync_generation: i64,
    pub base_revision: i64,
    pub client_mutation_id: Uuid,
    pub changes: Vec<HostChange>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PushOutcome {
    Applied {
        sync_generation: i64,
        revision: i64,
        changed_count: u32,
        idempotent: bool,
    },
    Unchanged {
        sync_generation: i64,
        revision: i64,
        idempotent: bool,
    },
    Conflict {
        sync_generation: i64,
        conflict: HostConflictView,
        idempotent: bool,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct HostConflictView {
    pub id: Uuid,
    pub host_id: Uuid,
    pub client_mutation_id: Uuid,
    pub base_revision: i64,
    pub remote_revision: i64,
    pub proposed_operation: HostOperation,
    pub source_device_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteResolution {
    ReplaceRemote,
    KeepRemote,
}

impl RemoteResolution {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ReplaceRemote => "replace_remote",
            Self::KeepRemote => "keep_remote",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveConflictRequest {
    pub sync_generation: i64,
    pub action: RemoteResolution,
    pub resolution_mutation_id: Uuid,
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolveConflictOutcome {
    pub conflict_id: Uuid,
    pub resolution_mutation_id: Uuid,
    pub action: RemoteResolution,
    pub revision: i64,
    pub idempotent: bool,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PullRequest {
    pub sync_generation: i64,
    #[serde(default)]
    pub since_revision: i64,
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    #[serde(default)]
    pub after_revision: Option<i64>,
    #[serde(default)]
    pub after_host_id: Option<Uuid>,
    #[serde(default = "default_pull_limit")]
    pub limit: u32,
}

impl Default for PullRequest {
    fn default() -> Self {
        Self {
            sync_generation: 1,
            since_revision: 0,
            snapshot_revision: None,
            after_revision: None,
            after_host_id: None,
            limit: default_pull_limit(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PullHostRecord {
    pub host_id: Uuid,
    pub revision: i64,
    pub address: String,
    pub port: u16,
    pub name: String,
    pub platform: String,
    pub tags: Vec<String>,
    pub status: HostStatus,
    pub ciphertext: Option<String>,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PullResponse {
    pub sync_generation: i64,
    pub records: Vec<PullHostRecord>,
    pub snapshot_revision: i64,
    pub next_revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_after_host_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalDecision {
    ReplaceLocal,
    KeepLocal,
}

impl LocalDecision {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ReplaceLocal => "replace_local",
            Self::KeepLocal => "keep_local",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullDecision {
    pub host_id: Uuid,
    pub cloud_revision: i64,
    pub action: LocalDecision,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullAckRequest {
    pub sync_generation: i64,
    pub acknowledged_revision: i64,
    pub decisions: Vec<PullDecision>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SyncStateView {
    pub sync_generation: i64,
    pub current_revision: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetConfirmation {
    ResetEncryptedSyncData,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResetSyncRequest {
    pub mutation_id: Uuid,
    pub confirmation: ResetConfirmation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResetSyncResponse {
    pub status: String,
    pub sync_generation: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RekeyHostCandidate {
    pub host_id: Uuid,
    pub cloud_revision: i64,
    pub ciphertext: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RekeySyncRequest {
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub hosts: Vec<RekeyHostCandidate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RekeyHostRevision {
    pub host_id: Uuid,
    pub cloud_revision: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RekeySyncResponse {
    pub status: String,
    pub sync_generation: i64,
    pub current_revision: i64,
    pub revisions: Vec<RekeyHostRevision>,
    pub idempotent: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSyncDirection {
    Upload,
    Download,
}

impl AdminSyncDirection {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "upload" => Some(Self::Upload),
            "download" => Some(Self::Download),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminSyncRecord {
    pub record_id: String,
    pub direction: AdminSyncDirection,
    pub device_id: Uuid,
    pub device_name: String,
    pub device_platform: String,
    pub outcome: String,
    pub revision: i64,
    pub changed_count: i32,
    pub occurred_at: DateTime<Utc>,
}

const fn default_pull_limit() -> u32 {
    100
}

fn deserialize_optional_ciphertext<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}
