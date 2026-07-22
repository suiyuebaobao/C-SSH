//! 以 revision 乐观锁软删除包装密钥，只写墓碑而不物理删除密文字节。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{DeleteVaultKeyWrapperOutcome, repository::storage};

use super::failure;

type DeleteRow = (Uuid, i64, DateTime<Utc>);

pub(crate) async fn delete(
    pool: &PgPool,
    account_id: Uuid,
    wrapper_id: Uuid,
    expected_revision: i64,
) -> AppResult<DeleteVaultKeyWrapperOutcome> {
    let row = sqlx::query_as::<_, DeleteRow>(
        "UPDATE vault_key_wrappers \
         SET revision = revision + 1, deleted_at = now(), updated_at = now() \
         WHERE account_id = $1 AND id = $2 AND revision = $3 AND deleted_at IS NULL \
         RETURNING id, revision, deleted_at",
    )
    .bind(account_id)
    .bind(wrapper_id)
    .bind(expected_revision)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法删除包装密钥"))?;
    match row {
        Some(row) => Ok(DeleteVaultKeyWrapperOutcome {
            id: row.0,
            revision: row.1,
            deleted_at: row.2,
        }),
        None => failure::revision(pool, account_id, wrapper_id, expected_revision).await,
    }
}
