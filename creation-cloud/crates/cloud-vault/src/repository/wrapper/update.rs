//! 以 revision 乐观锁完整替换包装密钥密文，稳定 wrapper_key 不允许修改。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultKeyWrapper, repository::storage, wrapper_types::UpdateKeyWrapper};

use super::{WrapperRow, failure, wrapper_from_row};

pub(crate) async fn update(
    pool: &PgPool,
    account_id: Uuid,
    wrapper_id: Uuid,
    wrapper: UpdateKeyWrapper,
) -> AppResult<VaultKeyWrapper> {
    let row = sqlx::query_as::<_, WrapperRow>(
        "UPDATE vault_key_wrappers SET revision = revision + 1, schema_version = $4, \
         key_version = $5, cipher_suite = $6, kdf_algorithm = $7, kdf_salt = $8, \
         kdf_memory_kib = $9, kdf_iterations = $10, kdf_parallelism = $11, nonce = $12, \
         wrapped_master_key_ciphertext = $13, updated_at = now() \
         WHERE account_id = $1 AND id = $2 AND revision = $3 AND deleted_at IS NULL \
         RETURNING id, wrapper_key, revision, schema_version, key_version, cipher_suite, \
          kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism, \
          nonce, wrapped_master_key_ciphertext, created_at, updated_at",
    )
    .bind(account_id)
    .bind(wrapper_id)
    .bind(wrapper.expected_revision)
    .bind(wrapper.schema_version)
    .bind(wrapper.key_version)
    .bind(wrapper.cipher_suite)
    .bind(wrapper.kdf_algorithm)
    .bind(wrapper.kdf_salt)
    .bind(i64::from(wrapper.kdf_memory_kib))
    .bind(i64::from(wrapper.kdf_iterations))
    .bind(i64::from(wrapper.kdf_parallelism))
    .bind(wrapper.nonce)
    .bind(wrapper.wrapped_master_key)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法更新包装密钥"))?;
    match row {
        Some(row) => wrapper_from_row(row),
        None => failure::revision(pool, account_id, wrapper_id, wrapper.expected_revision).await,
    }
}
