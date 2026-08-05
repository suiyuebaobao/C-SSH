//! 用户云端主机列表。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_host::{HostStatus, HostView};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

struct HostRow {
    name: String,
    address: String,
    port: u16,
    platform: String,
    status: &'static str,
    tags: String,
    revision: i64,
    secret_present: bool,
    updated_at: String,
}

impl From<HostView> for HostRow {
    fn from(value: HostView) -> Self {
        Self {
            name: value.name,
            address: value.address,
            port: value.port,
            platform: value.platform,
            status: status(value.status),
            tags: value.tags.join(", "),
            revision: value.revision,
            secret_present: value.secret_present,
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

fn status(value: HostStatus) -> &'static str {
    match value {
        HostStatus::Active => "active",
        HostStatus::Disabled => "disabled",
        HostStatus::Archived => "archived",
    }
}

#[derive(Template)]
#[template(path = "console-hosts.html")]
struct HostsTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    hosts: Vec<HostRow>,
    total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let host_page = state
        .host()
        .list_self(&session, common::first_page())
        .await?;
    let locale = query.locale();
    common::render(&HostsTemplate {
        view: common::view(PageId::Sync, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        total: host_page.total,
        hosts: host_page.items.into_iter().map(HostRow::from).collect(),
    })
}
