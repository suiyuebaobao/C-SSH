//! 处理用户中心总览、设备、同步、模型与保险库展示页面。

use axum::{extract::Query, response::Html};
use cloud_domain::AppResult;
use cloud_site::PageId;

use crate::{query::LocaleQuery, render};

pub(crate) async fn overview(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Console, query.locale())
}

pub(crate) async fn devices(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Devices, query.locale())
}

pub(crate) async fn sync(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Sync, query.locale())
}

pub(crate) async fn models(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Models, query.locale())
}

pub(crate) async fn vault(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Vault, query.locale())
}
