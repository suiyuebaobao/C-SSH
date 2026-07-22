//! 将 Cookie 原始令牌哈希后查询会话，并构造跨业务鉴权上下文。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;

use crate::{repository, session::AuthenticatedSession, token};

pub(crate) async fn authenticate(
    pool: &PgPool,
    raw_token: &str,
) -> AppResult<AuthenticatedSession> {
    let token_hash = token::hash(raw_token)?;
    let row = repository::session::authenticate(pool, &token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("会话无效或已过期".to_owned()))?;
    Ok(AuthenticatedSession {
        session_id: row.session_id,
        account_id: row.account_id,
        email: row.email,
        admin_login_name: row.admin_login_name,
        role: row.role,
        device_id: row.device_id,
        expires_at: row.expires_at,
        csrf_token: token::csrf(raw_token),
    })
}
