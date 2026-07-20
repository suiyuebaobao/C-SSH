//! 定义密文信封 API、持久化命令和不含明文的响应对象。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KdfMetadata {
    pub algorithm: String,
    pub salt: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateVaultEnvelopeInput {
    pub envelope_key: Uuid,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub ciphertext: String,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateVaultEnvelopeInput {
    pub expected_revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub ciphertext: String,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteVaultInput {
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct VaultEnvelope {
    pub id: Uuid,
    pub envelope_key: Uuid,
    pub revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub ciphertext: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteVaultOutcome {
    pub id: Uuid,
    pub revision: i64,
    pub deleted_at: DateTime<Utc>,
}

pub(crate) struct CreateEnvelope {
    pub id: Uuid,
    pub envelope_key: Uuid,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub(crate) struct UpdateEnvelope {
    pub expected_revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}
