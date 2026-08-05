//! 串行读取和更新全局认证设置，并在同一事务作废旧挑战、写语义审计。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::use_case::auth_settings::{AuthSettings, ClientLoginConfig, UpdateAuthSettings};

use super::error;

type SettingsRow = (
    bool,
    bool,
    bool,
    bool,
    i32,
    i32,
    i32,
    i64,
    Option<Uuid>,
    DateTime<Utc>,
);

pub(crate) struct InvalidatedChallenges {
    pub registration: u64,
    pub user_login: u64,
    pub user_captcha: u64,
    pub admin_login: u64,
    pub admin_captcha: u64,
}

pub(crate) async fn read(pool: &PgPool) -> AppResult<AuthSettings> {
    sqlx::query_as::<_, SettingsRow>(
        "SELECT email_verification_enabled, user_captcha_enabled, \
                admin_email_verification_enabled, admin_captcha_enabled, \
                email_cooldown_seconds, login_failure_threshold, login_lockout_minutes, \
                revision, updated_by, updated_at \
         FROM auth_settings WHERE singleton = TRUE",
    )
    .fetch_optional(pool)
    .await
    .map_err(error::storage)?
    .map(into_settings)
    .ok_or_else(missing)
}

pub(crate) async fn lock(transaction: &mut Transaction<'_, Postgres>) -> AppResult<AuthSettings> {
    sqlx::query_as::<_, SettingsRow>(
        "SELECT email_verification_enabled, user_captcha_enabled, \
                admin_email_verification_enabled, admin_captcha_enabled, \
                email_cooldown_seconds, login_failure_threshold, login_lockout_minutes, \
                revision, updated_by, updated_at \
         FROM auth_settings WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .map(into_settings)
    .ok_or_else(missing)
}

pub(crate) async fn client_login_config(pool: &PgPool) -> AppResult<ClientLoginConfig> {
    let settings = read(pool).await?;
    Ok(ClientLoginConfig {
        revision: settings.revision,
        captcha_enabled: settings.user_captcha_enabled,
        email_code_enabled: settings.email_verification_enabled,
    })
}

pub(crate) async fn update(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    input: &UpdateAuthSettings,
) -> AppResult<AuthSettings> {
    sqlx::query_as::<_, SettingsRow>(
        "UPDATE auth_settings \
         SET email_verification_enabled = $1, user_captcha_enabled = $2, \
             admin_email_verification_enabled = $3, admin_captcha_enabled = $4, \
             email_cooldown_seconds = $5, login_failure_threshold = $6, \
             login_lockout_minutes = $7, \
             revision = revision + 1, updated_by = $8, updated_at = now() \
         WHERE singleton = TRUE AND revision = $9 \
           AND (email_verification_enabled IS DISTINCT FROM $1 \
             OR user_captcha_enabled IS DISTINCT FROM $2 \
             OR admin_email_verification_enabled IS DISTINCT FROM $3 \
             OR admin_captcha_enabled IS DISTINCT FROM $4 \
             OR email_cooldown_seconds IS DISTINCT FROM $5 \
             OR login_failure_threshold IS DISTINCT FROM $6 \
             OR login_lockout_minutes IS DISTINCT FROM $7) \
         RETURNING email_verification_enabled, user_captcha_enabled, \
                   admin_email_verification_enabled, admin_captcha_enabled, \
                   email_cooldown_seconds, login_failure_threshold, login_lockout_minutes, \
                   revision, updated_by, updated_at",
    )
    .bind(input.email_verification_enabled)
    .bind(input.user_captcha_enabled)
    .bind(input.admin_email_verification_enabled)
    .bind(input.admin_captcha_enabled)
    .bind(input.email_cooldown_seconds)
    .bind(input.login_failure_threshold)
    .bind(input.login_lockout_minutes)
    .bind(actor_id)
    .bind(input.expected_revision)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .map(into_settings)
    .ok_or_else(|| AppError::Conflict("认证设置已变化，请刷新后重试".to_owned()))
}

pub(crate) async fn invalidate_open_email_challenges(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<(u64, u64)> {
    let registration = sqlx::query(
        "UPDATE email_verification_challenges SET consumed_at = now() \
         WHERE consumed_at IS NULL",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?
    .rows_affected();
    let login = sqlx::query(
        "UPDATE login_verification_challenges AS challenge SET consumed_at = now() \
         WHERE challenge.consumed_at IS NULL \
           AND EXISTS (SELECT 1 FROM accounts \
                       WHERE accounts.id = challenge.account_id \
                         AND accounts.role = 'user')",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?
    .rows_affected();
    Ok((registration, login))
}

pub(crate) async fn all_active_admins_have_verified_email(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT NOT EXISTS ( \
            SELECT 1 FROM accounts \
            WHERE role = 'admin' AND status = 'active' \
              AND (email IS NULL OR email_verified_at IS NULL) \
         )",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(error::storage)
}

pub(crate) async fn invalidate_open_user_captchas(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<u64> {
    sqlx::query(
        "UPDATE auth_captcha_challenges SET consumed_at = now() \
         WHERE consumed_at IS NULL AND purpose IN ('register', 'login', 'password_reset')",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)
    .map(|result| result.rows_affected())
}

pub(crate) async fn invalidate_open_admin_login_challenges(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<u64> {
    sqlx::query(
        "UPDATE login_verification_challenges AS challenge SET consumed_at = now() \
         WHERE challenge.consumed_at IS NULL \
           AND EXISTS (SELECT 1 FROM accounts \
                       WHERE accounts.id = challenge.account_id \
                         AND accounts.role = 'admin')",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)
    .map(|result| result.rows_affected())
}

pub(crate) async fn invalidate_open_admin_captchas(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<u64> {
    sqlx::query(
        "UPDATE auth_captcha_challenges SET consumed_at = now() \
         WHERE consumed_at IS NULL AND purpose = 'admin_login'",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)
    .map(|result| result.rows_affected())
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    settings: &AuthSettings,
    invalidated: InvalidatedChallenges,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'auth_settings.updated', \
                 'auth_settings', 'global', 'success', $3, \
                 jsonb_build_object( \
                    'email_verification_enabled', $4::boolean, \
                    'user_captcha_enabled', $5::boolean, \
                    'admin_email_verification_enabled', $6::boolean, \
                    'admin_captcha_enabled', $7::boolean, \
                    'email_cooldown_seconds', $8::integer, \
                    'login_failure_threshold', $9::integer, \
                    'login_lockout_minutes', $10::integer, \
                    'revision', $11::bigint, \
                    'registration_challenges_invalidated', $12::bigint, \
                    'user_login_challenges_invalidated', $13::bigint, \
                    'user_captcha_challenges_invalidated', $14::bigint, \
                    'admin_login_challenges_invalidated', $15::bigint, \
                    'admin_captcha_challenges_invalidated', $16::bigint \
                 ))",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(request_id)
    .bind(settings.email_verification_enabled)
    .bind(settings.user_captcha_enabled)
    .bind(settings.admin_email_verification_enabled)
    .bind(settings.admin_captcha_enabled)
    .bind(settings.email_cooldown_seconds)
    .bind(settings.login_failure_threshold)
    .bind(settings.login_lockout_minutes)
    .bind(settings.revision)
    .bind(i64::try_from(invalidated.registration).unwrap_or(i64::MAX))
    .bind(i64::try_from(invalidated.user_login).unwrap_or(i64::MAX))
    .bind(i64::try_from(invalidated.user_captcha).unwrap_or(i64::MAX))
    .bind(i64::try_from(invalidated.admin_login).unwrap_or(i64::MAX))
    .bind(i64::try_from(invalidated.admin_captcha).unwrap_or(i64::MAX))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存认证设置审计失败".to_owned()))?;
    Ok(())
}

fn into_settings(row: SettingsRow) -> AuthSettings {
    AuthSettings {
        email_verification_enabled: row.0,
        user_captcha_enabled: row.1,
        admin_email_verification_enabled: row.2,
        admin_captcha_enabled: row.3,
        email_cooldown_seconds: row.4,
        login_failure_threshold: row.5,
        login_lockout_minutes: row.6,
        revision: row.7,
        updated_by: row.8,
        updated_at: row.9,
    }
}

fn missing() -> AppError {
    AppError::Storage("认证设置单例缺失".to_owned())
}
