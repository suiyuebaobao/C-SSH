//! 展示管理后台的全局系统设置。
//! 页面只负责读取认证设置并渲染紧凑表单，写入继续进入认证域 use-case。

pub(crate) mod auth_settings;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SettingsQuery {
    lang: Option<String>,
}

struct AuthSettingsView {
    email_verification_enabled: bool,
    user_captcha_enabled: bool,
    admin_email_verification_enabled: bool,
    admin_captcha_enabled: bool,
    email_cooldown_seconds: i32,
    login_failure_threshold: i32,
    login_lockout_minutes: i32,
    revision: i64,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin-settings.html")]
struct SettingsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    settings: Option<AuthSettingsView>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SettingsQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let (settings, load_error) = match state.auth().auth_settings(&actor).await {
        Ok(settings) => (Some(AuthSettingsView::from(settings)), None),
        Err(_) => (
            None,
            Some(if locale == Locale::En {
                "System settings are temporarily unavailable.".to_owned()
            } else {
                "系统设置暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminSettings, locale, &session);
    shared::render(&SettingsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        settings,
        load_error,
    })
}

impl From<cloud_auth::AuthSettings> for AuthSettingsView {
    fn from(value: cloud_auth::AuthSettings) -> Self {
        Self {
            email_verification_enabled: value.email_verification_enabled,
            user_captcha_enabled: value.user_captcha_enabled,
            admin_email_verification_enabled: value.admin_email_verification_enabled,
            admin_captcha_enabled: value.admin_captcha_enabled,
            email_cooldown_seconds: value.email_cooldown_seconds,
            login_failure_threshold: value.login_failure_threshold,
            login_lockout_minutes: value.login_lockout_minutes,
            revision: value.revision,
            updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::*;

    fn render_settings(
        locale: Locale,
        user_enabled: bool,
        admin_email_enabled: bool,
        admin_captcha_enabled: bool,
    ) -> String {
        let session = AuthenticatedSession {
            account_id: Uuid::now_v7(),
            email: "admin@example.test".to_owned(),
            admin_login_name: Some("admin".to_owned()),
            role: "admin".to_owned(),
            device_id: None,
            expires_at: Utc::now() + Duration::minutes(10),
            csrf_token: "csrf-example".to_owned(),
            session_id: Uuid::now_v7(),
        };
        let parts = shared::page_parts(PageId::AdminSettings, locale, &session);
        SettingsTemplate {
            view: parts.view,
            seo: parts.seo,
            session_identity: Some(parts.session_identity),
            csrf_token: parts.csrf_token,
            is_en: parts.is_en,
            settings: Some(AuthSettingsView {
                email_verification_enabled: user_enabled,
                user_captcha_enabled: user_enabled,
                admin_email_verification_enabled: admin_email_enabled,
                admin_captcha_enabled,
                email_cooldown_seconds: 60,
                login_failure_threshold: 5,
                login_lockout_minutes: 30,
                revision: 7,
                updated_at: "2026-07-30 10:00 UTC".to_owned(),
            }),
            load_error: None,
        }
        .render()
        .expect("system settings template should render")
    }

    #[test]
    fn auth_settings_form_is_compact_bilingual_and_revisioned() {
        let zh = render_settings(Locale::ZhCn, true, true, true);
        assert!(zh.contains("普通用户"));
        assert!(zh.contains("管理员"));
        assert!(zh.contains(
            "action=\"/admin/settings/auth-settings\" method=\"post\" hx-post=\"/admin/settings/auth-settings\""
        ));
        assert!(zh.contains("class=\"admin-setting-row\""));
        assert_eq!(zh.matches("class=\"admin-setting-row\"").count(), 4);
        assert_eq!(zh.matches("class=\"admin-switch\"").count(), 4);
        assert!(!zh.contains("class=\"admin-field\""));
        assert!(zh.contains("安全策略"));
        assert!(zh.contains("name=\"email_cooldown_seconds\""));
        assert!(zh.contains("name=\"login_failure_threshold\""));
        assert!(zh.contains("name=\"login_lockout_minutes\""));
        assert!(zh.contains("value=\"60\""));
        assert!(zh.contains("value=\"5\""));
        assert!(zh.contains("value=\"30\""));
        assert!(zh.contains("name=\"csrf_token\" value=\"csrf-example\""));
        assert!(zh.contains("name=\"lang\" value=\"zh-CN\""));
        assert!(zh.contains("name=\"expected_revision\" value=\"7\""));
        assert!(zh.contains("name=\"email_verification_enabled\" value=\"true\" checked"));
        assert!(zh.contains("name=\"user_captcha_enabled\" value=\"true\" checked"));
        assert!(zh.contains("name=\"admin_email_verification_enabled\" value=\"true\" checked"));
        assert!(zh.contains("name=\"admin_captcha_enabled\" value=\"true\" checked"));

        let en = render_settings(Locale::En, false, false, true);
        assert!(en.contains("Login verification"));
        assert!(en.contains("name=\"lang\" value=\"en\""));
        assert!(en.contains("Email verification code"));
        assert!(en.contains("Visual CAPTCHA"));
        assert!(en.contains("Security policy"));
        assert_eq!(en.matches("value=\"true\" checked").count(), 1);
    }
}
