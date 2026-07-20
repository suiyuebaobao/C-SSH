//! 创建仅含客户端密文字节与算法元数据的保险库信封。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultEnvelope, types::CreateEnvelope};

use super::{EnvelopeRow, envelope_from_row, storage};

pub(crate) async fn create(
    pool: &PgPool,
    account_id: Uuid,
    envelope: CreateEnvelope,
) -> AppResult<VaultEnvelope> {
    let kdf = serde_json::to_value(&envelope.kdf)
        .map_err(|_| AppError::Internal("无法编码 KDF 元数据".to_owned()))?;
    let row = sqlx::query_as::<_, EnvelopeRow>(
        "INSERT INTO vault_envelopes \
         (id, account_id, envelope_key, revision, schema_version, key_version, \
          cipher_suite, kdf_metadata, nonce, ciphertext) \
         VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT (account_id, envelope_key) DO NOTHING \
         RETURNING id, envelope_key, revision, schema_version, key_version, cipher_suite, \
          kdf_metadata, nonce, ciphertext, created_at, updated_at",
    )
    .bind(envelope.id)
    .bind(account_id)
    .bind(envelope.envelope_key)
    .bind(envelope.schema_version)
    .bind(envelope.key_version)
    .bind(envelope.cipher_suite)
    .bind(kdf)
    .bind(envelope.nonce)
    .bind(envelope.ciphertext)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法创建密文信封"))?
    .ok_or_else(|| AppError::Conflict("envelope_key 已存在".to_owned()))?;
    envelope_from_row(row)
}
