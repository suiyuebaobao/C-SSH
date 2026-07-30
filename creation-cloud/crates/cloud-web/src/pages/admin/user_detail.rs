//! Simple account detail page that keeps user-owned data behind an explicit selection.

use std::collections::HashMap;

use askama::Template;
use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Html,
};
use cloud_admin::{AdminDevice, AdminDeviceListQuery, AdminUser};
use cloud_domain::{AppResult, AuthenticatedSession, PageQuery};
use cloud_host::{AdminSyncDirection, AdminSyncRecord, HostStatus, HostView};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct UserDetailQuery {
    lang: Option<String>,
    tab: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

struct UserSummary {
    id: String,
    email: String,
    login_name: String,
    display_name: String,
    role: &'static str,
    status: &'static str,
    email_verified: bool,
    device_count: i64,
    host_count: i64,
    created_at: String,
    updated_at: String,
}

impl From<AdminUser> for UserSummary {
    fn from(value: AdminUser) -> Self {
        Self {
            id: value.id.to_string(),
            email: value.masked_email,
            login_name: value.admin_login_name.unwrap_or_default(),
            display_name: value.display_name,
            role: value.role.as_str(),
            status: value.status.as_str(),
            email_verified: value.email_verified,
            device_count: value.device_count,
            host_count: value.host_count,
            created_at: value.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        }
    }
}

struct HostRow {
    name: String,
    endpoint: String,
    platform: String,
    tags: String,
    status: &'static str,
    revision: i64,
    source_device: String,
    secret_present: bool,
    updated_at: String,
}

struct SyncRow {
    direction: &'static str,
    device: String,
    platform: String,
    outcome: String,
    revision: i64,
    changed_count: i32,
    occurred_at: String,
}

struct DeviceRow {
    id: String,
    name: String,
    platform: &'static str,
    public_id: String,
    revoked: bool,
    last_seen_at: String,
}

impl From<AdminDevice> for DeviceRow {
    fn from(value: AdminDevice) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            platform: value.platform.as_str(),
            public_id: value.public_id,
            revoked: value.revoked_at.is_some(),
            last_seen_at: value.last_seen_at.map_or_else(
                || "—".to_owned(),
                |at| at.format("%Y-%m-%d %H:%M UTC").to_string(),
            ),
        }
    }
}

#[derive(Template)]
#[template(path = "admin-user-detail.html")]
struct UserDetailTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    account_id: String,
    active_tab: String,
    user: Option<UserSummary>,
    hosts: Vec<HostRow>,
    sync_records: Vec<SyncRow>,
    devices: Vec<DeviceRow>,
    total: i64,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Query(query): Query<UserDetailQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let active_tab = valid_tab(query.tab.as_deref());
    let page = PageQuery {
        page: query.page.unwrap_or(1),
        size: query.size.unwrap_or(50),
    }
    .normalized();

    let mut hosts = Vec::new();
    let mut sync_records = Vec::new();
    let mut devices = Vec::new();
    let mut total = 0;
    let mut load_error = None;
    let user = match state.admin().get_user(&actor, account_id).await {
        Ok(user) => Some(UserSummary::from(user)),
        Err(_) => {
            load_error = Some(message(locale, "user"));
            None
        }
    };

    if user.is_some() {
        match active_tab {
            "hosts" => {
                let device_page = state
                    .admin()
                    .list_devices(
                        &actor,
                        AdminDeviceListQuery {
                            page: PageQuery { page: 1, size: 100 },
                            account_id: Some(account_id),
                            platform: None,
                            revoked: None,
                        },
                    )
                    .await
                    .ok();
                let names = device_page
                    .map(|result| {
                        result
                            .items
                            .into_iter()
                            .map(|device| (device.id, device.name))
                            .collect::<HashMap<_, _>>()
                    })
                    .unwrap_or_default();
                match state
                    .host()
                    .admin_list_for_user(&actor, account_id, page)
                    .await
                {
                    Ok(result) => {
                        total = result.total;
                        hosts = result
                            .items
                            .into_iter()
                            .map(|host| host_row(host, &names))
                            .collect();
                    }
                    Err(_) => load_error = Some(message(locale, "hosts")),
                }
            }
            "sync" => match state
                .host()
                .admin_list_sync_records(&actor, account_id, page)
                .await
            {
                Ok(result) => {
                    total = result.total;
                    sync_records = result.items.into_iter().map(sync_row).collect();
                }
                Err(_) => load_error = Some(message(locale, "sync")),
            },
            "devices" => match state
                .admin()
                .list_devices(
                    &actor,
                    AdminDeviceListQuery {
                        page,
                        account_id: Some(account_id),
                        platform: None,
                        revoked: None,
                    },
                )
                .await
            {
                Ok(result) => {
                    total = result.total;
                    devices = result.items.into_iter().map(DeviceRow::from).collect();
                }
                Err(_) => load_error = Some(message(locale, "devices")),
            },
            _ => {}
        }
    }

    let parts = shared::page_parts(PageId::AdminUsers, locale, &session);
    shared::render(&UserDetailTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        account_id: account_id.to_string(),
        active_tab: active_tab.to_owned(),
        user,
        hosts,
        sync_records,
        devices,
        total,
        load_error,
    })
}

fn valid_tab(value: Option<&str>) -> &'static str {
    match value {
        Some("basic") => "basic",
        Some("sync") => "sync",
        Some("devices") => "devices",
        _ => "hosts",
    }
}

fn host_row(value: HostView, names: &HashMap<Uuid, String>) -> HostRow {
    HostRow {
        name: value.name,
        endpoint: format!("{}:{}", value.address, value.port),
        platform: value.platform,
        tags: value.tags.join(" · "),
        status: match value.status {
            HostStatus::Active => "active",
            HostStatus::Disabled => "disabled",
            HostStatus::Archived => "archived",
        },
        revision: value.revision,
        source_device: names
            .get(&value.source_device_id)
            .cloned()
            .unwrap_or_else(|| value.source_device_id.to_string()),
        secret_present: value.secret_present,
        updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
    }
}

fn sync_row(value: AdminSyncRecord) -> SyncRow {
    SyncRow {
        direction: match value.direction {
            AdminSyncDirection::Upload => "upload",
            AdminSyncDirection::Download => "download",
        },
        device: value.device_name,
        platform: value.device_platform,
        outcome: value.outcome,
        revision: value.revision,
        changed_count: value.changed_count,
        occurred_at: value.occurred_at.format("%Y-%m-%d %H:%M UTC").to_string(),
    }
}

fn message(locale: Locale, subject: &str) -> String {
    if locale == Locale::En {
        format!("{subject} data is temporarily unavailable.")
    } else {
        match subject {
            "user" => "用户信息暂时无法读取。",
            "hosts" => "主机数据暂时无法读取。",
            "sync" => "同步记录暂时无法读取。",
            "devices" => "设备数据暂时无法读取。",
            _ => "数据暂时无法读取。",
        }
        .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::valid_tab;

    #[test]
    fn unknown_tabs_return_to_hosts() {
        assert_eq!(valid_tab(Some("sync")), "sync");
        assert_eq!(valid_tab(Some("unknown")), "hosts");
        assert_eq!(valid_tab(None), "hosts");
    }
}
