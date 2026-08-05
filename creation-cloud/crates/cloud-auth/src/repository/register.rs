//! 按全局设置原子创建待验证账号或直接激活账号并签发会话。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use super::{captcha, error, login, settings, verification};

pub(crate) struct RegistrationAccount<'a> {
    pub account_id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub locale: &'a str,
    pub challenge_id: Uuid,
    pub code_digest: &'a [u8],
    pub expires_at: DateTime<Utc>,
    pub session_id: Uuid,
    pub session_token_hash: &'a [u8],
    pub session_expires_at: DateTime<Utc>,
    pub captcha_id: Uuid,
    pub captcha_digest: &'a [u8; 32],
    pub captcha_code_valid: bool,
    pub request_metadata: &'a crate::TrustedRequestMetadata,
}

pub(crate) enum RegistrationPreparation {
    Verification { should_send: bool },
    Session { account_id: Uuid },
    Accepted,
}

pub(crate) async fn prepare(
    pool: &PgPool,
    verification_available: bool,
    account: RegistrationAccount<'_>,
) -> AppResult<RegistrationPreparation> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let auth_settings = settings::lock(&mut transaction).await?;
    if auth_settings.user_captcha_enabled {
        let valid = captcha::consume(
            &mut transaction,
            account.captcha_id,
            crate::captcha::CaptchaPurpose::Register,
            account.captcha_digest,
        )
        .await?;
        if !valid || !account.captcha_code_valid {
            transaction.commit().await.map_err(error::storage)?;
            return Err(AppError::Unauthorized("图形验证码错误或已失效".to_owned()));
        }
    }
    let verification_enabled = auth_settings.email_verification_enabled;
    if verification_enabled && !verification_available {
        return Err(AppError::Unavailable("邮箱验证密钥尚未安全配置".to_owned()));
    }
    verification::lock_email(&mut transaction, account.email).await?;
    if let Some(existing) = verification::find_account(&mut transaction, account.email).await? {
        if verification_enabled {
            let should_send = existing.status == "pending_verification"
                && existing.email_verified_at.is_none()
                && verification::replace_if_cooled_down(
                    &mut transaction,
                    existing.id,
                    account.email,
                    account.challenge_id,
                    account.code_digest,
                    account.expires_at,
                    auth_settings.email_cooldown_seconds,
                )
                .await?;
            transaction.commit().await.map_err(error::storage)?;
            return Ok(if existing.status == "pending_verification" {
                RegistrationPreparation::Verification { should_send }
            } else {
                RegistrationPreparation::Accepted
            });
        }
        if existing.status == "pending_verification" && existing.email_verified_at.is_none() {
            let credential_version = activate_pending(
                &mut transaction,
                existing.id,
                account.password_hash,
                account.display_name,
                account.locale,
            )
            .await?;
            verification::consume_open(&mut transaction, existing.id).await?;
            login::insert_session(
                &mut transaction,
                account.session_id,
                existing.id,
                account.session_token_hash,
                account.session_expires_at,
                credential_version,
                account.request_metadata,
            )
            .await?;
            transaction.commit().await.map_err(error::storage)?;
            return Ok(RegistrationPreparation::Session {
                account_id: existing.id,
            });
        }
        transaction.commit().await.map_err(error::storage)?;
        return Ok(RegistrationPreparation::Accepted);
    }

    let status = if verification_enabled {
        "pending_verification"
    } else {
        "active"
    };
    sqlx::query(
        "INSERT INTO accounts \
         (id, email, password_hash, status, email_verified_at) \
         VALUES ($1, $2, $3, $4, CASE WHEN $4 = 'active' THEN now() ELSE NULL END)",
    )
    .bind(account.account_id)
    .bind(account.email)
    .bind(account.password_hash)
    .bind(status)
    .execute(&mut *transaction)
    .await
    .map_err(error::create_account)?;
    sqlx::query(
        "INSERT INTO user_profiles (account_id, display_name, locale) \
         VALUES ($1, $2, $3)",
    )
    .bind(account.account_id)
    .bind(account.display_name)
    .bind(account.locale)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if verification_enabled {
        verification::insert(
            &mut transaction,
            account.account_id,
            account.email,
            account.challenge_id,
            account.code_digest,
            account.expires_at,
        )
        .await?;
    } else {
        login::insert_session(
            &mut transaction,
            account.session_id,
            account.account_id,
            account.session_token_hash,
            account.session_expires_at,
            1,
            account.request_metadata,
        )
        .await?;
    }
    transaction.commit().await.map_err(error::storage)?;
    Ok(if verification_enabled {
        RegistrationPreparation::Verification { should_send: true }
    } else {
        RegistrationPreparation::Session {
            account_id: account.account_id,
        }
    })
}

async fn activate_pending(
    transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
    account_id: Uuid,
    password_hash: &str,
    display_name: &str,
    locale: &str,
) -> AppResult<i64> {
    let credential_version = sqlx::query_scalar::<_, i64>(
        "UPDATE accounts \
         SET password_hash = $2, status = 'active', email_verified_at = now(), \
             credential_version = credential_version + 1, updated_at = now() \
         WHERE id = $1 AND status = 'pending_verification' \
         RETURNING credential_version",
    )
    .bind(account_id)
    .bind(password_hash)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .ok_or_else(|| AppError::Conflict("注册账号状态已变化，请重新登录".to_owned()))?;
    sqlx::query(
        "UPDATE user_profiles SET display_name = $2, locale = $3, updated_at = now() \
         WHERE account_id = $1",
    )
    .bind(account_id)
    .bind(display_name)
    .bind(locale)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(credential_version)
}
