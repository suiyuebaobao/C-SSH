//! 校验账号版本、设备状态和双期限，并续期长期设备会话的闲置期限。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) struct SessionRow {
    pub session_id: Uuid,
    pub account_id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub device_id: Option<Uuid>,
    pub session_kind: String,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
}

type SessionTuple = (
    Uuid,
    Uuid,
    Option<String>,
    bool,
    Option<String>,
    String,
    Option<Uuid>,
    String,
    DateTime<Utc>,
    DateTime<Utc>,
);

pub(crate) const AUTHENTICATE_SESSION_SQL: &str = "WITH active_session AS (\
        UPDATE sessions SET \
            last_seen_at = now(), \
            expires_at = CASE \
                WHEN session_kind = 'device' \
                THEN LEAST(now() + interval '90 days', absolute_expires_at) \
                ELSE expires_at \
            END \
        WHERE token_hash = $1 \
          AND revoked_at IS NULL \
          AND expires_at > now() \
          AND absolute_expires_at > now() \
        RETURNING id, account_id, credential_version, device_id, session_kind, \
                  expires_at, absolute_expires_at\
     ) \
     SELECT active_session.id, accounts.id, accounts.email, \
            accounts.email_verified_at IS NOT NULL, accounts.admin_login_name, \
            accounts.role, active_session.device_id, active_session.session_kind, \
            active_session.expires_at, active_session.absolute_expires_at \
     FROM active_session \
     JOIN accounts ON accounts.id = active_session.account_id \
         AND accounts.credential_version = active_session.credential_version \
     LEFT JOIN devices ON devices.account_id = active_session.account_id \
         AND devices.active_session_reference_id = active_session.device_id \
     WHERE accounts.status = 'active' \
       AND (accounts.role = 'admin' OR accounts.email_verified_at IS NOT NULL) \
       AND (active_session.device_id IS NULL OR devices.id IS NOT NULL)";

pub(crate) async fn authenticate(
    pool: &PgPool,
    token_hash: &[u8],
) -> AppResult<Option<SessionRow>> {
    sqlx::query_as::<_, SessionTuple>(AUTHENTICATE_SESSION_SQL)
        .bind(token_hash)
        .fetch_optional(pool)
        .await
        .map(|row| {
            row.map(|value| SessionRow {
                session_id: value.0,
                account_id: value.1,
                email: value.2,
                email_verified: value.3,
                admin_login_name: value.4,
                role: value.5,
                device_id: value.6,
                session_kind: value.7,
                idle_expires_at: value.8,
                absolute_expires_at: value.9,
            })
        })
        .map_err(error::storage)
}
