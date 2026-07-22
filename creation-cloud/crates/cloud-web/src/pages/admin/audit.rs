//! 分页读取服务端不可变审计事件并渲染时间线。
//! 页面只呈现审计领域已经脱敏的字段，不构造客户端伪事件。

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_admin::AuditEvent;
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct AuditRow {
    id: String,
    actor: String,
    action: String,
    resource_kind: String,
    resource_id: String,
    outcome: &'static str,
    request_id: String,
    details: String,
    created_at: String,
}

#[derive(Template)]
#[template(path = "admin-audit.html")]
struct AuditTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<AuditRow>,
    load_error: Option<String>,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.page_query();
    let (rows, total, load_error) = match state.admin().list_audit_events(&actor, page_query).await
    {
        Ok(page) => (
            page.items.into_iter().map(AuditRow::from).collect(),
            page.total,
            None,
        ),
        Err(_) => (
            Vec::new(),
            0,
            Some(if parts_is_en(locale) {
                "Audit events are temporarily unavailable.".to_owned()
            } else {
                "审计事件暂时无法读取。".to_owned()
            }),
        ),
    };
    let previous_href = (page_query.page > 1).then(|| audit_href(page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| audit_href(page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminAudit, locale, &session);
    shared::render(&AuditTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<AuditEvent> for AuditRow {
    fn from(value: AuditEvent) -> Self {
        Self {
            id: value.id.to_string(),
            actor: value
                .actor_account_id
                .map_or_else(|| "system".to_owned(), |id| id.to_string()),
            action: value.action,
            resource_kind: value.resource_kind,
            resource_id: value.resource_id.unwrap_or_else(|| "—".to_owned()),
            outcome: value.outcome.as_str(),
            request_id: value.request_id.unwrap_or_else(|| "—".to_owned()),
            details: value.details.to_string(),
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

fn audit_href(page: u32, locale: cloud_site::Locale) -> String {
    let path = format!("/admin/audit?page={page}");
    shared::localized_admin_path(&path, locale)
}

fn parts_is_en(locale: cloud_site::Locale) -> bool {
    locale == cloud_site::Locale::En
}
