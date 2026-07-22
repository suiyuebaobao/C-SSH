//! 区分包装密钥不存在与 revision 冲突，避免乐观锁失败被吞掉。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::repository::storage;

pub(crate) async fn revision<T>(
    pool: &PgPool,
    account_id: Uuid,
    wrapper_id: Uuid,
    expected_revision: i64,
) -> AppResult<T> {
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT revision FROM vault_key_wrappers \
         WHERE account_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .bind(wrapper_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法确认包装密钥 revision"))?;
    match current {
        Some(current) => Err(AppError::Conflict(format!(
            "包装密钥 revision 冲突：期望 {expected_revision}，当前 {current}"
        ))),
        None => Err(AppError::NotFound("包装密钥不存在".to_owned())),
    }
}
