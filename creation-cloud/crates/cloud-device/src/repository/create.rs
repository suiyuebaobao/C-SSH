//! 在账号、当前会话和设备锁内完成设备绑定与长期会话令牌轮换。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    MAX_ACTIVE_DEVICES_PER_ACCOUNT,
    model::{CreateDeviceOutcome, Device, DeviceRow, DeviceSessionView},
    session,
};

use super::error;

pub(crate) const LOCK_ACCOUNT_SQL: &str = "SELECT id, email, email_verified_at, admin_login_name, role, credential_version \
     FROM accounts WHERE id = $1 AND status = 'active' \
       AND (role = 'admin' OR email_verified_at IS NOT NULL) FOR UPDATE";
pub(crate) const LOCK_SESSION_SQL: &str = "SELECT device_id, session_kind, absolute_expires_at, \
            host(last_login_ip), user_agent, client_version, device_fingerprint FROM sessions \
     WHERE id = $1 AND account_id = $2 AND credential_version = $3 \
       AND revoked_at IS NULL AND expires_at > now() \
       AND absolute_expires_at > now() FOR UPDATE";
pub(crate) const REVOKE_CURRENT_SESSION_SQL: &str = "UPDATE sessions SET revoked_at = now() \
     WHERE id = $1 AND account_id = $2 AND revoked_at IS NULL";
pub(crate) const INSERT_DEVICE_SESSION_SQL: &str = "INSERT INTO sessions \
     (id, account_id, token_hash, credential_version, session_kind, device_id, \
      expires_at, absolute_expires_at, rotated_from_id, last_login_ip, user_agent, \
      client_version, device_fingerprint) \
     VALUES ($1, $2, $3, $4, 'device', $5, $6, $7, $8, $9::inet, $10, $11, $12)";
pub(crate) const COUNT_ACTIVE_DEVICES_SQL: &str =
    "SELECT count(*)::BIGINT FROM devices WHERE account_id = $1 AND revoked_at IS NULL";

