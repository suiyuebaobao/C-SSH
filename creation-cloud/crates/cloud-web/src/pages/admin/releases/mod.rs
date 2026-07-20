//! 分页展示真实发布版本及其状态机位置。
//! 创建、元数据更新、状态迁移和删除分别由独立写处理器承担。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_release::Release;
use cloud_site::{Locale, PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct ReleaseRow {
    id: String,
    version: String,
    channel: &'static str,
    status: &'static str,
    title_zh: String,
    title_en: String,
    notes_zh: String,
    notes_en: String,
    published_at: String,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin-releases.html")]
struct ReleasesTemplate {
    view: SiteView,
    seo: SeoHead,
    session_email: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<ReleaseRow>,
    load_error: Option<String>,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.page_query();
    let (rows, total, load_error) = match state.release().list_releases(&actor, page_query).await {
        Ok(page) => (
            page.items.into_iter().map(ReleaseRow::from).collect(),
            page.total,
            None,
        ),
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Releases are temporarily unavailable.".to_owned()
            } else {
                "版本列表暂时无法读取。".to_owned()
            }),
        ),
    };
    let previous_href = (page_query.page > 1).then(|| release_href(page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| release_href(page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminReleases, locale, &session);
    shared::render(&ReleasesTemplate {
        view: parts.view,
        seo: parts.seo,
        session_email: Some(parts.session_email),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<Release> for ReleaseRow {
    fn from(value: Release) -> Self {
        Self {
            id: value.id.to_string(),
            version: value.version,
            channel: value.channel.as_str(),
            status: value.status.as_str(),
            title_zh: value.title_zh,
            title_en: value.title_en,
            notes_zh: value.notes_zh,
            notes_en: value.notes_en,
            published_at: value
                .published_at
                .map_or_else(|| "—".to_owned(), |at| at.to_rfc3339()),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

fn release_href(page: u32, locale: Locale) -> String {
    shared::localized_admin_path(&format!("/admin/releases?page={page}"), locale)
}
