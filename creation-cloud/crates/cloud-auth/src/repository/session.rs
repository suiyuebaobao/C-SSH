//! 查询有效会话并在同一语句中推进最后使用时间。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) struct SessionRow {
    pub session_id: Uuid,
    pub account_id: Uuid,
    pub email: String,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub device_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

pub(crate) async fn authenticate(
    pool: &PgPool,
    token_hash: &[u8],
) -> AppResult<Option<SessionRow>> {
    sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            String,
            Option<Uuid>,
            DateTime<Utc>,
        ),
    >(
        "WITH active_session AS (\
            UPDATE sessions SET last_seen_at = now() \
            WHERE token_hash = $1 AND expires_at > now() \
            RETURNING id, account_id, device_id, expires_at\
         ) \
         SELECT active_session.id AS session_id, accounts.id AS account_id, \
                accounts.email, accounts.admin_login_name, accounts.role, \
                active_session.device_id, active_session.expires_at \
         FROM active_session \
         JOIN accounts ON accounts.id = active_session.account_id \
         LEFT JOIN devices ON devices.account_id = active_session.account_id \
             AND devices.active_session_reference_id = active_session.device_id \
         WHERE accounts.status = 'active' \
           AND (active_session.device_id IS NULL OR devices.id IS NOT NULL)",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|value| SessionRow {
            session_id: value.0,
            account_id: value.1,
            email: value.2,
            admin_login_name: value.3,
            role: value.4,
            device_id: value.5,
            expires_at: value.6,
        })
    })
    .map_err(error::storage)
}
