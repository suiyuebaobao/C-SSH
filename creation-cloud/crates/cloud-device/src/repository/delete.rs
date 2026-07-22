//! 在账号与设备锁内撤销设备，并先清除该设备的全部会话。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) const LOCK_ACCOUNT_SQL: &str =
    "SELECT id FROM accounts WHERE id = $1 AND status = 'active' FOR UPDATE";
pub(crate) const LOCK_DEVICE_SQL: &str = "SELECT id FROM devices \
     WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL FOR UPDATE";
pub(crate) const DELETE_DEVICE_SESSIONS_SQL: &str =
    "DELETE FROM sessions WHERE account_id = $1 AND device_id = $2";
pub(crate) const REVOKE_SQL: &str = "UPDATE devices SET revoked_at = now(), updated_at = now() \
     WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL";

pub(crate) async fn revoke(pool: &PgPool, account_id: Uuid, device_id: Uuid) -> AppResult<u64> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let account_exists = sqlx::query_scalar::<_, Uuid>(LOCK_ACCOUNT_SQL)
        .bind(account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .is_some();
    if !account_exists {
        return Err(AppError::Unauthorized("账号不可用".to_owned()));
    }
    let device_exists = sqlx::query_scalar::<_, Uuid>(LOCK_DEVICE_SQL)
        .bind(account_id)
        .bind(device_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .is_some();
    if !device_exists {
        return Ok(0);
    }
    sqlx::query(DELETE_DEVICE_SESSIONS_SQL)
        .bind(account_id)
        .bind(device_id)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    let result = sqlx::query(REVOKE_SQL)
        .bind(account_id)
        .bind(device_id)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(result.rows_affected())
}
