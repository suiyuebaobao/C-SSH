//! 读取并分页展示登录会话；设备注册元数据仍留在领域 API，不混入会话列表。

pub(crate) mod revoke;
pub(crate) mod session_delete;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_device::SessionView;
use cloud_domain::{AppResult, AuthenticatedSession, PageQuery};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DevicesQuery {
    lang: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    account_id: Option<Uuid>,
}

struct DeviceRow {
    session_id: String,
    account_id: String,
    account_label: String,
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
#[template(path = "admin-devices.html")]
struct DevicesTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<DeviceRow>,
    load_error: Option<String>,
    account_filter: String,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<DevicesQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let page_query = PageQuery {
        page: query.page.unwrap_or(1),
        size: query.size.unwrap_or(20),
    }
    .normalized();
    let (rows, total, load_error) = match state
        .device()
        .admin_list_sessions(&session, query.account_id, page_query)
        .await
    {
        Ok(page) => (
            page.items.into_iter().map(DeviceRow::from).collect(),
            page.total,
            None,
        ),
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Devices are temporarily unavailable.".to_owned()
            } else {
                "设备列表暂时无法读取。".to_owned()
            }),
        ),
    };
    let previous_href =
        (page_query.page > 1).then(|| devices_href(&query, page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| devices_href(&query, page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminDevices, locale, &session);
    shared::render(&DevicesTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
        account_filter: query
            .account_id
            .map_or_else(String::new, |id| id.to_string()),
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<SessionView> for DeviceRow {
    fn from(value: SessionView) -> Self {
        Self {
            session_id: value.session_id.to_string(),
            account_id: value.account_id.to_string(),
            account_label: if value.account_label.trim().is_empty() {
                "—".to_owned()
            } else {
                value.account_label
            },
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

fn devices_href(query: &DevicesQuery, page: u32, locale: Locale) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    params.append_pair("page", &page.to_string());
    if let Some(account_id) = query.account_id {
        params.append_pair("account_id", &account_id.to_string());
    }
    let path = format!("/admin/devices?{}", params.finish());
    shared::localized_admin_path(&path, locale)
}

fn optional(value: Option<String>) -> String {
    value
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| "—".to_owned())
}

fn timestamp(value: chrono::DateTime<chrono::Utc>) -> String {
    value.format("%Y-%m-%d %H:%M UTC").to_string()
}

#[cfg(test)]
mod tests {
    use askama::Template;
    use chrono::{Duration, TimeZone, Utc};
    use cloud_device::SessionView;
    use cloud_site::{Locale, PageId, content_service};
    use uuid::Uuid;

    use super::{DeviceRow, DevicesTemplate};
    use crate::seo::SeoHead;

    #[test]
    fn admin_session_table_has_conventional_columns_expandable_details_and_delete() {
        let now = Utc
            .with_ymd_and_hms(2026, 7, 30, 12, 0, 0)
            .single()
            .expect("fixed timestamp");
        let session_id = Uuid::now_v7();
        let account_id = Uuid::now_v7();
        let row = DeviceRow::from(SessionView {
            session_id,
            status: "offline".to_owned(),
            is_current: false,
            account_id,
            account_label: "user@example.test".to_owned(),
            device_id: Some(Uuid::now_v7()),
            device_name: None,
            last_login_ip: None,
            user_agent: Some("Creation-SSH/7.0.0 Android/15".to_owned()),
            client_version: Some("Creation-SSH 7.0.0".to_owned()),
            device_fingerprint: None,
            created_at: now,
            last_seen_at: now,
            idle_expires_at: now + Duration::days(30),
            absolute_expires_at: now + Duration::days(180),
            revoked_at: None,
        });
        let body = (DevicesTemplate {
            view: content_service().view(PageId::AdminDevices, Locale::ZhCn),
            seo: SeoHead::private(),
            session_identity: Some("admin".to_owned()),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            rows: vec![row],
            load_error: None,
            account_filter: String::new(),
            page_number: 1,
            total: 1,
            previous_href: None,
            next_href: None,
        })
        .render()
        .expect("admin session page should render");

        for marker in [
            "登录设备",
            "状态",
            "最近登录 IP",
            "客户端",
            "最近活动",
            "有效期",
            "操作",
            "不在线",
            "查看详情",
            "会话 ID",
            "设备指纹",
            "完整 User-Agent",
        ] {
            assert!(body.contains(marker), "missing {marker}");
        }
        assert!(body.contains(&format!("/admin/sessions/{session_id}/delete")));
        assert!(body.contains("name=\"csrf_token\" value=\"csrf-example\""));
        assert!(body.contains("hx-confirm="));
        assert!(body.contains(&account_id.to_string()));
        assert!(body.contains("—"));
    }
}
