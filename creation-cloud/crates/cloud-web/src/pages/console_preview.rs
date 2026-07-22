//! 为无数据库的独立视觉预览保留控制台静态壳；完整服务永远使用有状态控制台。

use axum::{extract::Query, response::Html};
use cloud_domain::AppResult;
use cloud_site::PageId;

use crate::{query::LocaleQuery, render};

pub(crate) async fn overview(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Console, query.locale())
}

pub(crate) async fn profile(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::Profile, query.locale())
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

pub(crate) async fn downloads(Query(query): Query<LocaleQuery>) -> AppResult<Html<String>> {
    render::console(PageId::ConsoleDownloads, query.locale())
}
