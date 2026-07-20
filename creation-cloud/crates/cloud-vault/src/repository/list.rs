//! 分页读取当前账号未删除的密文信封列表。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::VaultEnvelope;

use super::{EnvelopeRow, envelope_from_row, storage};

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<VaultEnvelope>> {
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM vault_envelopes \
         WHERE account_id = $1 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(storage("无法统计密文信封"))?;
    let rows = sqlx::query_as::<_, EnvelopeRow>(
        "SELECT id, envelope_key, revision, schema_version, key_version, cipher_suite, \
         kdf_metadata, nonce, ciphertext, created_at, updated_at \
         FROM vault_envelopes WHERE account_id = $1 AND deleted_at IS NULL \
         ORDER BY updated_at DESC, id DESC LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(storage("无法读取密文信封列表"))?;
    let items = rows
        .into_iter()
        .map(envelope_from_row)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}
