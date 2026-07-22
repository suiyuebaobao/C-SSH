//! 定义只能由已认证管理员会话派生的管理操作身份。
//! 业务用例只接收该收敛类型，避免调用方直接传入账号标识绕过角色校验。

use uuid::Uuid;

use crate::{AppError, AppResult, AuthenticatedSession};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdminActor {
    account_id: Uuid,
}

impl AdminActor {
    pub fn from_session(session: &AuthenticatedSession) -> AppResult<Self> {
        if session.role != "admin" {
            return Err(AppError::Forbidden("需要管理员权限".to_owned()));
        }
        if session.account_id.is_nil() {
            return Err(AppError::Unauthorized("管理员会话身份无效".to_owned()));
        }
        Ok(Self {
            account_id: session.account_id,
        })
    }

    #[must_use]
    pub const fn account_id(&self) -> Uuid {
        self.account_id
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;

    fn session(role: &str) -> AuthenticatedSession {
        AuthenticatedSession {
            account_id: Uuid::now_v7(),
            email: "admin@example.com".to_owned(),
            admin_login_name: None,
            role: role.to_owned(),
            device_id: None,
            expires_at: Utc::now() + Duration::minutes(10),
            csrf_token: "csrf-example".to_owned(),
            session_id: Uuid::now_v7(),
        }
    }

    #[test]
    fn accepts_only_admin_role() {
        let admin = session("admin");
        let actor = AdminActor::from_session(&admin).expect("管理员会话应可派生管理身份");
        assert_eq!(actor.account_id(), admin.account_id);

        assert!(matches!(
            AdminActor::from_session(&session("user")),
            Err(AppError::Forbidden(_))
        ));
    }
}
