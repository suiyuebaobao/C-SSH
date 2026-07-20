//! 在同一事务和管理员行锁内执行角色状态变更、末位保护与会话清理。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminUpdateUserInput, AdminUser, AdminUserRole, AdminUserStatus, Service, repository,
    validation,
};

impl Service {
    pub async fn update_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        input: AdminUpdateUserInput,
    ) -> AppResult<AdminUser> {
        validation::admin_actor(actor)?;
        let account_id = validation::valid_id(account_id, "账号标识")?;
        validate_input(input)?;

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_write_error)?;
        let active_admins = repository::users::update::lock_active_admins(&mut transaction).await?;
        let locked = repository::users::update::lock_account(&mut transaction, account_id).await?;
        let current_role = AdminUserRole::try_from(locked.role.as_str())?;
        let current_status = AdminUserStatus::try_from(locked.status.as_str())?;
        enforce_policy(
            actor,
            locked.id,
            current_role,
            current_status,
            input,
            active_admins.len(),
        )?;

        let next_status = input.status.unwrap_or(current_status);
        let user = repository::users::update::apply(&mut transaction, account_id, input).await?;
        if next_status == AdminUserStatus::Disabled {
            repository::users::update::delete_sessions(&mut transaction, account_id).await?;
        }
        transaction
            .commit()
            .await
            .map_err(repository::map_write_error)?;
        Ok(user)
    }
}

pub(crate) fn validate_input(input: AdminUpdateUserInput) -> AppResult<()> {
    if input.role.is_none() && input.status.is_none() {
        return Err(AppError::Validation(
            "角色或账号状态至少需要提供一项".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn enforce_policy(
    actor: &AdminActor,
    target_id: Uuid,
    current_role: AdminUserRole,
    current_status: AdminUserStatus,
    input: AdminUpdateUserInput,
    active_admin_count: usize,
) -> AppResult<()> {
    let next_role = input.role.unwrap_or(current_role);
    let next_status = input.status.unwrap_or(current_status);
    if target_id == actor.account_id()
        && (next_role == AdminUserRole::User || next_status == AdminUserStatus::Disabled)
    {
        return Err(AppError::Forbidden(
            "管理员不能禁用或降低自己的权限".to_owned(),
        ));
    }
    let removes_active_admin = current_role == AdminUserRole::Admin
        && current_status == AdminUserStatus::Active
        && (next_role != AdminUserRole::Admin || next_status != AdminUserStatus::Active);
    if removes_active_admin && active_admin_count <= 1 {
        return Err(AppError::Conflict(
            "不能禁用或降权最后一个有效管理员".to_owned(),
        ));
    }
    Ok(())
}
