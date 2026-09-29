//! Host 明文 metadata 的显式客户端迁移 wire；密码和 CDK 永不进入这些 DTO。

use chrono::{DateTime, Utc};
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
use uuid::Uuid;

use super::{DataProtectionEnvelopeInput, HostMetadataInput};

fn default_preview_limit() -> u32 {
    100
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct RequiredNullableCiphertext(pub Option<String>);

impl<'de> Deserialize<'de> for RequiredNullableCiphertext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NullableVisitor;

        impl<'de> Visitor<'de> for NullableVisitor {
            type Value = RequiredNullableCiphertext;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a ciphertext string or explicit null")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(RequiredNullableCiphertext(None))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(RequiredNullableCiphertext(None))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RequiredNullableCiphertext(Some(value.to_owned())))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(RequiredNullableCiphertext(Some(value)))
            }
        }

        deserializer.deserialize_any(NullableVisitor)
    }
}

impl RequiredNullableCiphertext {
    pub(crate) fn as_ref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostMetadataMigrationPreviewRequest {
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    #[serde(default)]
    pub after_host_id: Option<Uuid>,
    #[serde(default = "default_preview_limit")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostMetadataMigrationPreviewRecord {
    pub host_id: Uuid,
    pub cloud_revision: i64,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub metadata_encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_metadata: Option<HostMetadataInput>,
    pub ciphertext: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostMetadataMigrationPreviewResponse {
    pub sync_contract_version: u16,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub snapshot_revision: i64,
    pub records: Vec<HostMetadataMigrationPreviewRecord>,
    pub next_after_host_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostMetadataMigrationCandidate {
    pub host_id: Uuid,
    pub cloud_revision: i64,
    pub ciphertext: RequiredNullableCiphertext,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostMetadataMigrationRequest {
    pub sync_contract_version: u16,
    pub mutation_id: Uuid,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
    pub snapshot_revision: i64,
    pub target_envelope: DataProtectionEnvelopeInput,
    pub hosts: Vec<HostMetadataMigrationCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HostMetadataMigrationReceipt {
    pub status: String,
    pub mutation_id: Uuid,
    pub source_device_id: Uuid,
    pub request_hash: String,
    pub source_sync_generation: i64,
    pub source_protection_epoch: i64,
    pub source_protection_revision: i64,
    pub source_snapshot_revision: i64,
    pub result_sync_generation: i64,
    pub result_protection_epoch: i64,
    pub result_protection_revision: i64,
    pub result_current_revision: i64,
    pub host_count: i32,
    pub completed_at: DateTime<Utc>,
    pub idempotent: bool,
}
