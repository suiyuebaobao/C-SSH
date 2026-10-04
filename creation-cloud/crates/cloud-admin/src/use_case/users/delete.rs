//! 在末位管理员保护下永久删除账号及其级联用户数据。

use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use uuid::Uuid;

use crate::{AdminUserRole, AdminUserStatus, Service, repository, validation};

impl Service {
    pub async fn delete_user(&self, actor: &AdminActor, account_id: Uuid) -> AppResult<()> {
        let actor_id = validation::admin_actor(actor)?;
        let account_id = validation::valid_id(account_id, "账号标识")?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_write_error)?;
        let active_admins = repository::users::update::lock_active_admins(&mut transaction).await?;
        let target = repository::users::update::lock_account(&mut transaction, account_id).await?;
        enforce_policy(
            actor_id,
            target.id,
            AdminUserRole::try_from(target.role.as_str())?,
            AdminUserStatus::try_from(target.status.as_str())?,
            active_admins.len(),
        )?;
        repository::users::audit::record(
            &mut transaction,
            actor_id,
            account_id,
            "user.admin_delete",
        )
        .await?;
        repository::users::delete::execute(&mut transaction, account_id).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_write_error)?;
        mark_semantic_audit_recorded();
        Ok(())
    }
}

pub(crate) fn enforce_policy(
    actor_id: Uuid,
    target_id: Uuid,
    role: AdminUserRole,
    status: AdminUserStatus,
    active_admin_count: usize,
) -> AppResult<()> {
    if actor_id == target_id {
        return Err(AppError::Forbidden("管理员不能删除自己的账号".to_owned()));
    }
    if role == AdminUserRole::Admin && status == AdminUserStatus::Active && active_admin_count <= 1
    {
        return Err(AppError::Conflict("不能删除最后一个有效管理员".to_owned()));
    }
    Ok(())
}
