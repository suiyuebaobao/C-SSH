//! 用一个 revision/CAS 更新四项认证开关与三项安全策略。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_auth::UpdateAuthSettings;
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct AuthSettingsForm {
    #[serde(default)]
    email_verification_enabled: bool,
    #[serde(default)]
    user_captcha_enabled: bool,
    #[serde(default)]
    admin_email_verification_enabled: bool,
    #[serde(default)]
    admin_captcha_enabled: bool,
    email_cooldown_seconds: i32,
    login_failure_threshold: i32,
    login_lockout_minutes: i32,
    expected_revision: i64,
    #[serde(rename = "csrf_token")]
    _csrf_token: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<AuthSettingsForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = UpdateAuthSettings {
        email_verification_enabled: form.email_verification_enabled,
        user_captcha_enabled: form.user_captcha_enabled,
        admin_email_verification_enabled: form.admin_email_verification_enabled,
        admin_captcha_enabled: form.admin_captcha_enabled,
        email_cooldown_seconds: form.email_cooldown_seconds,
        login_failure_threshold: form.login_failure_threshold,
        login_lockout_minutes: form.login_lockout_minutes,
        expected_revision: form.expected_revision,
    };
    match state.auth().update_auth_settings(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/settings", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{Request, header},
    };

    use super::*;

    #[tokio::test]
    async fn unchecked_checkbox_deserializes_to_false() {
        let request = Request::post("/admin/settings/auth-settings")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(
                "csrf_token=csrf-example&lang=en&expected_revision=7&email_cooldown_seconds=60&login_failure_threshold=5&login_lockout_minutes=30",
            ))
            .expect("form request should be valid");
        let Form(form) = Form::<AuthSettingsForm>::from_request(request, &())
            .await
            .expect("unchecked checkbox should be accepted");

        assert!(!form.email_verification_enabled);
        assert!(!form.user_captcha_enabled);
        assert!(!form.admin_email_verification_enabled);
        assert!(!form.admin_captcha_enabled);
        assert_eq!(form.email_cooldown_seconds, 60);
        assert_eq!(form.login_failure_threshold, 5);
        assert_eq!(form.login_lockout_minutes, 30);
        assert_eq!(form.expected_revision, 7);
        assert_eq!(form.lang.as_deref(), Some("en"));
    }

    #[tokio::test]
    async fn checked_checkbox_deserializes_to_true() {
        let request = Request::post("/admin/settings/auth-settings")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(
                "csrf_token=csrf-example&lang=zh-CN&expected_revision=8&email_verification_enabled=true&user_captcha_enabled=true&admin_email_verification_enabled=true&admin_captcha_enabled=true&email_cooldown_seconds=90&login_failure_threshold=6&login_lockout_minutes=45",
            ))
            .expect("form request should be valid");
        let Form(form) = Form::<AuthSettingsForm>::from_request(request, &())
            .await
            .expect("checked checkbox should be accepted");

        assert!(form.email_verification_enabled);
        assert!(form.user_captcha_enabled);
        assert!(form.admin_email_verification_enabled);
        assert!(form.admin_captcha_enabled);
        assert_eq!(form.email_cooldown_seconds, 90);
        assert_eq!(form.login_failure_threshold, 6);
        assert_eq!(form.login_lockout_minutes, 45);
        assert_eq!(form.expected_revision, 8);
        assert_eq!(form.lang.as_deref(), Some("zh-CN"));
    }
}
