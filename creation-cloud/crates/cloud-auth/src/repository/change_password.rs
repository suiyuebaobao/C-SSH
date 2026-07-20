//! 读取当前密码哈希，并原子更新密码与撤销其他会话。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) async fn current_hash(pool: &PgPool, account_id: Uuid) -> AppResult<Option<String>> {
    sqlx::query_as::<_, (String,)>(
        "SELECT password_hash FROM accounts WHERE id = $1 AND status = 'active'",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(|value| value.0))
    .map_err(error::storage)
}

pub(crate) async fn update(
    pool: &PgPool,
    account_id: Uuid,
    current_session_id: Uuid,
    password_hash: &str,
    revoke_other_sessions: bool,
) -> AppResult<()> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let result = sqlx::query(
        "UPDATE accounts SET password_hash = $2, updated_at = now() \
         WHERE id = $1 AND status = 'active'",
    )
    .bind(account_id)
    .bind(password_hash)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if result.rows_affected() != 1 {
        return Err(AppError::Unauthorized("账号不可用".to_owned()));
    }

    if revoke_other_sessions {
        sqlx::query("DELETE FROM sessions WHERE account_id = $1 AND id <> $2")
            .bind(account_id)
            .bind(current_session_id)
            .execute(&mut *transaction)
            .await
            .map_err(error::storage)?;
    }
    transaction.commit().await.map_err(error::storage)
}
