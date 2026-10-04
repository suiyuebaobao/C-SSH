//! 校验并哈希管理员提供的初始密码，再原子创建账号与语义审计。

use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use uuid::Uuid;

use crate::{
    AdminCreateUserInput, AdminUser, AdminUserRole, AdminUserStatus, Service, repository,
    validation,
};

struct ValidatedCreate {
    email: String,
    password: String,
    display_name: String,
    role: AdminUserRole,
    status: AdminUserStatus,
    admin_login_name: Option<String>,
}

impl Service {
    pub async fn create_user(
        &self,
        actor: &AdminActor,
        input: AdminCreateUserInput,
    ) -> AppResult<AdminUser> {
        let actor_id = validation::admin_actor(actor)?;
        let input = validate(input)?;
        let password_hash = cloud_auth::hash_admin_password(&input.password).await?;
        let account_id = Uuid::now_v7();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_write_error)?;
        let user = repository::users::create::execute(
            &mut transaction,
            repository::users::create::NewUser {
                id: account_id,
                email: &input.email,
                password_hash: &password_hash,
                display_name: &input.display_name,
                role: input.role.as_str(),
                status: input.status.as_str(),
                admin_login_name: input.admin_login_name.as_deref(),
            },
        )
        .await?;
        repository::users::audit::record(
            &mut transaction,
            actor_id,
            account_id,
            "user.admin_create",
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_write_error)?;
        mark_semantic_audit_recorded();
        Ok(user)
    }
}

fn validate(input: AdminCreateUserInput) -> AppResult<ValidatedCreate> {
    let role = input.role.unwrap_or(AdminUserRole::User);
    let status = input.status.unwrap_or(AdminUserStatus::Active);
    if status == AdminUserStatus::PendingVerification {
        return Err(AppError::Validation(
            "管理员创建账号只允许 active 或 disabled 状态".to_owned(),
        ));
    }
    let admin_login_name = input
        .admin_login_name
        .as_deref()
        .map(validation::admin_login_name::normalize)
        .transpose()?;
    if role == AdminUserRole::User && admin_login_name.is_some() {
        return Err(AppError::Validation(
            "普通用户不能设置管理员登录名".to_owned(),
        ));
    }
    Ok(ValidatedCreate {
        email: validation::account_email(&input.email)?,
        password: input.password,
        display_name: validation::display_name(&input.display_name)?,
        role,
        status,
        admin_login_name,
    })
}
