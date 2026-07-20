//! 装配管理后台的测试占位入口与生产 SSR/HTMX 页面域。
//! 生产入口必须携带管理员会话和 `AdminPageState`，无状态入口只供纯页面测试。

pub(crate) mod assets;
pub(crate) mod audit;
pub(crate) mod devices;
pub(crate) mod feedback;
pub(crate) mod overview;
pub(crate) mod releases;
pub(crate) mod shared;
pub(crate) mod site;
pub(crate) mod users;

use axum::{extract::Query, response::Html};
use cloud_domain::AppResult;
use cloud_site::PageId;

use crate::{query::LocaleQuery, render};

pub(crate) async fn static_overview(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::Admin, query.locale())
}

pub(crate) async fn static_users(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminUsers, query.locale())
}

pub(crate) async fn static_devices(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminDevices, query.locale())
}

pub(crate) async fn static_releases(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminReleases, query.locale())
}

pub(crate) async fn static_assets(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminAssets, query.locale())
}

pub(crate) async fn static_site(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminSite, query.locale())
}

pub(crate) async fn static_audit(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminAudit, query.locale())
}

pub(crate) async fn static_feedback(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::admin(PageId::AdminFeedback, query.locale())
}
