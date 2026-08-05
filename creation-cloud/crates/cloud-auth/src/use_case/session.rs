//! 由原始会话令牌构造业务身份及可公开期限元数据。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;

use crate::{
    repository,
    session::{AuthenticatedSession, SessionMetadata},
    token,
};

pub(crate) async fn authenticate(
    pool: &PgPool,
    raw_token: &str,
) -> AppResult<(AuthenticatedSession, SessionMetadata)> {
    let token_hash = token::hash(raw_token)?;
    let row = repository::session::authenticate(pool, &token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("会话无效或已过期".to_owned()))?;
    let metadata = SessionMetadata {
        email_verified: row.email_verified,
        session_kind: row.session_kind,
        device_name: row.device_name,
        last_login_ip: row.last_login_ip,
        user_agent: row.user_agent,
        client_version: row.client_version,
        device_fingerprint: row.device_fingerprint,
        created_at: row.created_at,
        last_seen_at: row.last_seen_at,
        idle_expires_at: row.idle_expires_at,
        absolute_expires_at: row.absolute_expires_at,
        revoked_at: row.revoked_at,
    };
    Ok((
        AuthenticatedSession {
            session_id: row.session_id,
            account_id: row.account_id,
            email: row.email.unwrap_or_default(),
            admin_login_name: row.admin_login_name,
            role: row.role,
            device_id: row.device_id,
            expires_at: row.idle_expires_at,
            csrf_token: token::csrf(raw_token),
        },
        metadata,
    ))
}
