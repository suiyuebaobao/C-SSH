//! 原子更新密码、递增凭据版本、撤销旧会话并签发当前新会话。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_notification::{AccountNotificationEvent, record_account_event};
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) struct PasswordSnapshot {
    pub password_hash: String,
    pub credential_version: i64,
    pub session_kind: String,
    pub device_id: Option<Uuid>,
    pub device_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub device_fingerprint: Option<String>,
    pub absolute_expires_at: DateTime<Utc>,
}

type SnapshotRow = (
    String,
    i64,
    String,
    Option<Uuid>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    DateTime<Utc>,
);

pub(crate) const UPDATE_CREDENTIAL_SQL: &str = "UPDATE accounts SET password_hash = $2, credential_version = $3, \
     updated_at = now() WHERE id = $1";
pub(crate) const REVOKE_ACCOUNT_SESSIONS_SQL: &str = "UPDATE sessions SET revoked_at = now() \
     WHERE account_id = $1 AND revoked_at IS NULL";
pub(crate) const INSERT_ROTATED_SESSION_SQL: &str = "INSERT INTO sessions \
     (id, account_id, token_hash, credential_version, session_kind, device_id, \
      expires_at, absolute_expires_at, rotated_from_id, last_login_ip, user_agent, \
      client_version, device_fingerprint) \
     SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, \
            COALESCE($10::inet, source.last_login_ip), \
            COALESCE($11, source.user_agent), source.client_version, \
            source.device_fingerprint \
     FROM sessions AS source WHERE source.id = $9";

pub(crate) async fn current_snapshot(
    pool: &PgPool,
    account_id: Uuid,
    session_id: Uuid,
) -> AppResult<Option<PasswordSnapshot>> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT account.password_hash, account.credential_version, \
                session.session_kind, session.device_id, device.name, \
                host(session.last_login_ip), session.user_agent, session.client_version, \
                session.device_fingerprint, session.absolute_expires_at \
         FROM accounts AS account \
         JOIN sessions AS session ON session.account_id = account.id \
         LEFT JOIN devices AS device ON device.account_id = session.account_id \
            AND device.id = session.device_id \
         WHERE account.id = $1 AND session.id = $2 \
           AND account.status = 'active' \
           AND session.revoked_at IS NULL \
           AND session.credential_version = account.credential_version \
           AND session.expires_at > now() \
           AND session.absolute_expires_at > now()",
    )
    .bind(account_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|value| PasswordSnapshot {
            password_hash: value.0,
            credential_version: value.1,
            session_kind: value.2,
            device_id: value.3,
            device_name: value.4,
            last_login_ip: value.5,
            user_agent: value.6,
            client_version: value.7,
            device_fingerprint: value.8,
            absolute_expires_at: value.9,
        })
    })
    .map_err(error::storage)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_and_rotate(
    pool: &PgPool,
    account_id: Uuid,
    current_session_id: Uuid,
    expected_password_hash: &str,
    expected_credential_version: i64,
    new_password_hash: &str,
    new_session_id: Uuid,
    token_hash: &[u8],
    session_kind: &str,
    device_id: Option<Uuid>,
    idle_expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    request_metadata: &crate::TrustedRequestMetadata,
) -> AppResult<i64> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let current = sqlx::query_as::<_, (String, i64)>(
        "SELECT password_hash, credential_version FROM accounts \
         WHERE id = $1 AND status = 'active' FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?
    .ok_or_else(account_unavailable)?;
    let current_session_exists = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sessions WHERE id = $1 AND account_id = $2 \
         AND revoked_at IS NULL AND credential_version = $3 \
         AND expires_at > now() AND absolute_expires_at > now() FOR UPDATE",
    )
    .bind(current_session_id)
    .bind(account_id)
    .bind(expected_credential_version)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?
    .is_some();
    if current.0 != expected_password_hash
        || current.1 != expected_credential_version
        || !current_session_exists
    {
        return Err(account_unavailable());
    }
    let new_version = current
        .1
        .checked_add(1)
        .ok_or_else(|| AppError::Internal("凭据版本已超出支持范围".to_owned()))?;
    sqlx::query(UPDATE_CREDENTIAL_SQL)
        .bind(account_id)
        .bind(new_password_hash)
        .bind(new_version)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    sqlx::query(REVOKE_ACCOUNT_SESSIONS_SQL)
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    sqlx::query(INSERT_ROTATED_SESSION_SQL)
        .bind(new_session_id)
        .bind(account_id)
        .bind(token_hash)
        .bind(new_version)
        .bind(session_kind)
        .bind(device_id)
        .bind(idle_expires_at)
        .bind(absolute_expires_at)
        .bind(current_session_id)
        .bind(&request_metadata.last_login_ip)
        .bind(&request_metadata.user_agent)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    record_account_event(
        &mut transaction,
        account_id,
        AccountNotificationEvent::PasswordChanged,
    )
    .await?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(new_version)
}

fn account_unavailable() -> AppError {
    AppError::Unauthorized("账号或当前会话不可用".to_owned())
}
