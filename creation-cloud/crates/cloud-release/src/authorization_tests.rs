//! 验证版本域 use-case 只接受由管理员会话派生的能力身份。

use chrono::{Duration, Utc};
use cloud_domain::{AdminActor, AuthenticatedSession};
use uuid::Uuid;

#[test]
fn accepts_admin_actor() {
    let session = AuthenticatedSession {
        account_id: Uuid::now_v7(),
        email: "admin@example.com".into(),
        admin_login_name: Some("ops-admin".into()),
        role: "admin".into(),
        device_id: None,
        expires_at: Utc::now() + Duration::minutes(5),
        csrf_token: "csrf-example".into(),
        session_id: Uuid::now_v7(),
    };
    let actor = AdminActor::from_session(&session).expect("管理员会话应可派生管理身份");
    let account_id = super::authorization::require(&actor).expect("管理员身份应通过用例校验");
    assert_eq!(account_id, session.account_id);
}
