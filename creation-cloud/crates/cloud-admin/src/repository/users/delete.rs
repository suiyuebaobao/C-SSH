//! 永久删除账号，并将不可变全局资源外键阻断映射为明确冲突。

use cloud_domain::{AppError, AppResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) const DELETE_ACCOUNT_SQL: &str = "DELETE FROM accounts WHERE id = $1";

pub(crate) async fn execute(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    let result = sqlx::query(DELETE_ACCOUNT_SQL)
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(map_delete_error)?;
    if result.rows_affected() != 1 {
        return Err(AppError::NotFound("账号不存在".to_owned()));
    }
    Ok(())
}

fn map_delete_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_foreign_key_violation()) {
        AppError::Conflict("账号仍承担不可变全局资源责任，无法删除".to_owned())
    } else {
        map_write_error(error)
    }
}
