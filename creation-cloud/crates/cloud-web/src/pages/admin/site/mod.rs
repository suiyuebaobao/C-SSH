//! 展示首页二维码发布槽位的当前版本与受控历史。
//! 上传、替代文本更新、发布、撤销和删除分别由独立处理器承担。

pub(crate) mod auth_settings;
pub(crate) mod content;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod publish;
pub(crate) mod revoke;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use cloud_site_content::{SiteContentListQuery, SiteContentRevision, SiteContentState};
use cloud_site_media::{PublicHomeQr, SiteMedia};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SiteQuery {
    lang: Option<String>,
}

struct CurrentMedia {
    content_url: String,
    alt_zh: String,
    alt_en: String,
    dimensions: String,
    published_at: String,
}

struct MediaRow {
    id: String,
    state: &'static str,
    content_type: String,
    byte_size: i64,
    sha256: String,
    dimensions: String,
    alt_zh: String,
    alt_en: String,
    created_at: String,
}

struct ContentHistoryRow {
    id: String,
    document_label_zh: &'static str,
    document_label_en: &'static str,
    locale_label_zh: &'static str,
    locale_label_en: &'static str,
    state: &'static str,
    revision: i64,
    updated_at: String,
    is_published: bool,
}

struct AuthSettingsView {
    email_verification_enabled: bool,
    revision: i64,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin-site.html")]
struct SiteTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    auth_settings: Option<AuthSettingsView>,
    auth_settings_error: Option<String>,
    current: Option<CurrentMedia>,
    rows: Vec<MediaRow>,
    load_error: Option<String>,
    content_editors: Vec<content::ContentEditor>,
    content_history: Vec<ContentHistoryRow>,
    content_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SiteQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let (auth_settings, auth_settings_error) = match state.auth().auth_settings(&actor).await {
        Ok(settings) => (Some(AuthSettingsView::from(settings)), None),
        Err(_) => (
            None,
            Some(if locale == Locale::En {
                "Authentication settings are temporarily unavailable.".to_owned()
            } else {
                "认证设置暂时无法读取。".to_owned()
            }),
        ),
    };
    let (content_editors, content_history, content_error) = match state
        .site_content()
        .list(&actor, SiteContentListQuery::default())
        .await
    {
        Ok(records) => match split_content(records) {
            Ok((editors, history)) => (editors, history, None),
            Err(_) => (Vec::new(), Vec::new(), Some(content_load_error(locale))),
        },
        Err(_) => (Vec::new(), Vec::new(), Some(content_load_error(locale))),
    };
    let (rows, load_error) = match state.site_media().list(&actor, Some(100)).await {
        Ok(items) => (items.into_iter().map(MediaRow::from).collect(), None),
        Err(_) => (
            Vec::new(),
            Some(if locale == Locale::En {
                "Site media history is temporarily unavailable.".to_owned()
            } else {
                "站点媒体历史暂时无法读取。".to_owned()
            }),
        ),
    };
    let (current, current_error) = match state.site_media().current_home_qr().await {
        Ok(media) => (Some(CurrentMedia::from(media)), None),
        Err(AppError::NotFound(_)) => (None, None),
        Err(_) => (
            None,
            Some(if locale == Locale::En {
                "The current publication state is temporarily unavailable.".to_owned()
            } else {
                "当前发布状态暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminSite, locale, &session);
    shared::render(&SiteTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        auth_settings,
        auth_settings_error,
        current,
        rows,
        load_error: load_error.or(current_error),
        content_editors,
        content_history,
        content_error,
    })
}

impl From<cloud_auth::AuthSettings> for AuthSettingsView {
    fn from(value: cloud_auth::AuthSettings) -> Self {
        Self {
            email_verification_enabled: value.email_verification_enabled,
            revision: value.revision,
            updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        }
    }
}

fn split_content(
    records: Vec<SiteContentRevision>,
) -> AppResult<(Vec<content::ContentEditor>, Vec<ContentHistoryRow>)> {
    let mut editors = Vec::new();
    let mut history = Vec::new();
    for record in records {
        if record.state == SiteContentState::Draft {
            editors.push(content::ContentEditor::try_from(record)?);
        } else {
            history.push(ContentHistoryRow::from(record));
        }
    }
    Ok((editors, history))
}

fn content_load_error(locale: Locale) -> String {
    if locale == Locale::En {
        "Structured site content is temporarily unavailable.".to_owned()
    } else {
        "结构化站点内容暂时无法读取。".to_owned()
    }
}

impl From<PublicHomeQr> for CurrentMedia {
    fn from(value: PublicHomeQr) -> Self {
        Self {
            content_url: value.content_url,
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            dimensions: format!("{}×{}", value.width, value.height),
            published_at: value.published_at.to_rfc3339(),
        }
    }
}

impl From<SiteMedia> for MediaRow {
    fn from(value: SiteMedia) -> Self {
        Self {
            id: value.id.to_string(),
            state: value.state.as_str(),
            content_type: value.content_type,
            byte_size: value.byte_size,
            sha256: value.sha256,
            dimensions: format!("{}×{}", value.width, value.height),
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<SiteContentRevision> for ContentHistoryRow {
    fn from(value: SiteContentRevision) -> Self {
        Self {
            id: value.id.to_string(),
            document_label_zh: match value.document_key {
                cloud_site_content::SiteContentDocumentKey::SiteShell => "公共页头与页脚",
                cloud_site_content::SiteContentDocumentKey::Home => "首页正文",
            },
            document_label_en: match value.document_key {
                cloud_site_content::SiteContentDocumentKey::SiteShell => "Shared header and footer",
                cloud_site_content::SiteContentDocumentKey::Home => "Home page",
            },
            locale_label_zh: match value.locale {
                Locale::ZhCn => "简体中文",
                Locale::En => "英文",
            },
            locale_label_en: match value.locale {
                Locale::ZhCn => "Chinese",
                Locale::En => "English",
            },
            state: value.state.as_str(),
            revision: value.revision,
            updated_at: value.updated_at.to_rfc3339(),
            is_published: value.state == SiteContentState::Published,
        }
    }
}

#[cfg(test)]
mod tests {
    use askama::Template;
    use chrono::{Duration, Utc};
    use cloud_domain::AuthenticatedSession;
    use uuid::Uuid;

    use super::*;

    fn render_settings(locale: Locale, enabled: bool) -> String {
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
        let parts = shared::page_parts(PageId::AdminSite, locale, &session);
        SiteTemplate {
            view: parts.view,
            seo: parts.seo,
            session_identity: Some(parts.session_identity),
            csrf_token: parts.csrf_token,
            is_en: parts.is_en,
            auth_settings: Some(AuthSettingsView {
                email_verification_enabled: enabled,
                revision: 7,
                updated_at: "2026-07-29 10:00 UTC".to_owned(),
            }),
            auth_settings_error: None,
            current: None,
            rows: Vec::new(),
            load_error: None,
            content_editors: Vec::new(),
            content_history: Vec::new(),
            content_error: None,
        }
        .render()
        .expect("site settings template should render")
    }

    #[test]
    fn auth_settings_form_is_native_htmx_bilingual_and_revisioned() {
        let zh = render_settings(Locale::ZhCn, true);
        assert!(zh.contains("邮箱验证码"));
        assert!(zh.contains(
            "action=\"/admin/site/auth-settings\" method=\"post\" hx-post=\"/admin/site/auth-settings\""
        ));
        assert!(zh.contains("name=\"csrf_token\" value=\"csrf-example\""));
        assert!(zh.contains("name=\"lang\" value=\"zh-CN\""));
        assert!(zh.contains("name=\"expected_revision\" value=\"7\""));
        assert!(zh.contains("name=\"email_verification_enabled\" value=\"true\" checked"));

        let en = render_settings(Locale::En, false);
        assert!(en.contains("Email verification"));
        assert!(en.contains("name=\"lang\" value=\"en\""));
        assert!(en.contains("Require a code to activate registration"));
        assert!(!en.contains("value=\"true\" checked"));
    }
}
