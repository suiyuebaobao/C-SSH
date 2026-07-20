//! 按账号所有权读取单个未删除密文信封。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::VaultEnvelope;

use super::{EnvelopeRow, envelope_from_row, storage};

pub(crate) async fn get(
    pool: &PgPool,
    account_id: Uuid,
    envelope_id: Uuid,
) -> AppResult<VaultEnvelope> {
    let row = sqlx::query_as::<_, EnvelopeRow>(
        "SELECT id, envelope_key, revision, schema_version, key_version, cipher_suite, \
         kdf_metadata, nonce, ciphertext, created_at, updated_at \
         FROM vault_envelopes \
         WHERE account_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .bind(envelope_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法读取密文信封"))?
    .ok_or_else(|| AppError::NotFound("密文信封不存在".to_owned()))?;
    envelope_from_row(row)
}
