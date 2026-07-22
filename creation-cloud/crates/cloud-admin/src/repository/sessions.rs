//! 集中撤销指定账号的全部 Creation Cloud 会话。

use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) const DELETE_ACCOUNT_SESSIONS_SQL: &str = "DELETE FROM sessions WHERE account_id = $1";

pub(crate) async fn delete_for_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(DELETE_ACCOUNT_SESSIONS_SQL)
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    Ok(())
}
