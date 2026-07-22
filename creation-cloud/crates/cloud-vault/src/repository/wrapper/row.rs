//! 把包装密钥数据库行转换为不含明文密钥材料的 API 对象。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{KdfMetadata, VaultKeyWrapper};

pub(crate) type WrapperRow = (
    Uuid,
    Uuid,
    i64,
    i32,
    i32,
    String,
    String,
    Vec<u8>,
    i64,
    i64,
    i64,
    Vec<u8>,
    Vec<u8>,
    DateTime<Utc>,
    DateTime<Utc>,
);

pub(crate) fn wrapper_from_row(row: WrapperRow) -> AppResult<VaultKeyWrapper> {
    Ok(VaultKeyWrapper {
        id: row.0,
        wrapper_key: row.1,
        revision: row.2,
        schema_version: row.3,
        key_version: row.4,
        cipher_suite: row.5,
        kdf: KdfMetadata {
            algorithm: row.6,
            salt: STANDARD.encode(row.7),
            memory_kib: to_u32("kdf_memory_kib", row.8)?,
            iterations: to_u32("kdf_iterations", row.9)?,
            parallelism: to_u32("kdf_parallelism", row.10)?,
        },
        nonce: STANDARD.encode(row.11),
        wrapped_master_key: STANDARD.encode(row.12),
        created_at: row.13,
        updated_at: row.14,
    })
}

fn to_u32(name: &str, value: i64) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| AppError::Internal(format!("包装密钥 {name} 超出响应范围")))
}
