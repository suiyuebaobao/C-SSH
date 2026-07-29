//! 全局模型目录的管理员页面。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_model::{CreateGlobalModelInput, GlobalModel, ReplaceGlobalModelInput};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;
use serde_json::Value;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

#[derive(Debug, Deserialize)]
pub(crate) struct ModelForm {
    pub(crate) name: String,
    pub(crate) provider: String,
    pub(crate) base_url: String,
    pub(crate) model_name: String,
    pub(crate) context_length: i32,
    #[serde(default)]
    pub(crate) capability_tags: String,
    #[serde(default = "default_parameters")]
    pub(crate) default_parameters: String,
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) is_default: bool,
    pub(crate) sort_order: i32,
    pub(crate) expected_revision: Option<i64>,
    pub(crate) lang: Option<String>,
}

impl ModelForm {
    pub(crate) fn into_create(self) -> AppResult<CreateGlobalModelInput> {
        Ok(CreateGlobalModelInput {
            name: self.name,
            provider: self.provider,
            base_url: shared::optional_text(self.base_url),
            model_name: self.model_name,
            context_length: self.context_length,
            capability_tags: parse_tags(&self.capability_tags),
            default_parameters: parse_parameters(&self.default_parameters)?,
            enabled: self.enabled,
            is_default: self.is_default,
            sort_order: self.sort_order,
        })
    }

    pub(crate) fn into_replace(self) -> AppResult<ReplaceGlobalModelInput> {
        let expected_revision = self
            .expected_revision
            .ok_or_else(|| AppError::Validation("缺少模型 expected_revision".to_owned()))?;
        Ok(ReplaceGlobalModelInput {
            expected_revision,
            name: self.name,
            provider: self.provider,
            base_url: shared::optional_text(self.base_url),
            model_name: self.model_name,
            context_length: self.context_length,
            capability_tags: parse_tags(&self.capability_tags),
            default_parameters: parse_parameters(&self.default_parameters)?,
            enabled: self.enabled,
            is_default: self.is_default,
            sort_order: self.sort_order,
        })
    }
}

fn parse_tags(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_parameters(value: &str) -> AppResult<Value> {
    serde_json::from_str(value.trim())
        .map_err(|_| AppError::Validation("默认参数必须是合法 JSON 对象".to_owned()))
}

fn default_parameters() -> String {
    "{}".to_owned()
}

struct ModelRow {
    id: String,
    name: String,
    provider: String,
    base_url: String,
    model_name: String,
    context_length: i32,
    capability_tags: String,
    default_parameters: String,
    enabled: bool,
    is_default: bool,
    sort_order: i32,
    revision: i64,
    updated_at: String,
}

impl From<GlobalModel> for ModelRow {
    fn from(value: GlobalModel) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            provider: value.provider,
            base_url: value.base_url.unwrap_or_default(),
            model_name: value.model_name,
            context_length: value.context_length,
            capability_tags: value.capability_tags.join(", "),
            default_parameters: value.default_parameters.to_string(),
            enabled: value.enabled,
            is_default: value.is_default,
            sort_order: value.sort_order,
            revision: value.revision,
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin-models.html")]
struct ModelsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<ModelRow>,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let (rows, load_error) = match state.model().list_admin(&actor, query.page_query()).await {
        Ok(page) => (page.items.into_iter().map(ModelRow::from).collect(), None),
        Err(_) => (
            Vec::new(),
            Some(if locale == Locale::En {
                "The model catalog is temporarily unavailable.".to_owned()
            } else {
                "模型目录暂时无法读取。".to_owned()
            }),
        ),
    };
    let parts = shared::page_parts(PageId::Admin, locale, &session);
    shared::render(&ModelsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        load_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_and_parameters_are_structured_inputs() {
        assert_eq!(
            parse_tags("vision, tools, vision"),
            ["vision", "tools", "vision"]
        );
        assert!(parse_parameters(r#"{"temperature":0.2}"#).is_ok());
        assert!(parse_parameters("not-json").is_err());
    }
}
