//! 锁定目标账号、写入管理员登录名并追加不含敏感标识的系统审计。

use cloud_domain::{AppError, AppResult};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::repository::map_write_error;

#[derive(Debug, FromRow)]
pub(crate) struct LockedAccount {
    pub id: Uuid,
    pub role: String,
    pub status: String,
}

pub(crate) const LOCK_ACCOUNT_SQL: &str =
    "SELECT id, role, status FROM accounts WHERE lower(email) = $1 FOR UPDATE";
pub(crate) const SET_LOGIN_SQL: &str = r#"
    UPDATE accounts
    SET admin_login_name = $2, updated_at = now()
    WHERE id = $1 AND role = 'admin' AND status = 'active'
"#;
pub(crate) const AUDIT_SQL: &str = r#"
    INSERT INTO audit_events (
        id, actor_account_id, action, resource_kind, resource_id,
        outcome, request_id, details
    )
    VALUES ($1, NULL, 'system.admin_login_set', 'account', $2, 'success', NULL, '{}'::jsonb)
"#;

pub(crate) async fn lock_account(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> AppResult<LockedAccount> {
    sqlx::query_as::<_, LockedAccount>(LOCK_ACCOUNT_SQL)
        .bind(email)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("有效注册账号不存在".to_owned()))
}

pub(crate) async fn apply(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    login_name: &str,
) -> AppResult<()> {
    let updated = sqlx::query(SET_LOGIN_SQL)
        .bind(account_id)
        .bind(login_name)
        .execute(&mut **transaction)
        .await
        .map_err(map_login_name_write_error)?;
    if updated.rows_affected() != 1 {
        return Err(AppError::Conflict("只能配置有效管理员账号".to_owned()));
    }
    Ok(())
}

pub(crate) async fn record_audit(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(AUDIT_SQL)
        .bind(Uuid::now_v7())
        .bind(account_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    Ok(())
}

fn map_login_name_write_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("管理员登录名不可用".to_owned())
    } else {
        map_write_error(error)
    }
}
