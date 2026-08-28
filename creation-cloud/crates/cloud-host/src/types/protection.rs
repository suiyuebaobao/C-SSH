//! Strict account-level data-protection envelope and transition DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    PullAiProviderRecord, PullHostRecord, RekeyResourceCandidate, ResourceKind, ResourceRevision,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataProtectionEnvelopeInput {
    pub format_version: u16,
    pub kdf_algorithm: String,
    pub kdf_version: u32,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub kdf_output_length: u32,
    pub salt: String,
    pub nonce: String,
    pub wrapped_data_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataProtectionEnvelopeView {
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub format_version: u16,
    pub kdf_algorithm: String,
    pub kdf_version: u32,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub kdf_output_length: u32,
    pub salt: String,
    pub nonce: String,
    pub wrapped_data_key: String,
    pub source_device_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DataProtectionView {
    #[serde(flatten)]
    pub state: super::SyncStateView,
    pub envelope: Option<DataProtectionEnvelopeView>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SetupDataProtectionRequest {
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub expected_epoch: i64,
    pub expected_revision: i64,
    pub current_revision: i64,
    pub envelope: DataProtectionEnvelopeInput,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MigrateDataProtectionRequest {
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub expected_epoch: i64,
    pub expected_revision: i64,
    pub current_revision: i64,
    pub envelope: DataProtectionEnvelopeInput,
    pub resources: Vec<RekeyResourceCandidate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeDataProtectionRequest {
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub expected_epoch: i64,
    pub expected_revision: i64,
    pub current_revision: i64,
    pub envelope: DataProtectionEnvelopeInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataProtectionMutationResponse {
    pub status: String,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub current_revision: i64,
    pub data_protection_configured: bool,
    pub revisions: Vec<ResourceRevision>,
    pub idempotent: bool,
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResetAuthorization {
    KnownPasswordClientConfirmation,
    EmailVerification {
        challenge_id: Uuid,
        authorization_token: String,
    },
}

impl Drop for ResetAuthorization {
    fn drop(&mut self) {
        if let Self::EmailVerification {
            authorization_token,
            ..
        } = self
        {
            zeroize_string(authorization_token);
        }
    }
}

impl ResetAuthorization {
    pub(crate) const fn audit_mode(&self) -> &'static str {
        match self {
            Self::KnownPasswordClientConfirmation => "client_local_check",
            Self::EmailVerification { .. } => "email_recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPullRequest {
    pub sync_generation: i64,
    pub expected_epoch: i64,
    pub expected_revision: i64,
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    #[serde(default)]
    pub after_revision: Option<i64>,
    #[serde(default)]
    pub after_resource_kind: Option<ResourceKind>,
    #[serde(default)]
    pub after_resource_id: Option<Uuid>,
    #[serde(default = "default_legacy_pull_limit")]
    pub limit: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct LegacyPullCursor {
    pub revision: i64,
    pub resource_kind: ResourceKind,
    pub resource_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct LegacyPullResponse {
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub snapshot_revision: i64,
    pub host_records: Vec<PullHostRecord>,
    pub ai_records: Vec<PullAiProviderRecord>,
    pub next_cursor: Option<LegacyPullCursor>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionResetChallengeRequest {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtectionResetChallengeResponse {
    pub status: String,
    pub challenge_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyProtectionResetChallengeRequest {
    pub challenge_id: Uuid,
    pub code: String,
}

impl Drop for VerifyProtectionResetChallengeRequest {
    fn drop(&mut self) {
        zeroize_string(&mut self.code);
    }
}

#[derive(Eq, PartialEq, Serialize)]
pub struct VerifyProtectionResetChallengeResponse {
    pub status: String,
    pub challenge_id: Uuid,
    pub authorization_token: String,
    pub expires_at: DateTime<Utc>,
}

impl Drop for VerifyProtectionResetChallengeResponse {
    fn drop(&mut self) {
        zeroize_string(&mut self.authorization_token);
    }
}

const fn default_legacy_pull_limit() -> u32 {
    100
}

fn zeroize_string(value: &mut String) {
    let mut bytes = std::mem::take(value).into_bytes();
    bytes.fill(0);
}
