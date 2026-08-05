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

#[cfg(test)]
mod tests {
    use cloud_domain::AppError;

    use super::*;

    fn input() -> AdminCreateUserInput {
        AdminCreateUserInput {
            email: " Person@Example.com ".to_owned(),
            password: "correct-horse-battery".to_owned(),
            display_name: " Person ".to_owned(),
            role: None,
            status: None,
            admin_login_name: None,
        }
    }

    #[test]
    fn normalizes_defaults_without_exposing_password() {
        let value = validate(input()).expect("创建输入应有效");
        assert_eq!(value.email, "person@example.com");
        assert_eq!(value.display_name, "Person");
        assert_eq!(value.role, AdminUserRole::User);
        assert_eq!(value.status, AdminUserStatus::Active);
    }

    #[test]
    fn rejects_login_name_for_normal_user_and_pending_status() {
        let mut value = input();
        value.admin_login_name = Some("ops-admin".to_owned());
        assert!(matches!(validate(value), Err(AppError::Validation(_))));

        let mut value = input();
        value.status = Some(AdminUserStatus::PendingVerification);
        assert!(matches!(validate(value), Err(AppError::Validation(_))));
    }
}
