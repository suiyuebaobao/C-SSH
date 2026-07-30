//! 用户只读全局模型目录；模型配置由管理员统一维护。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

struct ConsoleModelRow {
    provider: String,
    model_name: String,
    api_format: &'static str,
    base_url: String,
}

impl From<cloud_model::GlobalModel> for ConsoleModelRow {
    fn from(value: cloud_model::GlobalModel) -> Self {
        Self {
            provider: value.provider,
            model_name: value.model_name,
            api_format: match value.api_format.as_str() {
                "anthropic_compatible" => "Claude / Anthropic",
                _ => "OpenAI",
            },
            base_url: value.base_url.unwrap_or_default(),
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
