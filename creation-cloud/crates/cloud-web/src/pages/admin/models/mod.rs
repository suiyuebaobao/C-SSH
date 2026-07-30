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
use serde_json::json;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

#[derive(Debug, Deserialize)]
pub(crate) struct ModelForm {
    pub(crate) provider: String,
    pub(crate) api_format: String,
    pub(crate) base_url: String,
    pub(crate) model_name: String,
    #[serde(default)]
    pub(crate) enabled: bool,
    pub(crate) expected_revision: Option<i64>,
    pub(crate) lang: Option<String>,
}

impl ModelForm {
    pub(crate) fn into_create(self) -> AppResult<CreateGlobalModelInput> {
        let model_name = self.model_name;
        Ok(CreateGlobalModelInput {
            name: model_name.clone(),
            provider: self.provider,
            api_format: self.api_format,
            base_url: shared::optional_text(self.base_url),
            model_name,
            context_length: 128_000,
            capability_tags: Vec::new(),
            default_parameters: json!({}),
            enabled: self.enabled,
            is_default: false,
            sort_order: 0,
        })
    }

    pub(crate) fn into_replace(self, current: &GlobalModel) -> AppResult<ReplaceGlobalModelInput> {
        let expected_revision = self
            .expected_revision
            .ok_or_else(|| AppError::Validation("缺少模型 expected_revision".to_owned()))?;
        let model_name = self.model_name;
        Ok(ReplaceGlobalModelInput {
            expected_revision,
            name: model_name.clone(),
            provider: self.provider,
            api_format: self.api_format,
            base_url: shared::optional_text(self.base_url),
            model_name,
            context_length: current.context_length,
            capability_tags: current.capability_tags.clone(),
            default_parameters: current.default_parameters.clone(),
            enabled: self.enabled,
            is_default: current.is_default,
            sort_order: current.sort_order,
        })
    }
}

struct ModelRow {
    id: String,
    provider: String,
    api_format: String,
    api_format_label: &'static str,
    base_url: String,
    model_name: String,
    enabled: bool,
    revision: i64,
}

impl From<GlobalModel> for ModelRow {
    fn from(value: GlobalModel) -> Self {
        let api_format_label = match value.api_format.as_str() {
            "anthropic_compatible" => "Claude / Anthropic",
            _ => "OpenAI",
        };
        Self {
            id: value.id.to_string(),
            provider: value.provider,
            api_format: value.api_format,
            api_format_label,
            base_url: value.base_url.unwrap_or_default(),
            model_name: value.model_name,
            enabled: value.enabled,
            revision: value.revision,
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
    let parts = shared::page_parts(PageId::AdminModels, locale, &session);
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
    use cloud_site::content_service;

    #[test]
    fn create_form_uses_safe_internal_defaults() {
        let input = ModelForm {
            provider: "DeepSeek".to_owned(),
            api_format: "openai_compatible".to_owned(),
            base_url: "https://api.deepseek.com".to_owned(),
            model_name: "deepseek-v4-pro".to_owned(),
            enabled: true,
            expected_revision: None,
            lang: None,
        }
        .into_create()
        .expect("极简表单应生成模型输入");
        assert_eq!(input.name, "deepseek-v4-pro");
        assert_eq!(input.model_name, "deepseek-v4-pro");
        assert_eq!(input.api_format, "openai_compatible");
        assert_eq!(input.context_length, 128_000);
        assert!(input.capability_tags.is_empty());
        assert_eq!(input.default_parameters, json!({}));
        assert!(!input.is_default);
    }

    #[test]
    fn template_keeps_model_actions_in_simple_expandable_rows() {
        let id = "01917f21-9f82-7ca4-b1dd-034518738965";
        let body = ModelsTemplate {
            view: content_service().view(PageId::AdminModels, Locale::En),
            seo: SeoHead::private(),
            session_identity: Some("ops-admin".to_owned()),
            csrf_token: "csrf-example".to_owned(),
            is_en: true,
            rows: vec![ModelRow {
                id: id.to_owned(),
                provider: "example-provider".to_owned(),
                api_format: "openai_compatible".to_owned(),
                api_format_label: "OpenAI",
                base_url: "https://api.example.com".to_owned(),
                model_name: "example-model<script>".to_owned(),
                enabled: true,
                revision: 7,
            }],
            load_error: None,
        }
        .render()
        .expect("模型管理模板应可渲染");

        assert!(body.contains("<details class=\"admin-model-create\">"));
        assert!(body.contains(&format!(
            "<details class=\"admin-model-row\" id=\"model-{id}\">"
        )));
        assert!(body.contains("hx-post=\"/admin/models\""));
        assert!(body.contains(&format!("hx-post=\"/admin/models/{id}\"")));
        assert!(body.contains(&format!("hx-post=\"/admin/models/{id}/delete\"")));
        assert!(body.contains("name=\"expected_revision\" value=\"7\""));
        assert!(body.contains("data-csrf-token=\"csrf-example\""));
        assert!(body.contains("name=\"api_format\""));
        assert!(body.contains("openai_compatible"));
        assert!(body.contains("anthropic_compatible"));
        assert!(!body.contains("example-model<script>"));
        assert!(!body.contains("name=\"context_length\""));
        assert!(!body.contains("name=\"capability_tags\""));
        assert!(!body.contains("name=\"default_parameters\""));
        assert!(!body.contains("name=\"sort_order\""));
        assert!(!body.contains("name=\"is_default\""));
        assert!(!body.contains("name=\"api_key\""));
        assert!(!body.contains("admin-seo-record"));
    }
}
