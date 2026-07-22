//! 定义账号级 master-key 包装密文的 API、持久化命令与响应对象。
//! 所有二进制字段在 HTTP 边界使用 base64，服务端不解密包装内容。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::KdfMetadata;

#[derive(Clone, Debug, Deserialize)]
pub struct CreateVaultKeyWrapperInput {
    pub wrapper_key: Uuid,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub wrapped_master_key: String,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateVaultKeyWrapperInput {
    pub expected_revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub wrapped_master_key: String,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteVaultKeyWrapperInput {
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct VaultKeyWrapper {
    pub id: Uuid,
    pub wrapper_key: Uuid,
    pub revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf: KdfMetadata,
    pub nonce: String,
    pub wrapped_master_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteVaultKeyWrapperOutcome {
    pub id: Uuid,
    pub revision: i64,
    pub deleted_at: DateTime<Utc>,
}

pub(crate) struct CreateKeyWrapper {
    pub id: Uuid,
    pub wrapper_key: Uuid,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf_algorithm: String,
    pub kdf_salt: Vec<u8>,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub nonce: Vec<u8>,
    pub wrapped_master_key: Vec<u8>,
}

pub(crate) struct UpdateKeyWrapper {
    pub expected_revision: i64,
    pub schema_version: i32,
    pub key_version: i32,
    pub cipher_suite: String,
    pub kdf_algorithm: String,
    pub kdf_salt: Vec<u8>,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub nonce: Vec<u8>,
    pub wrapped_master_key: Vec<u8>,
}
