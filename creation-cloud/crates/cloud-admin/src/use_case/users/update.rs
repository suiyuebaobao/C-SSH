//! 在同一事务和管理员行锁内执行账号资料更新、策略保护与语义审计。

use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use cloud_notification::{AccountNotificationEvent, record_account_event};
use uuid::Uuid;

use crate::{
    AdminUpdateUserInput, AdminUser, AdminUserRole, AdminUserStatus, Service, repository,
    validation,
};

pub(super) struct ValidatedUpdate {
    email: Option<String>,
    display_name: Option<String>,
    admin_login_name: Option<String>,
    clear_admin_login_name: bool,
    role: Option<AdminUserRole>,
    status: Option<AdminUserStatus>,
    new_password: Option<String>,
}

impl Service {
    pub async fn update_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        input: AdminUpdateUserInput,
    ) -> AppResult<AdminUser> {
        let actor_id = validation::admin_actor(actor)?;
        let account_id = validation::valid_id(account_id, "账号标识")?;
        let input = validate_input(input)?;
        let password_hash = match input.new_password.as_deref() {
            Some(password) => Some(cloud_auth::hash_admin_password(password).await?),
            None => None,
        };

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
            actor_id,
            &locked,
            current_role,
            current_status,
            &input,
            active_admins.len(),
        )?;

        let invalidates_sessions = invalidates_sessions(
            &locked,
            current_role,
            current_status,
            &input,
            password_hash.is_some(),
        );
        let user = repository::users::update::apply(
            &mut transaction,
            account_id,
            repository::users::update::UserUpdate {
                email: input.email.as_deref(),
                display_name: input.display_name.as_deref(),
                admin_login_name: input.admin_login_name.as_deref(),
                clear_admin_login_name: input.clear_admin_login_name,
                role: input.role.map(AdminUserRole::as_str),
                status: input.status.map(AdminUserStatus::as_str),
                password_hash: password_hash.as_deref(),
            },
        )
        .await?;
        if invalidates_sessions {
            repository::sessions::delete_for_account(&mut transaction, account_id).await?;
            repository::users::challenges::delete_for_account(&mut transaction, account_id).await?;
        }
        repository::users::audit::record(
            &mut transaction,
            actor_id,
            account_id,
            "user.admin_update",
        )
        .await?;
        if password_hash.is_some() {
            record_account_event(
                &mut transaction,
                account_id,
                AccountNotificationEvent::PasswordChanged,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(repository::map_write_error)?;
        mark_semantic_audit_recorded();
        Ok(user)
    }
}

pub(super) fn validate_input(input: AdminUpdateUserInput) -> AppResult<ValidatedUpdate> {
    if input.email.is_none()
        && input.display_name.is_none()
        && input.admin_login_name.is_none()
        && !input.clear_admin_login_name
        && input.role.is_none()
        && input.status.is_none()
        && input.new_password.is_none()
    {
        return Err(AppError::Validation("至少需要提供一项账号修改".to_owned()));
    }
    if input.status == Some(AdminUserStatus::PendingVerification) {
        return Err(AppError::Validation(
            "待验证状态只能由邮箱验证流程管理".to_owned(),
        ));
    }
    if input.clear_admin_login_name && input.admin_login_name.is_some() {
        return Err(AppError::Validation(
            "不能同时设置并清空管理员登录名".to_owned(),
        ));
    }
    Ok(ValidatedUpdate {
        email: input
            .email
            .as_deref()
            .map(validation::account_email)
            .transpose()?,
        display_name: input
            .display_name
            .as_deref()
            .map(validation::display_name)
            .transpose()?,
        admin_login_name: input
            .admin_login_name
            .as_deref()
            .map(validation::admin_login_name::normalize)
            .transpose()?,
        clear_admin_login_name: input.clear_admin_login_name,
        role: input.role,
        status: input.status,
        new_password: input.new_password,
    })
}

pub(super) fn enforce_policy(
    actor_id: Uuid,
    locked: &repository::users::update::LockedAccount,
    current_role: AdminUserRole,
    current_status: AdminUserStatus,
    input: &ValidatedUpdate,
    active_admin_count: usize,
) -> AppResult<()> {
    let next_role = input.role.unwrap_or(current_role);
    let next_status = input.status.unwrap_or(current_status);
    let next_email_verified = if input.email.is_some() {
        next_status != AdminUserStatus::PendingVerification
    } else {
        locked.email_verified
    };
    let has_email = input.email.as_ref().or(locked.email.as_ref()).is_some();
    if next_status == AdminUserStatus::Active && (!has_email || !next_email_verified) {
        return Err(AppError::Conflict("有效账号必须保有已核验邮箱".to_owned()));
    }
    if next_status == AdminUserStatus::PendingVerification && next_role != AdminUserRole::User {
        return Err(AppError::Conflict(
            "待验证状态只允许普通用户使用".to_owned(),
        ));
    }
    if next_role == AdminUserRole::User && input.admin_login_name.is_some() {
        return Err(AppError::Validation(
            "普通用户不能设置管理员登录名".to_owned(),
        ));
    }
    if locked.id == actor_id
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

pub(super) fn invalidates_sessions(
    locked: &repository::users::update::LockedAccount,
    current_role: AdminUserRole,
    current_status: AdminUserStatus,
    input: &ValidatedUpdate,
    password_changed: bool,
) -> bool {
    let next_role = input.role.unwrap_or(current_role);
    let next_login = if next_role == AdminUserRole::User || input.clear_admin_login_name {
        None
    } else {
        input
            .admin_login_name
            .as_ref()
            .or(locked.admin_login_name.as_ref())
    };
    input
        .email
        .as_ref()
        .is_some_and(|email| Some(email) != locked.email.as_ref())
        || next_login != locked.admin_login_name.as_ref()
        || next_role != current_role
        || input.status.unwrap_or(current_status) != current_status
        || password_changed
}
