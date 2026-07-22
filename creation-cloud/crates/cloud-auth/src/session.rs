//! 定义认证成功后的会话上下文及可安全返回给客户端的视图。

use chrono::{DateTime, Utc};
pub use cloud_domain::AuthenticatedSession;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Serialize)]
pub struct SessionView {
    pub account_id: Uuid,
    pub email: String,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub device_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub csrf_token: String,
}

impl From<&AuthenticatedSession> for SessionView {
    fn from(session: &AuthenticatedSession) -> Self {
        Self {
            account_id: session.account_id,
            email: session.email.clone(),
            admin_login_name: session.admin_login_name.clone(),
            role: session.role.clone(),
            device_id: session.device_id,
            expires_at: session.expires_at,
            csrf_token: session.csrf_token.clone(),
        }
    }
}

pub(crate) struct IssuedSession {
    pub raw_token: String,
    pub session: AuthenticatedSession,
}
