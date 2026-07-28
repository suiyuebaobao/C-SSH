//! 展示并维护公开页面使用的双语 SEO 主题词。
//! 主题词只是内容治理输入；页面不会把它们解释为排名承诺或隐藏堆词指令。

pub(crate) mod create;
pub(crate) mod delete;
#[cfg(test)]
mod tests;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_seo::SeoTopic;
use cloud_site::{Locale, PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct TopicRow {
    id: String,
    locale: &'static str,
    phrase: String,
    sort_order: i32,
    enabled: bool,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin-seo.html")]
struct SeoTopicsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<TopicRow>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let (rows, load_error) = match state.seo().list_topics(&actor).await {
        Ok(topics) => (topics.into_iter().map(TopicRow::from).collect(), None),
        Err(_) => (
            Vec::new(),
            Some(if locale == Locale::En {
                "SEO topics are temporarily unavailable.".to_owned()
            } else {
                "SEO 主题词暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::AdminSeo, locale, &session);
    shared::render(&SeoTopicsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
    })
}

impl From<SeoTopic> for TopicRow {
    fn from(value: SeoTopic) -> Self {
        Self {
            id: value.id.to_string(),
            locale: value.locale.as_str(),
            phrase: value.phrase,
            sort_order: value.sort_order,
            enabled: value.enabled,
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}
