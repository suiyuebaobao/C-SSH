//! 以模糊错误校验账号密码并创建新的安全会话。

use std::time::Duration;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    password, repository,
    repository::login::LoginAccount,
    session::{AuthenticatedSession, IssuedSession},
    token, validation,
};

const INVALID_CREDENTIALS: &str = "账号或密码错误";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    pub identifier: String,
    pub password: String,
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
    command: Login,
) -> AppResult<IssuedSession> {
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
    let password_valid = password::verify(command.password, account.password_hash.clone()).await?;
    if !password_valid || account.status != "active" {
        return Err(invalid_credentials());
    }

    let session_id = Uuid::now_v7();
    let expires_at = Utc::now()
        + chrono::Duration::from_std(session_ttl)
            .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?;
    let (raw_token, token_hash) = token::issue();
    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let Some(current_account) = repository::login::lock_by_id(&mut transaction, account.id).await?
    else {
        return reject_stale_snapshot(transaction).await;
    };
    if !snapshot_allows_session(&account, &current_account, &identifier) {
        return reject_stale_snapshot(transaction).await;
    }
    repository::login::insert_session(
        &mut transaction,
        session_id,
        current_account.id,
        &token_hash,
        expires_at,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;

    Ok(IssuedSession {
        session: AuthenticatedSession {
            session_id,
            account_id: current_account.id,
            email: current_account.email,
            admin_login_name: current_account.admin_login_name,
            role: current_account.role,
            device_id: None,
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        raw_token,
    })
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized(INVALID_CREDENTIALS.to_owned())
}

async fn reject_after_dummy_hash() -> AppResult<IssuedSession> {
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
        && initial.admin_login_name == current.admin_login_name
        && initial.password_hash == current.password_hash
        && initial.role == current.role
        && initial.status == current.status
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
        account.email == identifier.value
    }
}

async fn reject_stale_snapshot(transaction: Transaction<'_, Postgres>) -> AppResult<IssuedSession> {
    // 身份漂移必须始终表现为同一模糊认证失败；回滚异常也不得形成枚举信号。
    let _ = transaction.rollback().await;
    Err(invalid_credentials())
}
