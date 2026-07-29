//! 用户只读全局模型目录；模型配置由管理员统一维护。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

struct ConsoleModelRow {
    name: String,
    provider: String,
    model_name: String,
    context_length: i32,
    capabilities: String,
    is_default: bool,
}

impl From<cloud_model::GlobalModel> for ConsoleModelRow {
    fn from(value: cloud_model::GlobalModel) -> Self {
        Self {
            name: value.name,
            provider: value.provider,
            model_name: value.model_name,
            context_length: value.context_length,
            capabilities: value.capability_tags.join(", "),
            is_default: value.is_default,
        }
    }
}

#[derive(Template)]
#[template(path = "console-models.html")]
struct ModelsTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    models: Vec<ConsoleModelRow>,
    total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let models = state.model().list_public(common::first_page()).await?;
    let locale = query.locale();
    common::render(&ModelsTemplate {
        view: common::view(PageId::Models, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        models: models
            .items
            .into_iter()
            .map(ConsoleModelRow::from)
            .collect(),
        total: models.total,
    })
}
