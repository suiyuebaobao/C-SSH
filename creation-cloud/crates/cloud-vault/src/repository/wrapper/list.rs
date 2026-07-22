//! 分页读取当前账号所有未软删除的包装密钥。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultKeyWrapper, repository::storage};

use super::{WrapperRow, wrapper_from_row};

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<VaultKeyWrapper>> {
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM vault_key_wrappers \
         WHERE account_id = $1 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(storage("无法统计包装密钥"))?;
    let rows = sqlx::query_as::<_, WrapperRow>(
        "SELECT id, wrapper_key, revision, schema_version, key_version, cipher_suite, \
         kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism, \
         nonce, wrapped_master_key_ciphertext, created_at, updated_at \
         FROM vault_key_wrappers WHERE account_id = $1 AND deleted_at IS NULL \
         ORDER BY updated_at DESC, id DESC LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(storage("无法读取包装密钥列表"))?;
    let items = rows
        .into_iter()
        .map(wrapper_from_row)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}
