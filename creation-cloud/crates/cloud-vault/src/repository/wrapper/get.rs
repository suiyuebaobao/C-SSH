//! 按账号所有权读取一条未软删除的包装密钥。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultKeyWrapper, repository::storage};

use super::{WrapperRow, wrapper_from_row};

pub(crate) async fn get(
    pool: &PgPool,
    account_id: Uuid,
    wrapper_id: Uuid,
) -> AppResult<VaultKeyWrapper> {
    let row = sqlx::query_as::<_, WrapperRow>(
        "SELECT id, wrapper_key, revision, schema_version, key_version, cipher_suite, \
         kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism, \
         nonce, wrapped_master_key_ciphertext, created_at, updated_at \
         FROM vault_key_wrappers \
         WHERE account_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .bind(wrapper_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法读取包装密钥"))?
    .ok_or_else(|| AppError::NotFound("包装密钥不存在".to_owned()))?;
    wrapper_from_row(row)
}
