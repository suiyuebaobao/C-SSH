//! 串行读取和更新全局认证设置，并在同一事务作废旧挑战、写语义审计。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::use_case::auth_settings::AuthSettings;

use super::error;

type SettingsRow = (bool, i64, Option<Uuid>, DateTime<Utc>);

pub(crate) async fn read(pool: &PgPool) -> AppResult<AuthSettings> {
    sqlx::query_as::<_, SettingsRow>(
        "SELECT email_verification_enabled, revision, updated_by, updated_at \
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
        "SELECT email_verification_enabled, revision, updated_by, updated_at \
         FROM auth_settings WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .map(into_settings)
    .ok_or_else(missing)
}

pub(crate) async fn email_verification_enabled(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT email_verification_enabled FROM auth_settings \
         WHERE singleton = TRUE FOR SHARE",
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .ok_or_else(missing)
}

pub(crate) async fn update(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    expected_revision: i64,
    enabled: bool,
) -> AppResult<AuthSettings> {
    sqlx::query_as::<_, SettingsRow>(
        "UPDATE auth_settings \
         SET email_verification_enabled = $1, revision = revision + 1, \
             updated_by = $2, updated_at = now() \
         WHERE singleton = TRUE AND revision = $3 \
           AND email_verification_enabled IS DISTINCT FROM $1 \
         RETURNING email_verification_enabled, revision, updated_by, updated_at",
    )
    .bind(enabled)
    .bind(actor_id)
    .bind(expected_revision)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .map(into_settings)
    .ok_or_else(|| AppError::Conflict("认证设置已变化，请刷新后重试".to_owned()))
}

pub(crate) async fn invalidate_open_challenges(
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
        "UPDATE login_verification_challenges SET consumed_at = now() \
         WHERE consumed_at IS NULL",
    )
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?
    .rows_affected();
    Ok((registration, login))
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    settings: &AuthSettings,
    invalidated: (u64, u64),
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'auth_settings.email_verification_updated', \
                 'auth_settings', 'global', 'success', $3, \
                 jsonb_build_object( \
                    'email_verification_enabled', $4::boolean, \
                    'revision', $5::bigint, \
                    'registration_challenges_invalidated', $6::bigint, \
                    'login_challenges_invalidated', $7::bigint \
                 ))",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(request_id)
    .bind(settings.email_verification_enabled)
    .bind(settings.revision)
    .bind(i64::try_from(invalidated.0).unwrap_or(i64::MAX))
    .bind(i64::try_from(invalidated.1).unwrap_or(i64::MAX))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存认证设置审计失败".to_owned()))?;
    Ok(())
}

fn into_settings(row: SettingsRow) -> AuthSettings {
    AuthSettings {
        email_verification_enabled: row.0,
        revision: row.1,
        updated_by: row.2,
        updated_at: row.3,
    }
}

fn missing() -> AppError {
    AppError::Storage("认证设置单例缺失".to_owned())
}
