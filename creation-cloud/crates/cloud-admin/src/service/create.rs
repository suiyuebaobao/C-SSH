//! 创建不依赖邮箱的本地管理员账号。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{repository::map_write_error, validation};

const INSERT_ACCOUNT_SQL: &str = r#"
    INSERT INTO accounts (
        id, email, admin_login_name, password_hash, role, status,
        email_verified_at, credential_version
    )
    VALUES ($1, NULL, $2, $3, 'admin', 'active', NULL, 1)
"#;

const INSERT_PROFILE_SQL: &str = r#"
    INSERT INTO user_profiles (account_id, display_name, locale)
    VALUES ($1, $2, 'zh-CN')
"#;

const INSERT_AUDIT_SQL: &str = r#"
    INSERT INTO audit_events (
        id, actor_account_id, action, resource_kind, resource_id,
        outcome, request_id, details
    )
    VALUES (
        $1, NULL, 'system.admin_created', 'account', $2,
        'success', NULL, '{}'::jsonb
    )
"#;

pub async fn create_local_admin(
    pool: &PgPool,
    admin_login_name: &str,
    password_hash: &str,
) -> AppResult<Uuid> {
    let login_name = validation::admin_login_name::normalize(admin_login_name)?;
    if password_hash.is_empty() || password_hash.len() > 1024 {
        return Err(AppError::Validation("管理员密码摘要无效".to_owned()));
    }

    let account_id = Uuid::now_v7();
    let mut transaction = pool.begin().await.map_err(map_write_error)?;
    sqlx::query(INSERT_ACCOUNT_SQL)
        .bind(account_id)
        .bind(&login_name)
        .bind(password_hash)
        .execute(&mut *transaction)
        .await
        .map_err(map_create_error)?;
    sqlx::query(INSERT_PROFILE_SQL)
        .bind(account_id)
        .bind(&login_name)
        .execute(&mut *transaction)
        .await
        .map_err(map_write_error)?;
    sqlx::query(INSERT_AUDIT_SQL)
        .bind(Uuid::now_v7())
        .bind(account_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(map_write_error)?;
    transaction.commit().await.map_err(map_write_error)?;
    Ok(account_id)
}

fn map_create_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("管理员登录名不可用".to_owned())
    } else {
        map_write_error(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_admin_insert_has_no_email_dependency() {
        assert!(INSERT_ACCOUNT_SQL.contains("VALUES ($1, NULL, $2, $3"));
        assert!(INSERT_ACCOUNT_SQL.contains("'admin', 'active'"));
        assert!(!INSERT_AUDIT_SQL.contains("admin_login_name"));
    }
}
