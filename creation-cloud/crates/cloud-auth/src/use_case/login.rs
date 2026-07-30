//! 校验账号密码；普通用户进入邮箱挑战，管理员直接创建安全会话。

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    mailer::{VerificationMailer, VerificationPurpose},
    password, repository,
    repository::login::LoginAccount,
    session::{AuthenticatedSession, IssuedSession},
    token, validation, verification,
};

const INVALID_CREDENTIALS: &str = "账号或密码错误";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    pub identifier: String,
    pub password: String,
    #[serde(default)]
    pub captcha_id: Option<Uuid>,
    #[serde(default)]
    pub captcha_code: Option<String>,
}

pub enum LoginOutcome {
    VerificationRequired(LoginVerificationRequired),
    Session(IssuedSession),
}

#[derive(Clone, Serialize)]
pub struct LoginVerificationRequired {
    pub status: &'static str,
    pub challenge_id: Uuid,
    pub expires_at: chrono::DateTime<Utc>,
}

impl Login {
    pub(crate) fn validate(&self) -> AppResult<validation::LoginIdentifier> {
        let identifier = validation::login_identifier(&self.identifier)?;
        validation::login_password(&self.password)?;
        Ok(identifier)
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session_ttl: Duration,
    verification_key: &[u8],
    captcha_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: Login,
) -> AppResult<LoginOutcome> {
    let identifier = match command.validate() {
        Ok(identifier) => identifier,
        Err(AppError::Validation(_)) => return reject_after_dummy_hash().await,
        Err(error) => return Err(error),
    };
    let account = if identifier.is_admin_login_name() {
        repository::login::find_admin_by_login_name(pool, &identifier.value).await?
    } else {
        repository::login::find_by_email(pool, &identifier.value).await?
    };
    let Some(account) = account else {
        // 未命中账号时仍执行一次 Argon2id，降低账号枚举的时序差异。
        return reject_after_dummy_hash().await;
    };
    // Argon2id 验证可能耗时，必须在取得账号行锁之前完成。
    let password_valid =
        password::verify(command.password.clone(), account.password_hash.clone()).await?;
    if !password_valid
        || account.status != "active"
        || (account.role != "admin" && account.email_verified_at.is_none())
    {
        if account.role == "admin" {
            consume_admin_captcha_attempt(
                pool,
                captcha_key,
                command.captcha_id,
                command.captcha_code.as_deref(),
            )
            .await?;
            return Err(invalid_admin_credentials());
        }
        return Err(invalid_credentials());
    }

    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let Some(current_account) = repository::login::lock_by_id(&mut transaction, account.id).await?
    else {
        return reject_stale_snapshot(transaction).await;
    };
    if !snapshot_allows_session(&account, &current_account, &identifier) {
        return reject_stale_snapshot(transaction).await;
    }
    if current_account.role == "admin" {
        let challenge_id = command.captcha_id.ok_or_else(invalid_admin_credentials)?;
        let supplied_code = command.captcha_code.as_deref().unwrap_or_default();
        let digest_code = if valid_captcha_code(supplied_code) {
            supplied_code
        } else {
            "000000"
        };
        let supplied_digest = verification::captcha_digest(captcha_key, challenge_id, digest_code);
        let valid =
            repository::captcha::consume(&mut transaction, challenge_id, &supplied_digest).await?;
        if !valid || !valid_captcha_code(supplied_code) {
            transaction
                .commit()
                .await
                .map_err(repository::error::storage)?;
            return Err(invalid_admin_credentials());
        }
    }
    let email_verification_enabled = if current_account.role == "admin" {
        false
    } else {
        repository::settings::email_verification_enabled(&mut transaction).await?
    };
    if requires_login_verification(&current_account, email_verification_enabled) {
        let email = current_account
            .email
            .as_deref()
            .filter(|_| current_account.email_verified_at.is_some())
            .ok_or_else(invalid_credentials)?;
        if verification_key.len() < 32 {
            return Err(AppError::Unavailable(
                "登录验证码密钥尚未安全配置".to_owned(),
            ));
        }
        let challenge_id = Uuid::now_v7();
        let code = verification::issue_code();
        let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
        let code_digest = verification::login_digest(
            verification_key,
            challenge_id,
            current_account.id,
            email,
            current_account.credential_version,
            &code,
        );
        repository::login_verification::replace_open(
            &mut transaction,
            repository::login_verification::NewLoginChallenge {
                id: challenge_id,
                account_id: current_account.id,
                email,
                credential_version: current_account.credential_version,
                code_digest: &code_digest,
                expires_at,
            },
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(repository::error::storage)?;
        if let Err(error) = mailer
            .send_verification(email, &code, VerificationPurpose::Login)
            .await
        {
            repository::login_verification::cancel_unsent(pool, challenge_id).await?;
            return Err(error);
        }
        repository::login_verification::mark_sent(pool, challenge_id).await?;
        return Ok(LoginOutcome::VerificationRequired(
            LoginVerificationRequired {
                status: "verification_required",
                challenge_id,
                expires_at,
            },
        ));
    }

    let session_id = Uuid::now_v7();
    let expires_at = session_expiry(session_ttl)?;
    let (raw_token, token_hash) = token::issue();
    repository::login::insert_session(
        &mut transaction,
        session_id,
        current_account.id,
        &token_hash,
        expires_at,
        current_account.credential_version,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;

    Ok(LoginOutcome::Session(IssuedSession {
        session: AuthenticatedSession {
            session_id,
            account_id: current_account.id,
            email: current_account.email.clone().unwrap_or_default(),
            admin_login_name: current_account.admin_login_name,
            role: current_account.role,
            device_id: None,
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        metadata: crate::session::SessionMetadata::unbound(
            expires_at,
            current_account.email_verified_at.is_some(),
        ),
        raw_token,
    }))
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized(INVALID_CREDENTIALS.to_owned())
}

fn invalid_admin_credentials() -> AppError {
    AppError::Unauthorized("管理员账号、密码或图形验证码错误".to_owned())
}

async fn consume_admin_captcha_attempt(
    pool: &PgPool,
    captcha_key: &[u8],
    challenge_id: Option<Uuid>,
    supplied_code: Option<&str>,
) -> AppResult<()> {
    let Some(challenge_id) = challenge_id else {
        return Ok(());
    };
    let supplied_code = supplied_code.unwrap_or_default();
    let digest_code = if valid_captcha_code(supplied_code) {
        supplied_code
    } else {
        "000000"
    };
    let supplied_digest = verification::captcha_digest(captcha_key, challenge_id, digest_code);
    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let _ = repository::captcha::consume(&mut transaction, challenge_id, &supplied_digest).await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;
    Ok(())
}

async fn reject_after_dummy_hash() -> AppResult<LoginOutcome> {
    // 非法或不存在的标识符统一哈希固定有界值，既保持时序成本又拒绝超长密码放大 CPU。
    let _ = password::hash("bounded-dummy-password".to_owned()).await?;
    Err(invalid_credentials())
}

pub(crate) fn snapshot_allows_session(
    initial: &LoginAccount,
    current: &LoginAccount,
    identifier: &validation::LoginIdentifier,
) -> bool {
    same_account_snapshot(initial, current) && identifier_matches_current(identifier, current)
}

fn same_account_snapshot(initial: &LoginAccount, current: &LoginAccount) -> bool {
    initial.id == current.id
        && initial.email == current.email
        && initial.email_verified_at == current.email_verified_at
        && initial.admin_login_name == current.admin_login_name
        && initial.password_hash == current.password_hash
        && initial.role == current.role
        && initial.status == current.status
        && initial.credential_version == current.credential_version
}

fn identifier_matches_current(
    identifier: &validation::LoginIdentifier,
    account: &LoginAccount,
) -> bool {
    if account.status != "active" {
        return false;
    }
    if identifier.is_admin_login_name() {
        account.role == "admin"
            && account.admin_login_name.as_deref() == Some(identifier.value.as_str())
    } else {
        account.email.as_deref() == Some(identifier.value.as_str())
    }
}

async fn reject_stale_snapshot(transaction: Transaction<'_, Postgres>) -> AppResult<LoginOutcome> {
    // 身份漂移必须始终表现为同一模糊认证失败；回滚异常也不得形成枚举信号。
    let _ = transaction.rollback().await;
    Err(invalid_credentials())
}

pub(crate) fn requires_login_verification(
    account: &LoginAccount,
    email_verification_enabled: bool,
) -> bool {
    account.role != "admin" && email_verification_enabled
}

pub(crate) fn session_expiry(session_ttl: Duration) -> AppResult<chrono::DateTime<Utc>> {
    Ok(Utc::now()
        + chrono::Duration::from_std(session_ttl)
            .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?)
}

fn valid_captcha_code(code: &str) -> bool {
    code.len() == crate::captcha::CODE_LENGTH && code.bytes().all(|byte| byte.is_ascii_digit())
}
