//! 渲染本人登录会话列表；浏览器页面只管理账号会话，不伪装产品设备。

pub(crate) mod rename;
pub(crate) mod revoke;
pub(crate) mod session_revoke;

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_device::SessionView;
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-devices.html")]
struct DevicesTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    rows: Vec<SessionRow>,
    total: i64,
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
    can_revoke: bool,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let sessions = state
        .device()
        .list_sessions(&session, common::first_page())
        .await?;
    let locale = query.locale();
    common::render(&DevicesTemplate {
        view: common::view(PageId::Devices, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        rows: sessions.items.into_iter().map(SessionRow::from).collect(),
        total: sessions.total,
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
            can_revoke: value.revoked_at.is_none(),
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

#[cfg(test)]
mod tests {
    use askama::Template;
    use chrono::{Duration, TimeZone, Utc};
    use cloud_device::SessionView;
    use cloud_site::{Locale, PageId};
    use uuid::Uuid;

    use super::{DevicesTemplate, SessionRow, common};

    #[test]
    fn session_page_keeps_status_current_badge_details_and_single_revoke_action() {
        let now = Utc
            .with_ymd_and_hms(2026, 7, 30, 12, 0, 0)
            .single()
            .expect("fixed timestamp");
        let session_id = Uuid::now_v7();
        let row = SessionRow::from(SessionView {
            session_id,
            status: "online".to_owned(),
            is_current: true,
            account_id: Uuid::now_v7(),
            account_label: "user@example.test".to_owned(),
            device_id: None,
            device_name: Some("Windows 工作站".to_owned()),
            last_login_ip: Some("192.0.2.10".to_owned()),
            user_agent: Some("Creation-SSH/7.0.0 Windows/11".to_owned()),
            client_version: Some("Creation-SSH 7.0.0".to_owned()),
            device_fingerprint: Some("fp-example".to_owned()),
            created_at: now,
            last_seen_at: now,
            idle_expires_at: now + Duration::days(30),
            absolute_expires_at: now + Duration::days(180),
            revoked_at: None,
        });
        let body = (DevicesTemplate {
            view: common::view(PageId::Devices, Locale::ZhCn),
            seo: common::seo(),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            rows: vec![row],
            total: 1,
        })
        .render()
        .expect("session page should render");

        for marker in [
            "最近登录 IP",
            "客户端",
            "有效期",
            "在线",
            "当前设备",
            "查看详情",
            "会话 ID",
            "设备指纹",
            "完整 User-Agent",
            "192.0.2.10",
            "fp-example",
        ] {
            assert!(body.contains(marker), "missing {marker}");
        }
        assert!(body.contains(&format!("/console/devices/sessions/{session_id}/revoke")));
    }
}
