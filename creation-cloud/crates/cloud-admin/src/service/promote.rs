//! 提供不经 HTTP 的已注册有效账号管理员提升，并在同一事务写入系统审计。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{repository, validation};

#[derive(Debug, FromRow)]
struct PromotionAccount {
    id: Uuid,
    role: String,
    status: String,
}

pub(crate) const LOCK_ACCOUNT_SQL: &str =
    "SELECT id, role, status FROM accounts WHERE lower(email) = $1 FOR UPDATE";
pub(crate) const PROMOTE_SQL: &str =
    "UPDATE accounts SET role = 'admin', updated_at = now() WHERE id = $1";
pub(crate) const AUDIT_SQL: &str = r#"
    INSERT INTO audit_events (
        id, actor_account_id, action, resource_kind, resource_id,
        outcome, request_id, details
    )
    VALUES ($1, NULL, 'system.admin_promote', 'account', $2, 'success', NULL, '{}'::jsonb)
"#;

pub async fn promote_registered_admin(pool: &PgPool, email: &str) -> AppResult<Uuid> {
    let email = validation::email_filter(email)?;
    let mut transaction = pool.begin().await.map_err(repository::map_write_error)?;
    let account = sqlx::query_as::<_, PromotionAccount>(LOCK_ACCOUNT_SQL)
        .bind(email)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(repository::map_write_error)?
        .ok_or_else(|| AppError::NotFound("有效注册账号不存在".to_owned()))?;
    if account.status != "active" {
        return Err(AppError::Conflict("只能提升有效注册账号".to_owned()));
    }
    if account.role == "admin" {
        return Err(AppError::Conflict("该账号已经是管理员".to_owned()));
    }
    if account.role != "user" {
        return Err(AppError::Internal("数据库中的账号角色无效".to_owned()));
    }

    sqlx::query(PROMOTE_SQL)
        .bind(account.id)
        .execute(&mut *transaction)
        .await
        .map_err(repository::map_write_error)?;
    // 角色提升必须令此前签发的普通用户会话失效，避免旧令牌继承管理员权限。
    repository::sessions::delete_for_account(&mut transaction, account.id).await?;
    sqlx::query(AUDIT_SQL)
        .bind(Uuid::now_v7())
        .bind(account.id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(repository::map_write_error)?;
    transaction
        .commit()
        .await
        .map_err(repository::map_write_error)?;
    Ok(account.id)
}
