//! 读取四项认证开关与三项安全策略，并由管理员以一个 revision/CAS 原子更新。

use chrono::{DateTime, Utc};
use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository;

#[derive(Clone, Debug, Serialize)]
pub struct AuthSettings {
    pub email_verification_enabled: bool,
    pub user_captcha_enabled: bool,
    pub admin_email_verification_enabled: bool,
    pub admin_captcha_enabled: bool,
    pub email_cooldown_seconds: i32,
    pub login_failure_threshold: i32,
    pub login_lockout_minutes: i32,
    pub revision: i64,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateAuthSettings {
    pub email_verification_enabled: bool,
    pub user_captcha_enabled: bool,
    pub admin_email_verification_enabled: bool,
    pub admin_captcha_enabled: bool,
    pub email_cooldown_seconds: i32,
    pub login_failure_threshold: i32,
    pub login_lockout_minutes: i32,
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientLoginConfig {
    pub revision: i64,
    pub captcha_enabled: bool,
    pub email_code_enabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct LoginCaptchaSettings {
    pub user_captcha_enabled: bool,
    pub admin_captcha_enabled: bool,
}

pub(crate) async fn client_login_config(pool: &PgPool) -> AppResult<ClientLoginConfig> {
    repository::settings::client_login_config(pool).await
}

pub(crate) async fn login_captcha_settings(pool: &PgPool) -> AppResult<LoginCaptchaSettings> {
    let settings = repository::settings::read(pool).await?;
    Ok(LoginCaptchaSettings {
        user_captcha_enabled: settings.user_captcha_enabled,
        admin_captcha_enabled: settings.admin_captcha_enabled,
    })
}

pub(crate) async fn get(pool: &PgPool, actor: &AdminActor) -> AppResult<AuthSettings> {
    require_actor(actor)?;
    repository::settings::read(pool).await
}

pub(crate) async fn update(
    pool: &PgPool,
    actor: &AdminActor,
    input: UpdateAuthSettings,
) -> AppResult<AuthSettings> {
    let actor_id = require_actor(actor)?;
    validate_input(&input)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| AppError::Storage("认证设置事务启动失败".to_owned()))?;
    let current = repository::settings::lock(&mut transaction).await?;
    if current.revision != input.expected_revision {
        return Err(AppError::Conflict(
            "认证设置已变化，请刷新后重试".to_owned(),
        ));
    }
    if current.email_verification_enabled == input.email_verification_enabled
        && current.user_captcha_enabled == input.user_captcha_enabled
        && current.admin_email_verification_enabled == input.admin_email_verification_enabled
        && current.admin_captcha_enabled == input.admin_captcha_enabled
        && current.email_cooldown_seconds == input.email_cooldown_seconds
        && current.login_failure_threshold == input.login_failure_threshold
        && current.login_lockout_minutes == input.login_lockout_minutes
    {
        return Err(AppError::Conflict("认证设置没有变化".to_owned()));
    }
    if !current.admin_email_verification_enabled
        && input.admin_email_verification_enabled
        && !repository::settings::all_active_admins_have_verified_email(&mut transaction).await?
    {
        return Err(AppError::Validation(
            "启用管理员邮箱验证码前，所有有效管理员都必须绑定并验证邮箱".to_owned(),
        ));
    }
    let disable_email = current.email_verification_enabled && !input.email_verification_enabled;
    let disable_captcha = current.user_captcha_enabled && !input.user_captcha_enabled;
    let disable_admin_email =
        current.admin_email_verification_enabled && !input.admin_email_verification_enabled;
    let disable_admin_captcha = current.admin_captcha_enabled && !input.admin_captcha_enabled;
    let updated = repository::settings::update(&mut transaction, actor_id, &input).await?;
    let email_invalidated = if disable_email {
        repository::settings::invalidate_open_email_challenges(&mut transaction).await?
    } else {
        (0, 0)
    };
    let captcha_invalidated = if disable_captcha {
        repository::settings::invalidate_open_user_captchas(&mut transaction).await?
    } else {
        0
    };
    let admin_login_invalidated = if disable_admin_email {
        repository::settings::invalidate_open_admin_login_challenges(&mut transaction).await?
    } else {
        0
    };
    let admin_captcha_invalidated = if disable_admin_captcha {
        repository::settings::invalidate_open_admin_captchas(&mut transaction).await?
    } else {
        0
    };
    repository::settings::audit(
        &mut transaction,
        actor_id,
        &updated,
        repository::settings::InvalidatedChallenges {
            registration: email_invalidated.0,
            user_login: email_invalidated.1,
            user_captcha: captcha_invalidated,
            admin_login: admin_login_invalidated,
            admin_captcha: admin_captcha_invalidated,
        },
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| AppError::Storage("认证设置事务提交失败".to_owned()))?;
    mark_semantic_audit_recorded();
    Ok(updated)
}

fn validate_input(input: &UpdateAuthSettings) -> AppResult<()> {
    if input.expected_revision < 1 {
        return Err(AppError::Validation(
            "expected_revision 必须大于零".to_owned(),
        ));
    }
    if !input.admin_email_verification_enabled && !input.admin_captcha_enabled {
        return Err(AppError::Validation(
            "管理员邮箱验证码和图形验证码不能同时关闭".to_owned(),
        ));
    }
    if !(30..=3_600).contains(&input.email_cooldown_seconds) {
        return Err(AppError::Validation(
            "邮件获取间隔必须在 30 到 3600 秒之间".to_owned(),
        ));
    }
    if !(3..=20).contains(&input.login_failure_threshold) {
        return Err(AppError::Validation(
            "连续失败次数必须在 3 到 20 次之间".to_owned(),
        ));
    }
    if !(1..=1_440).contains(&input.login_lockout_minutes) {
        return Err(AppError::Validation(
            "登录锁定时长必须在 1 到 1440 分钟之间".to_owned(),
        ));
    }
    Ok(())
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let actor_id = actor.account_id();
    if actor_id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".to_owned()))
    } else {
        Ok(actor_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn administrator_factors_cannot_both_be_disabled() {
        let input = UpdateAuthSettings {
            email_verification_enabled: true,
            user_captcha_enabled: true,
            admin_email_verification_enabled: false,
            admin_captcha_enabled: false,
            email_cooldown_seconds: 60,
            login_failure_threshold: 5,
            login_lockout_minutes: 30,
            expected_revision: 1,
        };
        assert!(matches!(
            validate_input(&input),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn numeric_policy_fields_enforce_backend_ranges() {
        let mut input = UpdateAuthSettings {
            email_verification_enabled: true,
            user_captcha_enabled: true,
            admin_email_verification_enabled: false,
            admin_captcha_enabled: true,
            email_cooldown_seconds: 60,
            login_failure_threshold: 5,
            login_lockout_minutes: 30,
            expected_revision: 1,
        };
        assert!(validate_input(&input).is_ok());

        input.email_cooldown_seconds = 29;
        assert!(matches!(
            validate_input(&input),
            Err(AppError::Validation(_))
        ));
        input.email_cooldown_seconds = 60;
        input.login_failure_threshold = 21;
        assert!(matches!(
            validate_input(&input),
            Err(AppError::Validation(_))
        ));
        input.login_failure_threshold = 5;
        input.login_lockout_minutes = 0;
        assert!(matches!(
            validate_input(&input),
            Err(AppError::Validation(_))
        ));
    }
}
