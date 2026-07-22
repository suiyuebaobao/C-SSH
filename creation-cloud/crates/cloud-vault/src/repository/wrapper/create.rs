//! 创建当前账号的包装密钥密文，并由数据库保证 active key 唯一。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{VaultKeyWrapper, repository::storage, wrapper_types::CreateKeyWrapper};

use super::{WrapperRow, wrapper_from_row};

pub(crate) async fn create(
    pool: &PgPool,
    account_id: Uuid,
    wrapper: CreateKeyWrapper,
) -> AppResult<VaultKeyWrapper> {
    let row = sqlx::query_as::<_, WrapperRow>(
        "INSERT INTO vault_key_wrappers \
         (id, account_id, wrapper_key, revision, schema_version, key_version, cipher_suite, \
          kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism, \
          nonce, wrapped_master_key_ciphertext) \
         VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
         ON CONFLICT (account_id, wrapper_key) WHERE deleted_at IS NULL DO NOTHING \
         RETURNING id, wrapper_key, revision, schema_version, key_version, cipher_suite, \
          kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism, \
          nonce, wrapped_master_key_ciphertext, created_at, updated_at",
    )
    .bind(wrapper.id)
    .bind(account_id)
    .bind(wrapper.wrapper_key)
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
    .map_err(storage("无法创建包装密钥"))?
    .ok_or_else(|| AppError::Conflict("active wrapper_key 已存在".to_owned()))?;
    wrapper_from_row(row)
}
