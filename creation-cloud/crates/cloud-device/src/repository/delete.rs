//! 以撤销时间实现设备删除语义并保留审计身份。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) const REVOKE_SQL: &str = "UPDATE devices SET revoked_at = now(), updated_at = now() \
     WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL";

pub(crate) async fn revoke(pool: &PgPool, account_id: Uuid, device_id: Uuid) -> AppResult<u64> {
    sqlx::query(REVOKE_SQL)
        .bind(account_id)
        .bind(device_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(error::storage)
}
