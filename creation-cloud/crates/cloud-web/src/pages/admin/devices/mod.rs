//! 读取、筛选并分页展示非 SSH 的客户端设备元数据。
//! 设备撤销由独立写处理器调用管理领域用例。

pub(crate) mod revoke;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_admin::{AdminDevice, AdminDeviceListQuery, AdminDevicePlatform};
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
    #[serde(default, deserialize_with = "empty_platform_as_none")]
    platform: Option<AdminDevicePlatform>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    revoked: Option<bool>,
}

struct DeviceRow {
    id: String,
    account_id: String,
    owner_email: String,
    name: String,
    platform: &'static str,
    public_id: String,
    revoked: bool,
    last_seen_at: String,
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
    platform_filter: String,
    revoked_filter: String,
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
    let actor = shared::actor_from_session(&session)?;
    let page_query = PageQuery {
        page: query.page.unwrap_or(1),
        size: query.size.unwrap_or(20),
    }
    .normalized();
    let request = AdminDeviceListQuery {
        page: page_query,
        account_id: query.account_id,
        platform: query.platform,
        revoked: query.revoked,
    };
    let (rows, total, load_error) = match state.admin().list_devices(&actor, request).await {
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
        platform_filter: query
            .platform
            .map_or("", AdminDevicePlatform::as_str)
            .to_owned(),
        revoked_filter: query
            .revoked
            .map_or_else(String::new, |value| value.to_string()),
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<AdminDevice> for DeviceRow {
    fn from(value: AdminDevice) -> Self {
        Self {
            id: value.id.to_string(),
            account_id: value.account_id.to_string(),
            owner_email: value.owner_masked_email,
            name: value.name,
            platform: value.platform.as_str(),
            public_id: value.public_id,
            revoked: value.revoked_at.is_some(),
            last_seen_at: value
                .last_seen_at
                .map_or_else(|| "—".to_owned(), |at| at.to_rfc3339()),
        }
    }
}

fn devices_href(query: &DevicesQuery, page: u32, locale: Locale) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    params.append_pair("page", &page.to_string());
    if let Some(account_id) = query.account_id {
        params.append_pair("account_id", &account_id.to_string());
    }
    if let Some(platform) = query.platform {
        params.append_pair("platform", platform.as_str());
    }
    if let Some(revoked) = query.revoked {
        params.append_pair("revoked", &revoked.to_string());
    }
    let path = format!("/admin/devices?{}", params.finish());
    shared::localized_admin_path(&path, locale)
}

fn empty_platform_as_none<'de, D>(deserializer: D) -> Result<Option<AdminDevicePlatform>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some("windows") => Ok(Some(AdminDevicePlatform::Windows)),
        Some("linux") => Ok(Some(AdminDevicePlatform::Linux)),
        Some("android") => Ok(Some(AdminDevicePlatform::Android)),
        Some("ios") => Ok(Some(AdminDevicePlatform::Ios)),
        Some("macos") => Ok(Some(AdminDevicePlatform::Macos)),
        Some("web") => Ok(Some(AdminDevicePlatform::Web)),
        Some(_) => Err(serde::de::Error::custom("invalid device platform")),
    }
}
