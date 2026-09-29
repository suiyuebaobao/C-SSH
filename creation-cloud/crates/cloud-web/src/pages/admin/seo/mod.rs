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
use cloud_seo::{SeoLocale, SeoTopic};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SeoPageQuery {
    lang: Option<String>,
    content_lang: Option<String>,
}

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
    content_lang: &'static str,
    rows: Vec<TopicRow>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SeoPageQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let content_locale = selected_content_locale(query.content_lang.as_deref(), locale)?;
    let actor = shared::actor_from_session(&session)?;
    let (rows, load_error) = match state.seo().list_topics(&actor).await {
        Ok(topics) => (
            topics
                .into_iter()
                .filter(|topic| topic.locale == content_locale)
                .map(TopicRow::from)
                .collect(),
            None,
        ),
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
        content_lang: content_locale.as_str(),
        rows,
        load_error,
    })
}

fn selected_content_locale(value: Option<&str>, ui_locale: Locale) -> AppResult<SeoLocale> {
    match value.map(str::trim) {
        None | Some("") if ui_locale == Locale::En => Ok(SeoLocale::En),
        None | Some("") => Ok(SeoLocale::ZhCn),
        Some("zh-CN") => Ok(SeoLocale::ZhCn),
        Some("en") => Ok(SeoLocale::En),
        Some(_) => Err(cloud_domain::AppError::Validation(
            "SEO 内容语种无效".to_owned(),
        )),
    }
}

pub(crate) fn return_path(content_locale: SeoLocale) -> String {
    format!("/admin/seo?content_lang={}", content_locale.as_str())
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
