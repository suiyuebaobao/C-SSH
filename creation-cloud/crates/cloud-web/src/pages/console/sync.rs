//! 渲染账号级只读同步摘要，不调用或放宽产品设备同步 actor。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};
use cloud_sync::AccountSyncSummary;

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-sync.html")]
struct SyncTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    summary: AccountSyncSummary,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let summary = state.sync().account_summary(&session).await?;
    let locale = query.locale();
    common::render(&SyncTemplate {
        view: common::view(PageId::Sync, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        summary,
    })
}
