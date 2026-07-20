//! 定义由认证层验证、供各业务域只读消费的当前会话身份。

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthenticatedSession {
    pub account_id: Uuid,
    pub email: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
    pub csrf_token: String,
    pub session_id: Uuid,
}
