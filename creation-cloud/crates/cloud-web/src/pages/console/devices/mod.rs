//! 渲染本人设备列表；浏览器页面不提供产品设备登记入口。

pub(crate) mod rename;
pub(crate) mod revoke;

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_device::Device;
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
    devices: Vec<Device>,
    total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let devices = state.device().list(&session, common::first_page()).await?;
    let locale = query.locale();
    common::render(&DevicesTemplate {
        view: common::view(PageId::Devices, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        devices: devices.items,
        total: devices.total,
    })
}
