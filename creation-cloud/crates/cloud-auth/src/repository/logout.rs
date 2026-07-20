//! 精确删除当前账号拥有的单个会话。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) async fn delete(pool: &PgPool, session_id: Uuid, account_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE id = $1 AND account_id = $2")
        .bind(session_id)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(error::storage)?;
    Ok(())
}
