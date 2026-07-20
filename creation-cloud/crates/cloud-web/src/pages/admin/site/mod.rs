//! 展示首页二维码发布槽位的当前版本与受控历史。
//! 上传、替代文本更新、发布、撤销和删除分别由独立处理器承担。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod publish;
pub(crate) mod revoke;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use cloud_site_media::{PublicHomeQr, SiteMedia};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SiteQuery {
    lang: Option<String>,
}

struct CurrentMedia {
    id: String,
    content_url: String,
    alt_zh: String,
    alt_en: String,
    sha256: String,
    dimensions: String,
    published_at: String,
}

struct MediaRow {
    id: String,
    state: &'static str,
    content_type: String,
    byte_size: i64,
    sha256: String,
    dimensions: String,
    alt_zh: String,
    alt_en: String,
    created_at: String,
}

#[derive(Template)]
#[template(path = "admin-site.html")]
struct SiteTemplate {
    view: SiteView,
    seo: SeoHead,
    session_email: Option<String>,
    csrf_token: String,
    is_en: bool,
    current: Option<CurrentMedia>,
    rows: Vec<MediaRow>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SiteQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let (rows, load_error) = match state.site_media().list(&actor, Some(100)).await {
        Ok(items) => (items.into_iter().map(MediaRow::from).collect(), None),
        Err(_) => (
            Vec::new(),
            Some(if locale == Locale::En {
                "Site media history is temporarily unavailable.".to_owned()
            } else {
                "站点媒体历史暂时无法读取。".to_owned()
            }),
        ),
    };
    let (current, current_error) = match state.site_media().current_home_qr().await {
        Ok(media) => (Some(CurrentMedia::from(media)), None),
        Err(AppError::NotFound(_)) => (None, None),
        Err(_) => (
            None,
            Some(if locale == Locale::En {
                "The current publication state is temporarily unavailable.".to_owned()
            } else {
                "当前发布状态暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminSite, locale, &session);
    shared::render(&SiteTemplate {
        view: parts.view,
        seo: parts.seo,
        session_email: Some(parts.session_email),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        current,
        rows,
        load_error: load_error.or(current_error),
    })
}

impl From<PublicHomeQr> for CurrentMedia {
    fn from(value: PublicHomeQr) -> Self {
        Self {
            id: value.id.to_string(),
            content_url: value.content_url,
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            sha256: value.sha256,
            dimensions: format!("{}×{}", value.width, value.height),
            published_at: value.published_at.to_rfc3339(),
        }
    }
}

impl From<SiteMedia> for MediaRow {
    fn from(value: SiteMedia) -> Self {
        Self {
            id: value.id.to_string(),
            state: value.state.as_str(),
            content_type: value.content_type,
            byte_size: value.byte_size,
            sha256: value.sha256,
            dimensions: format!("{}×{}", value.width, value.height),
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}