type AccountRow = (
    Uuid,
    Option<String>,
    Option<DateTime<Utc>>,
    Option<String>,
    String,
    i64,
);
type CurrentSessionRow = (
    Option<Uuid>,
    String,
    DateTime<Utc>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[allow(clippy::too_many_arguments)]
pub(crate) async fn bind(
    pool: &PgPool,
    current_session: &AuthenticatedSession,
    name: &str,
    platform: &str,
    public_id: &str,
    last_login_ip: Option<&str>,
    user_agent: Option<&str>,
    client_version: Option<&str>,
    device_fingerprint: Option<&str>,
) -> AppResult<CreateDeviceOutcome> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let account = sqlx::query_as::<_, AccountRow>(LOCK_ACCOUNT_SQL)
        .bind(current_session.account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .ok_or_else(|| AppError::Unauthorized("账号不可用".to_owned()))?;
    let active_session = sqlx::query_as::<_, CurrentSessionRow>(LOCK_SESSION_SQL)
        .bind(current_session.session_id)
        .bind(current_session.account_id)
        .bind(account.5)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .ok_or_else(|| AppError::Unauthorized("会话无效或已过期".to_owned()))?;

    let existing = sqlx::query_as::<_, DeviceRow>(
        "SELECT id, account_id, name, platform, public_id, last_seen_at, revoked_at, \
                created_at, updated_at FROM devices \
         WHERE account_id = $1 AND public_id = $2 FOR UPDATE",
    )
    .bind(current_session.account_id)
    .bind(public_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;

    let (device, created) = match existing {
        Some(row) => {
            if row.6.is_some() {
                return Err(AppError::Conflict(
                    "已撤销的设备标识不能重新登记".to_owned(),
                ));
            }
            if row.3 != platform {
                return Err(AppError::Conflict("设备平台与既有登记不一致".to_owned()));
            }
            if active_session.0.is_some_and(|device_id| device_id != row.0) {
                return Err(AppError::Conflict("当前会话已绑定其它设备".to_owned()));
            }
            let row = sqlx::query_as::<_, DeviceRow>(
                "UPDATE devices SET name = $3, last_seen_at = now(), updated_at = now() \
                 WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL \
                 RETURNING id, account_id, name, platform, public_id, last_seen_at, \
                           revoked_at, created_at, updated_at",
            )
            .bind(current_session.account_id)
            .bind(row.0)
            .bind(name)
            .fetch_one(&mut *transaction)
            .await
            .map_err(error::storage)?;
            (Device::from_row(row), false)
        }
        None => {
            if active_session.0.is_some() {
                return Err(AppError::Conflict("当前会话已绑定其它设备".to_owned()));
            }
            let active_device_count = sqlx::query_scalar::<_, i64>(COUNT_ACTIVE_DEVICES_SQL)
                .bind(current_session.account_id)
                .fetch_one(&mut *transaction)
                .await
                .map_err(error::storage)?;
            if active_device_count >= MAX_ACTIVE_DEVICES_PER_ACCOUNT {
                return Err(AppError::Conflict(
                    "账号的活动设备数量已达到上限".to_owned(),
                ));
            }
            let row = sqlx::query_as::<_, DeviceRow>(
                "INSERT INTO devices \
                 (id, account_id, name, platform, public_id, last_seen_at) \
                 VALUES ($1, $2, $3, $4, $5, now()) \
                 RETURNING id, account_id, name, platform, public_id, last_seen_at, \
                           revoked_at, created_at, updated_at",
            )
            .bind(Uuid::now_v7())
            .bind(current_session.account_id)
            .bind(name)
            .bind(platform)
            .bind(public_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(error::create)?;
            (Device::from_row(row), true)
        }
    };

    let now = Utc::now();
    let absolute_expires_at = if active_session.1 == "device" {
        active_session.2
    } else {
        now + chrono::Duration::days(365 * 2)
    };
    let idle_expires_at = std::cmp::min(now + chrono::Duration::days(90), absolute_expires_at);
    if idle_expires_at <= now {
        return Err(AppError::Unauthorized(
            "长期设备会话绝对期限已到期".to_owned(),
        ));
    }
    let new_session_id = Uuid::now_v7();
    let session_last_login_ip = last_login_ip
        .map(str::to_owned)
        .or_else(|| active_session.3.clone());
    let session_user_agent = user_agent
        .map(str::to_owned)
        .or_else(|| active_session.4.clone());
    let session_client_version = client_version
        .map(str::to_owned)
        .or_else(|| active_session.5.clone());
    let session_device_fingerprint = device_fingerprint
        .map(str::to_owned)
        .or_else(|| active_session.6.clone());
    let (raw_token, token_hash) = session::issue();
    sqlx::query(REVOKE_CURRENT_SESSION_SQL)
        .bind(current_session.session_id)
        .bind(current_session.account_id)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    sqlx::query(INSERT_DEVICE_SESSION_SQL)
        .bind(new_session_id)
        .bind(current_session.account_id)
        .bind(token_hash)
        .bind(account.5)
        .bind(device.id)
        .bind(idle_expires_at)
        .bind(absolute_expires_at)
        .bind(current_session.session_id)
        .bind(&session_last_login_ip)
        .bind(&session_user_agent)
        .bind(&session_client_version)
        .bind(&session_device_fingerprint)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;

    Ok(CreateDeviceOutcome {
        session: DeviceSessionView {
            session_id: new_session_id,
            account_id: account.0,
            email: account.1,
            email_verified: account.2.is_some(),
            admin_login_name: account.3,
            role: account.4,
            status: "online".to_owned(),
            is_current: true,
            device_id: device.id,
            device_name: Some(device.name.clone()),
            last_login_ip: session_last_login_ip,
            user_agent: session_user_agent,
            client_version: session_client_version,
            device_fingerprint: session_device_fingerprint,
            session_kind: "device".to_owned(),
            created_at: now,
            last_seen_at: now,
            idle_expires_at,
            absolute_expires_at,
            revoked_at: None,
            csrf_token: session::csrf(&raw_token),
        },
        raw_token,
        device,
        created,
    })
}
