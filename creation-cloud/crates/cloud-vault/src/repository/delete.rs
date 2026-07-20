//! 以 revision 乐观锁为密文信封写入墓碑，不删除历史字节。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::DeleteVaultOutcome;

use super::storage;

type DeleteRow = (Uuid, i64, DateTime<Utc>);

pub(crate) async fn delete(
    pool: &PgPool,
    account_id: Uuid,
    envelope_id: Uuid,
    expected_revision: i64,
) -> AppResult<DeleteVaultOutcome> {
    let row = sqlx::query_as::<_, DeleteRow>(
        "UPDATE vault_envelopes SET revision = revision + 1, deleted_at = now(), updated_at = now() \
         WHERE account_id = $1 AND id = $2 AND revision = $3 AND deleted_at IS NULL \
         RETURNING id, revision, deleted_at",
    )
    .bind(account_id)
    .bind(envelope_id)
    .bind(expected_revision)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法删除密文信封"))?;
    if let Some(row) = row {
        return Ok(DeleteVaultOutcome {
            id: row.0,
            revision: row.1,
            deleted_at: row.2,
        });
    }
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT revision FROM vault_envelopes \
         WHERE account_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .bind(envelope_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法确认密文信封删除状态"))?;
    match current {
        Some(current) => Err(AppError::Conflict(format!(
            "密文信封 revision 冲突：期望 {expected_revision}，当前 {current}"
        ))),
        None => Err(AppError::NotFound("密文信封不存在".to_owned())),
    }
}
