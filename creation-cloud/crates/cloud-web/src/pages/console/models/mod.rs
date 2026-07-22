//! 渲染安全模型元数据，并把各写动作拆到独立处理器。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod update;

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_model::ModelProfile;
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-models.html")]
struct ModelsTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    models: Vec<ModelProfile>,
    total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let models = state
        .model()
        .list(session.account_id, common::first_page())
        .await?;
    let locale = query.locale();
    common::render(&ModelsTemplate {
        view: common::view(PageId::Models, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        models: models.items,
        total: models.total,
    })
}
