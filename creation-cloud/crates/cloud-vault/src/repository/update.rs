//! 以 revision 乐观锁完整替换密文信封，不检查或解密 ciphertext。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultEnvelope, types::UpdateEnvelope};

use super::{EnvelopeRow, envelope_from_row, storage};

pub(crate) async fn update(
    pool: &PgPool,
    account_id: Uuid,
    envelope_id: Uuid,
    envelope: UpdateEnvelope,
) -> AppResult<VaultEnvelope> {
    let kdf = serde_json::to_value(&envelope.kdf)
        .map_err(|_| AppError::Internal("无法编码 KDF 元数据".to_owned()))?;
    let row = sqlx::query_as::<_, EnvelopeRow>(
        "UPDATE vault_envelopes SET revision = revision + 1, schema_version = $4, \
         key_version = $5, cipher_suite = $6, kdf_metadata = $7, nonce = $8, \
         ciphertext = $9, updated_at = now() \
         WHERE account_id = $1 AND id = $2 AND revision = $3 AND deleted_at IS NULL \
         RETURNING id, envelope_key, revision, schema_version, key_version, cipher_suite, \
          kdf_metadata, nonce, ciphertext, created_at, updated_at",
    )
    .bind(account_id)
    .bind(envelope_id)
    .bind(envelope.expected_revision)
    .bind(envelope.schema_version)
    .bind(envelope.key_version)
    .bind(envelope.cipher_suite)
    .bind(kdf)
    .bind(envelope.nonce)
    .bind(envelope.ciphertext)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法更新密文信封"))?;
    if let Some(row) = row {
        return envelope_from_row(row);
    }
    revision_failure(pool, account_id, envelope_id, envelope.expected_revision).await
}

async fn revision_failure(
    pool: &PgPool,
    account_id: Uuid,
    envelope_id: Uuid,
    expected_revision: i64,
) -> AppResult<VaultEnvelope> {
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT revision FROM vault_envelopes \
         WHERE account_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .bind(envelope_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法确认密文信封 revision"))?;
    match current {
        Some(current) => Err(AppError::Conflict(format!(
            "密文信封 revision 冲突：期望 {expected_revision}，当前 {current}"
        ))),
        None => Err(AppError::NotFound("密文信封不存在".to_owned())),
    }
}
