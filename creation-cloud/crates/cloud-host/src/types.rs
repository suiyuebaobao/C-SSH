//! 定义主机与 AI provider 账号密文共享的同步 API 形状。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

mod host_metadata_migration;
mod protection;
mod proxy_profile;
mod push_receipt;
mod serialization;

pub use host_metadata_migration::*;
pub use protection::*;
pub use proxy_profile::*;
pub use push_receipt::*;

pub const CURRENT_SYNC_CONTRACT_VERSION: u16 = 4;
pub const LEGACY_SYNC_CONTRACT_VERSION: u16 = 2;

const fn default_sync_contract_version() -> u16 {
    LEGACY_SYNC_CONTRACT_VERSION
}

pub(super) const fn is_legacy_sync_contract(version: &u16) -> bool {
    *version == LEGACY_SYNC_CONTRACT_VERSION
}

fn is_legacy_sync_contract_i32(version: &i32) -> bool {
    *version == i32::from(LEGACY_SYNC_CONTRACT_VERSION)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostStatus {
    Active,
    Disabled,
    Archived,
}

impl HostStatus {
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
    pub revision: i64,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub secret_present: bool,
    pub metadata_encrypted: bool,
    pub host_metadata_migration_required: bool,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Host,
    AiProviderAccount,
    ProxyProfile,
}

impl ResourceKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::AiProviderAccount => "ai_provider_account",
            Self::ProxyProfile => "proxy_profile",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "host" => Some(Self::Host),
            "ai_provider_account" => Some(Self::AiProviderAccount),
            "proxy_profile" => Some(Self::ProxyProfile),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderOperation {
    Insert,
    Update,
    Delete,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostChange {
    pub host_id: Uuid,
    pub operation: HostOperation,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata: Option<Option<HostMetadataInput>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_ciphertext",
        skip_serializing_if = "Option::is_none"
    )]
    pub ciphertext: Option<Option<String>>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
}

fn deserialize_optional_metadata<'de, D>(
    deserializer: D,
) -> Result<Option<Option<HostMetadataInput>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<HostMetadataInput>::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiProviderPayloadInput {
    pub ciphertext: String,
    pub nonce: String,
    pub envelope_metadata: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiProviderChange {
    pub resource_id: Uuid,
    pub operation: AiProviderOperation,
    #[serde(default)]
    pub payload: Option<AiProviderPayloadInput>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushRequest {
    #[serde(
        default = "default_sync_contract_version",
        skip_serializing_if = "is_legacy_sync_contract"
    )]
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub base_revision: i64,
    pub client_mutation_id: Uuid,
    pub host_changes: Vec<HostChange>,
    pub ai_changes: Vec<AiProviderChange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proxy_profile_changes: Vec<ProxyProfileChange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceRevision {
    pub resource_kind: ResourceKind,
    pub resource_id: Uuid,
    pub cloud_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PushOutcome {
    Applied {
        sync_generation: i64,
        protection_epoch: i64,
        protection_revision: i64,
        revision: i64,
        changed_count: u32,
        revisions: Vec<ResourceRevision>,
        idempotent: bool,
    },
    Unchanged {
        sync_generation: i64,
        protection_epoch: i64,
        protection_revision: i64,
        revision: i64,
        revisions: Vec<ResourceRevision>,
        idempotent: bool,
    },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PullMode {
    #[default]
    Incremental,
    Full,
}

/// A required pull intent. Preview is strictly read-only; download and
/// verification pulls create the delivery evidence required by a later ACK.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PullPurpose {
    Preview,
    Download,
    Verification,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PullRequest {
    #[serde(default = "default_sync_contract_version")]
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub purpose: PullPurpose,
    #[serde(default)]
    pub since_revision: i64,
    #[serde(default)]
    pub mode: PullMode,
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    #[serde(default)]
    pub after_revision: Option<i64>,
    #[serde(default = "default_pull_limit")]
    pub limit: u32,
}

impl Default for PullRequest {
    fn default() -> Self {
        Self {
            sync_contract_version: LEGACY_SYNC_CONTRACT_VERSION,
            sync_generation: 1,
            protection_epoch: 1,
            protection_revision: 1,
            purpose: PullPurpose::Download,
            since_revision: 0,
            mode: PullMode::Incremental,
            snapshot_revision: None,
            after_revision: None,
            limit: default_pull_limit(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PullHostRecord {
    pub host_id: Uuid,
    pub revision: i64,
    pub ciphertext: Option<String>,
    pub metadata_encrypted: bool,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PullAiProviderRecord {
    pub resource_id: Uuid,
    pub revision: i64,
    pub ciphertext: Option<String>,
    pub nonce: Option<String>,
    pub envelope_metadata: Option<serde_json::Value>,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PullResponse {
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub purpose: PullPurpose,
    pub mode: PullMode,
    pub host_records: Vec<PullHostRecord>,
    pub ai_records: Vec<PullAiProviderRecord>,
    pub proxy_profile_records: Vec<PullProxyProfileRecord>,
    pub snapshot_revision: i64,
    pub next_revision: i64,
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
    pub resource_kind: ResourceKind,
    pub resource_id: Uuid,
    pub cloud_revision: i64,
    pub action: LocalDecision,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullAckRequest {
    #[serde(default = "default_sync_contract_version")]
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub acknowledged_revision: i64,
    pub decisions: Vec<PullDecision>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncGenerationTransition {
    Initial,
    ProtectionSetup,
    LegacyMigration,
    HostMetadataMigration,
    Rekey,
    Reset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SyncStateView {
    #[serde(skip_serializing_if = "is_legacy_sync_contract_i32")]
    pub minimum_sync_contract_version: i32,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub current_revision: i64,
    pub compacted_through_revision: i64,
    pub generation_transition: SyncGenerationTransition,
    pub data_protection_configured: bool,
    pub legacy_migration_required: bool,
    pub host_metadata_migration_required: bool,
    pub secret_present: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetConfirmation {
    ResetEncryptedSyncData,
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResetSyncRequest {
    #[serde(
        default = "default_sync_contract_version",
        skip_serializing_if = "is_legacy_sync_contract"
    )]
    pub sync_contract_version: u16,
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub expected_epoch: i64,
    pub expected_revision: i64,
    pub current_revision: i64,
    pub confirmation: ResetConfirmation,
    pub authorization: ResetAuthorization,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResetSyncResponse {
    pub status: String,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub current_revision: i64,
    pub data_protection_configured: bool,
    pub idempotent: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "resource_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RekeyResourceCandidate {
    Host {
        resource_id: Uuid,
        cloud_revision: i64,
        ciphertext: String,
    },
    AiProviderAccount {
        resource_id: Uuid,
        cloud_revision: i64,
        ciphertext: String,
        nonce: String,
        envelope_metadata: serde_json::Value,
    },
    ProxyProfile {
        resource_id: Uuid,
        cloud_revision: i64,
        ciphertext: String,
        nonce: String,
        envelope_metadata: serde_json::Value,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RekeySyncRequest {
    #[serde(
        default = "default_sync_contract_version",
        skip_serializing_if = "is_legacy_sync_contract"
    )]
    pub sync_contract_version: u16,
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub resources: Vec<RekeyResourceCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RekeySyncResponse {
    pub status: String,
    pub sync_generation: i64,
    pub current_revision: i64,
    pub revisions: Vec<ResourceRevision>,
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
