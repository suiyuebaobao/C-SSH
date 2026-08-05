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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_self_and_last_active_admin() {
        let actor_id = Uuid::now_v7();
        assert!(matches!(
            enforce_policy(
                actor_id,
                actor_id,
                AdminUserRole::Admin,
                AdminUserStatus::Active,
                2,
            ),
            Err(AppError::Forbidden(_))
        ));
        assert!(matches!(
            enforce_policy(
                actor_id,
                Uuid::now_v7(),
                AdminUserRole::Admin,
                AdminUserStatus::Active,
                1,
            ),
            Err(AppError::Conflict(_))
        ));
    }
}
