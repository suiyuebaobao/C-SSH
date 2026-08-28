//! Simple create, edit, publish and hide controls for the global announcement API.

use askama::Template;
use axum::{
    Extension, Form,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{Html, Response},
};
use cloud_announcement::{
    Announcement, AnnouncementPriority, CreateAnnouncementInput, ReplaceAnnouncementInput,
    TransitionAnnouncementInput,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

#[derive(Debug, Deserialize)]
pub(crate) struct AnnouncementForm {
    title_zh_cn: String,
    body_zh_cn: String,
    title_en: String,
    body_en: String,
    priority: AnnouncementPriority,
    expected_revision: Option<i64>,
    lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransitionForm {
    expected_revision: i64,
    lang: Option<String>,
}

struct AnnouncementRow {
    id: String,
    title_zh_cn: String,
    body_zh_cn: String,
    title_en: String,
    body_en: String,
    priority: &'static str,
    status: &'static str,
    revision: i64,
    updated_at: String,
}

impl From<Announcement> for AnnouncementRow {
    fn from(value: Announcement) -> Self {
        Self {
            id: value.id.to_string(),
            title_zh_cn: value.title_zh_cn,
            body_zh_cn: value.body_zh_cn,
            title_en: value.title_en,
            body_en: value.body_en,
            priority: value.priority.as_str(),
            status: value.status.as_str(),
            revision: value.revision,
            updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin-announcements.html")]
struct AnnouncementsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<AnnouncementRow>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let (rows, load_error) = match state
        .announcement()
        .list_admin(&actor, query.page_query())
        .await
    {
        Ok(result) => (
            result
                .items
                .into_iter()
                .map(AnnouncementRow::from)
                .collect(),
            None,
        ),
        Err(_) => (
            Vec::new(),
            Some(if locale == Locale::En {
                "Announcements are temporarily unavailable.".to_owned()
            } else {
                "公告暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminAnnouncements, locale, &session);
    shared::render(&AnnouncementsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
    })
}

pub(crate) async fn create(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<AnnouncementForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateAnnouncementInput {
        title_zh_cn: form.title_zh_cn,
        body_zh_cn: form.body_zh_cn,
        title_en: form.title_en,
        body_en: form.body_en,
        priority: form.priority,
    };
    match state.announcement().create_admin(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/announcements", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

pub(crate) async fn update(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<AnnouncementForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let Some(expected_revision) = form.expected_revision else {
        return shared::action_error(
            locale,
            cloud_domain::AppError::Validation("缺少 expected_revision".to_owned()),
        );
    };
    let input = ReplaceAnnouncementInput {
        expected_revision,
        title_zh_cn: form.title_zh_cn,
        body_zh_cn: form.body_zh_cn,
        title_en: form.title_en,
        body_en: form.body_en,
        priority: form.priority,
    };
    match state.announcement().replace_admin(&actor, id, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/announcements", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

pub(crate) async fn publish(
    state: State<AdminPageState>,
    session: Extension<AuthenticatedSession>,
    path: Path<Uuid>,
    headers: HeaderMap,
    form: Form<TransitionForm>,
) -> Response {
    transition(state, session, path, headers, form, "publish").await
}

pub(crate) async fn hide(
    state: State<AdminPageState>,
    session: Extension<AuthenticatedSession>,
    path: Path<Uuid>,
    headers: HeaderMap,
    form: Form<TransitionForm>,
) -> Response {
    transition(state, session, path, headers, form, "hide").await
}

pub(crate) async fn delete(
    state: State<AdminPageState>,
    session: Extension<AuthenticatedSession>,
    path: Path<Uuid>,
    headers: HeaderMap,
    form: Form<TransitionForm>,
) -> Response {
    transition(state, session, path, headers, form, "delete").await
}

async fn transition(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<TransitionForm>,
    action: &str,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = TransitionAnnouncementInput {
        expected_revision: form.expected_revision,
    };
    let result = match action {
        "publish" => state
            .announcement()
            .publish_admin(&actor, id, input)
            .await
            .map(|_| ()),
        "hide" => state
            .announcement()
            .hide_admin(&actor, id, input)
            .await
            .map(|_| ()),
        _ => state.announcement().delete_admin(&actor, id, input).await,
    };
    match result {
        Ok(()) => shared::action_success(&headers, "/admin/announcements", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

#[cfg(test)]
mod tests {
    const TEMPLATE: &str = include_str!("../../../templates/admin-announcements.html");

    #[test]
    fn announcement_forms_expose_only_the_closed_priority_set() {
        assert_eq!(TEMPLATE.matches("name=\"priority\"").count(), 3);
        for value in ["normal", "important", "critical"] {
            assert!(
                TEMPLATE.contains(&format!("value=\"{value}\"")),
                "公告表单缺少优先级 {value}"
            );
        }
        assert!(TEMPLATE.contains("row.priority == \"critical\""));
        assert!(!TEMPLATE.contains("value=\"urgent\""));
    }

    #[test]
    fn account_notification_form_is_plain_language_and_has_no_internal_history() {
        assert!(TEMPLATE.contains("安全检查提醒"));
        assert!(TEMPLATE.contains("Security review required"));
        assert!(TEMPLATE.contains("一般"));
        assert!(!TEMPLATE.contains("name=\"resource_id\""));
        assert!(!TEMPLATE.contains("notification_rows"));
        assert!(!TEMPLATE.contains("<th>kind</th>"));
    }

    #[test]
    fn draft_and_published_announcements_offer_edit_and_delete_without_hide() {
        assert!(TEMPLATE.contains("row.status == \"draft\" || row.status == \"published\""));
        assert!(TEMPLATE.contains("button button-danger"));
        assert!(TEMPLATE.contains("客户端将不再显示"));
        assert!(TEMPLATE.contains("网站仍保留历史记录"));
        assert!(
            TEMPLATE.contains("{% if row.status != \"draft\" %}<div class=\"admin-simple-copy\"")
        );
        assert!(!TEMPLATE.contains("/hide"));
    }
}
