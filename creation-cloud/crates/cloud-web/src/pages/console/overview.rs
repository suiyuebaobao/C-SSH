//! 从各业务域聚合当前账号控制台总览，不生成占位统计。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};
use cloud_sync::AccountSyncSummary;
use cloud_user::Profile;

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-overview.html")]
struct OverviewTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    profile: Profile,
    device_total: i64,
    model_total: i64,
    vault_total: i64,
    sync: AccountSyncSummary,
    release_total: usize,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let page = common::first_page();
    let (profile, devices, models, vault, sync, releases) = tokio::try_join!(
        state.user().get(&session, session.account_id),
        state.device().list(&session, page),
        state.model().list(session.account_id, page),
        state.vault().list(session.account_id, page),
        state.sync().account_summary(&session),
        state.download().public_manifest(),
    )?;
    let locale = query.locale();
    common::render(&OverviewTemplate {
        view: common::view(PageId::Console, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        profile,
        device_total: devices.total,
        model_total: models.total,
        vault_total: vault.total,
        sync,
        release_total: releases.len(),
    })
}
