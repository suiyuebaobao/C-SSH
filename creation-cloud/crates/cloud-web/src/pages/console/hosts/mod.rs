//! 用户主机列表与逐设备下载白名单。

pub(crate) mod allowlist;

use std::collections::HashSet;

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

struct HostOption {
    id: String,
    label: String,
    selected: bool,
}

struct DevicePolicy {
    id: String,
    name: String,
    platform: String,
    options: Vec<HostOption>,
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
    device_policies: Vec<DevicePolicy>,
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
    let devices = state.device().list(&session, common::first_page()).await?;
    let mut device_policies = Vec::new();
    for device in devices
        .items
        .into_iter()
        .filter(|item| item.revoked_at.is_none())
    {
        let selected = state
            .host()
            .get_download_allowlist(&session, device.id)
            .await?
            .host_ids
            .into_iter()
            .collect::<HashSet<_>>();
        let options = host_page
            .items
            .iter()
            .map(|host| HostOption {
                id: host.id.to_string(),
                label: format!("{} · {}:{}", host.name, host.address, host.port),
                selected: selected.contains(&host.id),
            })
            .collect();
        device_policies.push(DevicePolicy {
            id: device.id.to_string(),
            name: device.name,
            platform: device.platform,
            options,
        });
    }
    let locale = query.locale();
    common::render(&HostsTemplate {
        view: common::view(PageId::Sync, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        total: host_page.total,
        hosts: host_page.items.into_iter().map(HostRow::from).collect(),
        device_policies,
    })
}
