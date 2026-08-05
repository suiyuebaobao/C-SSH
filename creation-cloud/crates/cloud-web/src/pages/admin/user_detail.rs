//! Simple account detail page that keeps user-owned data behind an explicit selection.

use std::collections::HashMap;

use askama::Template;
use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Html,
};
use cloud_admin::{AdminDeviceListQuery, AdminUser};
use cloud_device::SessionView;
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
    email_value: String,
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
        let email = value.email_for_admin_page().to_owned();
        let email_value = if email != "—" {
            email.clone()
        } else {
            String::new()
        };
        Self {
            id: value.id.to_string(),
            email,
            email_value,
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
    id: String,
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
    record_id: String,
    direction: &'static str,
    device: String,
    platform: String,
    outcome: String,
    revision: i64,
    changed_count: i32,
    occurred_at: String,
}

struct SessionRow {
    session_id: String,
    device_id: String,
    device_name: String,
    online: bool,
    is_current: bool,
    last_login_ip: String,
    client_version: String,
    last_seen_at: String,
    idle_expires_at: String,
    absolute_expires_at: String,
    device_fingerprint: String,
    user_agent: String,
    created_at: String,
    revoked_at: String,
    can_delete: bool,
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
    devices: Vec<SessionRow>,
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
                .device()
                .admin_list_sessions(&session, Some(account_id), page)
                .await
            {
                Ok(result) => {
                    total = result.total;
                    devices = result.items.into_iter().map(SessionRow::from).collect();
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

impl From<SessionView> for SessionRow {
    fn from(value: SessionView) -> Self {
        Self {
            session_id: value.session_id.to_string(),
            device_id: value
                .device_id
                .map_or_else(|| "—".to_owned(), |id| id.to_string()),
            device_name: optional(value.device_name),
            online: value.status == "online",
            is_current: value.is_current,
            last_login_ip: optional(value.last_login_ip),
            client_version: optional(value.client_version),
            last_seen_at: timestamp(value.last_seen_at),
            idle_expires_at: timestamp(value.idle_expires_at),
            absolute_expires_at: timestamp(value.absolute_expires_at),
            device_fingerprint: optional(value.device_fingerprint),
            user_agent: optional(value.user_agent),
            created_at: timestamp(value.created_at),
            revoked_at: value.revoked_at.map_or_else(|| "—".to_owned(), timestamp),
            can_delete: true,
        }
    }
}

fn optional(value: Option<String>) -> String {
    value
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| "—".to_owned())
}

fn timestamp(value: chrono::DateTime<chrono::Utc>) -> String {
    value.format("%Y-%m-%d %H:%M UTC").to_string()
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
        id: value.id.to_string(),
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
        record_id: value.record_id,
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
    use askama::Template;
    use cloud_site::{Locale, PageId, content_service};

    use super::{HostRow, SessionRow, SyncRow, UserDetailTemplate, UserSummary, valid_tab};
    use crate::seo::SeoHead;

    #[test]
    fn unknown_tabs_return_to_hosts() {
        assert_eq!(valid_tab(Some("sync")), "sync");
        assert_eq!(valid_tab(Some("unknown")), "hosts");
        assert_eq!(valid_tab(None), "hosts");
    }

    fn template(
        account_id: &str,
        active_tab: &str,
        hosts: Vec<HostRow>,
        sync_records: Vec<SyncRow>,
        devices: Vec<SessionRow>,
    ) -> UserDetailTemplate {
        UserDetailTemplate {
            view: content_service().view(PageId::AdminUsers, Locale::ZhCn),
            seo: SeoHead::private(),
            session_identity: Some("admin".to_owned()),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            account_id: account_id.to_owned(),
            active_tab: active_tab.to_owned(),
            user: Some(UserSummary {
                id: account_id.to_owned(),
                email: "user@example.test".to_owned(),
                email_value: "user@example.test".to_owned(),
                login_name: "test-admin".to_owned(),
                display_name: "Test User".to_owned(),
                role: "admin",
                status: "active",
                email_verified: true,
                device_count: 1,
                host_count: 1,
                created_at: "2026-07-30 12:00 UTC".to_owned(),
                updated_at: "2026-07-30 12:00 UTC".to_owned(),
            }),
            hosts,
            sync_records,
            devices,
            total: 1,
            load_error: None,
        }
    }

    #[test]
    fn user_device_tab_uses_the_same_session_table_and_scoped_delete() {
        let account_id = uuid::Uuid::now_v7().to_string();
        let session_id = uuid::Uuid::now_v7().to_string();
        let body = template(
            &account_id,
            "devices",
            Vec::new(),
            Vec::new(),
            vec![SessionRow {
                session_id: session_id.clone(),
                device_id: "—".to_owned(),
                device_name: "Android 手机".to_owned(),
                online: true,
                is_current: false,
                last_login_ip: "192.0.2.20".to_owned(),
                client_version: "Creation-SSH 7.0.0".to_owned(),
                last_seen_at: "2026-07-30 12:00 UTC".to_owned(),
                idle_expires_at: "2026-08-29 12:00 UTC".to_owned(),
                absolute_expires_at: "2027-01-26 12:00 UTC".to_owned(),
                device_fingerprint: "fp-user-detail".to_owned(),
                user_agent: "Creation-SSH/7.0.0 Android/15".to_owned(),
                created_at: "2026-07-30 12:00 UTC".to_owned(),
                revoked_at: "—".to_owned(),
                can_delete: true,
            }],
        )
        .render()
        .expect("user detail should render");

        assert!(body.contains("当前设备") || body.contains("登录设备"));
        assert!(body.contains(&format!("/admin/sessions/{session_id}/delete")));
        assert!(body.contains(&format!("name=\"account_id\" value=\"{account_id}\"")));
        assert!(body.contains("name=\"csrf_token\" value=\"csrf-example\""));
        assert!(body.contains("hx-confirm="));
        assert!(body.contains("fp-user-detail"));
        assert!(body.contains("完整 User-Agent"));
    }

    #[test]
    fn host_and_sync_tabs_use_account_scoped_delete_routes() {
        let account_id = uuid::Uuid::now_v7().to_string();
        let host_id = uuid::Uuid::now_v7().to_string();
        let record_id = format!("download:{}", uuid::Uuid::now_v7());
        let host_body = template(
            &account_id,
            "hosts",
            vec![HostRow {
                id: host_id.clone(),
                name: "Host".into(),
                endpoint: "192.0.2.10:22".into(),
                platform: "linux".into(),
                tags: String::new(),
                status: "active",
                revision: 3,
                source_device: "device".into(),
                secret_present: true,
                updated_at: "now".into(),
            }],
            Vec::new(),
            Vec::new(),
        )
        .render()
        .expect("host tab should render");
        let sync_body = template(
            &account_id,
            "sync",
            Vec::new(),
            vec![SyncRow {
                record_id: record_id.clone(),
                direction: "download",
                device: "device".into(),
                platform: "windows".into(),
                outcome: "acknowledged".into(),
                revision: 3,
                changed_count: 0,
                occurred_at: "now".into(),
            }],
            Vec::new(),
        )
        .render()
        .expect("sync tab should render");

        assert!(host_body.contains(&format!("/admin/users/{account_id}/hosts/{host_id}/delete")));
        assert!(sync_body.contains(&format!(
            "/admin/users/{account_id}/sync-records/{record_id}/delete"
        )));
        for body in [&host_body, &sync_body] {
            assert!(body.contains("name=\"csrf_token\" value=\"csrf-example\""));
            assert!(body.contains("hx-confirm="));
        }
    }

    #[test]
    fn account_tab_exposes_complete_update_and_permanent_delete_forms() {
        let account_id = uuid::Uuid::now_v7().to_string();
        let body = template(&account_id, "basic", Vec::new(), Vec::new(), Vec::new())
            .render()
            .expect("account tab should render");
        assert!(body.contains(&format!(
            "action=\"/admin/users/{account_id}\" method=\"post\""
        )));
        assert!(body.contains(&format!("/admin/users/{account_id}/delete")));
        for field in [
            "email",
            "display_name",
            "admin_login_name",
            "role",
            "status",
            "new_password",
        ] {
            assert!(
                body.contains(&format!("name=\"{field}\"")),
                "missing {field}"
            );
        }
        assert!(body.contains("name=\"csrf_token\" value=\"csrf-example\""));
        assert!(body.matches("hx-confirm=").count() >= 2);
    }
}
