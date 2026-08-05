//! 读取、筛选并分页展示管理员可见的完整用户邮箱。
//! 角色与状态变更交由独立写处理器调用管理领域用例。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod host_delete;
pub(crate) mod sync_record_delete;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_admin::{AdminUser, AdminUserListQuery, AdminUserRole, AdminUserStatus};
use cloud_domain::{AppResult, AuthenticatedSession, PageQuery};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct UsersQuery {
    lang: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    search: Option<String>,
    #[serde(default, deserialize_with = "empty_role_as_none")]
    role: Option<AdminUserRole>,
    #[serde(default, deserialize_with = "empty_status_as_none")]
    status: Option<AdminUserStatus>,
}

struct UserRow {
    id: String,
    email: String,
    login_name: String,
    display_name: String,
    role: &'static str,
    status: &'static str,
    device_count: i64,
    host_count: i64,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin-users.html")]
struct UsersTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<UserRow>,
    load_error: Option<String>,
    search_filter: String,
    role_filter: String,
    status_filter: String,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<UsersQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let page_query = PageQuery {
        page: query.page.unwrap_or(1),
        size: query.size.unwrap_or(20),
    }
    .normalized();
    let request = AdminUserListQuery {
        page: page_query,
        search: query.search.clone(),
        email: None,
        role: query.role,
        status: query.status,
    };
    let (rows, total, load_error) = match state.admin().list_users(&actor, request).await {
        Ok(page) => (
            page.items.into_iter().map(UserRow::from).collect(),
            page.total,
            None,
        ),
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Users are temporarily unavailable.".to_owned()
            } else {
                "用户列表暂时无法读取。".to_owned()
            }),
        ),
    };
    let previous_href =
        (page_query.page > 1).then(|| users_href(&query, page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| users_href(&query, page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminUsers, locale, &session);
    shared::render(&UsersTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
        search_filter: query.search.unwrap_or_default(),
        role_filter: query.role.map_or("", AdminUserRole::as_str).to_owned(),
        status_filter: query.status.map_or("", AdminUserStatus::as_str).to_owned(),
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<AdminUser> for UserRow {
    fn from(value: AdminUser) -> Self {
        let email = value.email_for_admin_page().to_owned();
        Self {
            id: value.id.to_string(),
            email,
            login_name: value.admin_login_name.unwrap_or_default(),
            display_name: value.display_name,
            role: value.role.as_str(),
            status: value.status.as_str(),
            device_count: value.device_count,
            host_count: value.host_count,
            updated_at: value.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        }
    }
}

fn users_href(query: &UsersQuery, page: u32, locale: Locale) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    params.append_pair("page", &page.to_string());
    if let Some(search) = query.search.as_deref().filter(|value| !value.is_empty()) {
        params.append_pair("search", search);
    }
    if let Some(role) = query.role {
        params.append_pair("role", role.as_str());
    }
    if let Some(status) = query.status {
        params.append_pair("status", status.as_str());
    }
    let path = format!("/admin/users?{}", params.finish());
    shared::localized_admin_path(&path, locale)
}

fn empty_role_as_none<'de, D>(deserializer: D) -> Result<Option<AdminUserRole>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some("user") => Ok(Some(AdminUserRole::User)),
        Some("admin") => Ok(Some(AdminUserRole::Admin)),
        Some(_) => Err(serde::de::Error::custom("invalid user role")),
    }
}

pub(crate) fn empty_status_as_none<'de, D>(
    deserializer: D,
) -> Result<Option<AdminUserStatus>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some("pending_verification") => Ok(Some(AdminUserStatus::PendingVerification)),
        Some("active") => Ok(Some(AdminUserStatus::Active)),
        Some("disabled") => Ok(Some(AdminUserStatus::Disabled)),
        Some(_) => Err(serde::de::Error::custom("invalid user status")),
    }
}

#[cfg(test)]
mod tests {
    use askama::Template;
    use cloud_site::{Locale, PageId, content_service};

    use super::UsersTemplate;
    use crate::seo::SeoHead;

    #[test]
    fn user_list_exposes_real_create_form_with_csrf_and_confirmation() {
        let body = (UsersTemplate {
            view: content_service().view(PageId::AdminUsers, Locale::ZhCn),
            seo: SeoHead::private(),
            session_identity: Some("admin".to_owned()),
            csrf_token: "csrf-create".to_owned(),
            is_en: false,
            rows: Vec::new(),
            load_error: None,
            search_filter: String::new(),
            role_filter: String::new(),
            status_filter: String::new(),
            page_number: 1,
            total: 0,
            previous_href: None,
            next_href: None,
        })
        .render()
        .expect("users page should render");

        assert!(body.contains("action=\"/admin/users\" method=\"post\""));
        assert!(body.contains("hx-post=\"/admin/users\""));
        assert!(body.contains("name=\"csrf_token\" value=\"csrf-create\""));
        assert!(body.contains("hx-confirm="));
        for field in ["email", "display_name", "password", "role", "status"] {
            assert!(
                body.contains(&format!("name=\"{field}\"")),
                "missing {field}"
            );
        }
    }
}
