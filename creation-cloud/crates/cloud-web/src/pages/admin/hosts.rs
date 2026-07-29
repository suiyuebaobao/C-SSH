//! 管理员在明确选中账号后查看该账号的主机元数据。

use askama::Template;
use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_host::{HostStatus, HostView};
use cloud_site::{Locale, PageId, SiteView};
use uuid::Uuid;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct AdminHostRow {
    id: String,
    name: String,
    address: String,
    port: u16,
    platform: String,
    status: &'static str,
    revision: i64,
    source_device_id: String,
    secret_present: bool,
    updated_at: String,
}

impl From<HostView> for AdminHostRow {
    fn from(value: HostView) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            address: value.address,
            port: value.port,
            platform: value.platform,
            status: match value.status {
                HostStatus::Active => "active",
                HostStatus::Disabled => "disabled",
                HostStatus::Archived => "archived",
            },
            revision: value.revision,
            source_device_id: value.source_device_id.to_string(),
            secret_present: value.secret_present,
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin-user-hosts.html")]
struct AdminHostsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    account_id: String,
    rows: Vec<AdminHostRow>,
    total: i64,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let (rows, total, load_error) = match state
        .host()
        .admin_list_for_user(&actor, account_id, query.page_query())
        .await
    {
        Ok(page) => (
            page.items.into_iter().map(AdminHostRow::from).collect(),
            page.total,
            None,
        ),
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Hosts are temporarily unavailable.".to_owned()
            } else {
                "主机列表暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminUsers, locale, &session);
    shared::render(&AdminHostsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        account_id: account_id.to_string(),
        rows,
        total,
        load_error,
    })
}
