//! 按账号主键硬删除单个用户资料。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) async fn remove(pool: &PgPool, account_id: Uuid) -> AppResult<u64> {
    sqlx::query("DELETE FROM user_profiles WHERE account_id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(error::storage)
}
