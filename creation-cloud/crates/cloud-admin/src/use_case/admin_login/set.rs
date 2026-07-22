//! 在同一事务中校验 active admin、设置规范化登录名并写系统审计。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{repository, validation};

pub async fn set_registered_admin_login(
    pool: &PgPool,
    registered_email: &str,
    admin_login_name: &str,
) -> AppResult<Uuid> {
    let email = validation::email_filter(registered_email)?;
    let login_name = validation::admin_login_name::normalize(admin_login_name)?;
    let mut transaction = pool.begin().await.map_err(repository::map_write_error)?;
    let account = repository::admin_login::set::lock_account(&mut transaction, &email).await?;
    validate_target(&account.role, &account.status)?;

    repository::admin_login::set::apply(&mut transaction, account.id, &login_name).await?;
    // 身份标识变化后撤销旧会话，避免既有令牌延续变更前的管理身份。
    repository::sessions::delete_for_account(&mut transaction, account.id).await?;
    repository::admin_login::set::record_audit(&mut transaction, account.id).await?;
    transaction
        .commit()
        .await
        .map_err(repository::map_write_error)?;
    Ok(account.id)
}

pub(crate) fn validate_target(role: &str, status: &str) -> AppResult<()> {
    if status != "active" {
        return Err(AppError::Conflict("只能配置有效管理员账号".to_owned()));
    }
    match role {
        "admin" => Ok(()),
        "user" => Err(AppError::Conflict("只能配置有效管理员账号".to_owned())),
        _ => Err(AppError::Internal("数据库中的账号角色无效".to_owned())),
    }
}
