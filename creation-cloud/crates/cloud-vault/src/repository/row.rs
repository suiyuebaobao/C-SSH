//! 把运行时 query_as 返回的密文字节行转换为 base64 API 对象。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use uuid::Uuid;

use crate::{KdfMetadata, VaultEnvelope};

pub(crate) type EnvelopeRow = (
    Uuid,
    Uuid,
    i64,
    i32,
    i32,
    String,
    Value,
    Vec<u8>,
    Vec<u8>,
    DateTime<Utc>,
    DateTime<Utc>,
);

pub(crate) fn envelope_from_row(row: EnvelopeRow) -> AppResult<VaultEnvelope> {
    let kdf = serde_json::from_value::<KdfMetadata>(row.6)
        .map_err(|_| AppError::Internal("密文信封 KDF 元数据格式无效".to_owned()))?;
    Ok(VaultEnvelope {
        id: row.0,
        envelope_key: row.1,
        revision: row.2,
        schema_version: row.3,
        key_version: row.4,
        cipher_suite: row.5,
        kdf,
        nonce: STANDARD.encode(row.7),
        ciphertext: STANDARD.encode(row.8),
        created_at: row.9,
        updated_at: row.10,
    })
}
