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
use cloud_model::{
    CreateGlobalModelInput, GlobalModel, ModelInterface, ReasoningControl, ReplaceGlobalModelInput,
};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;
use serde_json::json;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

#[derive(Debug, Deserialize)]
pub(crate) struct ModelForm {
    pub(crate) name: String,
    pub(crate) provider: String,
    pub(crate) context_length: i32,
    pub(crate) openai_base_url: String,
    pub(crate) openai_model_name: String,
    pub(crate) anthropic_base_url: String,
    pub(crate) anthropic_model_name: String,
    pub(crate) responses_base_url: String,
    pub(crate) responses_model_name: String,
    #[serde(default)]
    pub(crate) enabled: bool,
    pub(crate) expected_revision: Option<i64>,
    pub(crate) lang: Option<String>,
}

impl ModelForm {
    pub(crate) fn into_create(self) -> AppResult<CreateGlobalModelInput> {
        let interfaces = form_interfaces(
            self.openai_base_url,
            self.openai_model_name,
            self.anthropic_base_url,
            self.anthropic_model_name,
            self.responses_base_url,
            self.responses_model_name,
        )?;
        Ok(CreateGlobalModelInput {
            name: self.name,
            provider: self.provider,
            context_length: self.context_length,
            interfaces,
            reasoning_control: ReasoningControl::Unsupported,
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
        let interfaces = form_interfaces(
            self.openai_base_url,
            self.openai_model_name,
            self.anthropic_base_url,
            self.anthropic_model_name,
            self.responses_base_url,
            self.responses_model_name,
        )?;
        Ok(ReplaceGlobalModelInput {
            expected_revision,
            name: self.name,
            provider: self.provider,
            context_length: self.context_length,
            interfaces,
            reasoning_control: ReasoningControl::Unsupported,
            capability_tags: current.capability_tags.clone(),
            default_parameters: current.default_parameters.clone(),
            enabled: self.enabled,
            is_default: current.is_default,
            sort_order: current.sort_order,
        })
    }
}

fn form_interfaces(
    openai_base_url: String,
    openai_model_name: String,
    anthropic_base_url: String,
    anthropic_model_name: String,
    responses_base_url: String,
    responses_model_name: String,
) -> AppResult<Vec<ModelInterface>> {
    let mut interfaces = Vec::with_capacity(3);
    push_form_interface(
        &mut interfaces,
        "openai_compatible",
        openai_base_url,
        openai_model_name,
        "OpenAI",
    )?;
    push_form_interface(
        &mut interfaces,
        "anthropic_compatible",
        anthropic_base_url,
        anthropic_model_name,
        "Anthropic",
    )?;
    push_form_interface(
        &mut interfaces,
        "responses_compatible",
        responses_base_url,
        responses_model_name,
        "Responses",
    )?;
    if interfaces.is_empty() {
        return Err(AppError::Validation("至少配置一种模型接口".to_owned()));
    }
    Ok(interfaces)
}

fn push_form_interface(
    interfaces: &mut Vec<ModelInterface>,
    api_format: &str,
    base_url: String,
    model_name: String,
    label: &str,
) -> AppResult<()> {
    match (
        shared::optional_text(base_url),
        shared::optional_text(model_name),
    ) {
        (None, None) => Ok(()),
        (Some(base_url), Some(model_name)) => {
            interfaces.push(ModelInterface {
                api_format: api_format.to_owned(),
                base_url,
                model_name,
            });
            Ok(())
        }
        _ => Err(AppError::Validation(format!(
            "{label} 接口的 API 地址和 model ID 必须同时填写"
        ))),
    }
}

struct ModelRow {
    id: String,
    name: String,
    provider: String,
    context_length: i32,
    openai_base_url: String,
    openai_model_name: String,
    anthropic_base_url: String,
    anthropic_model_name: String,
    responses_base_url: String,
    responses_model_name: String,
    interface_count: usize,
    enabled: bool,
    revision: i64,
}

impl From<GlobalModel> for ModelRow {
    fn from(value: GlobalModel) -> Self {
        let interface_count = value.interfaces.len();
        let openai = value
            .interfaces
            .iter()
            .find(|item| item.api_format == "openai_compatible");
        let anthropic = value
            .interfaces
            .iter()
            .find(|item| item.api_format == "anthropic_compatible");
        let responses = value
            .interfaces
            .iter()
            .find(|item| item.api_format == "responses_compatible");
        Self {
            id: value.id.to_string(),
            name: value.name,
            provider: value.provider,
            context_length: value.context_length,
            openai_base_url: openai.map_or_else(String::new, |item| item.base_url.clone()),
            openai_model_name: openai.map_or_else(String::new, |item| item.model_name.clone()),
            anthropic_base_url: anthropic.map_or_else(String::new, |item| item.base_url.clone()),
            anthropic_model_name: anthropic
                .map_or_else(String::new, |item| item.model_name.clone()),
            responses_base_url: responses.map_or_else(String::new, |item| item.base_url.clone()),
            responses_model_name: responses
                .map_or_else(String::new, |item| item.model_name.clone()),
            interface_count,
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
            name: "DeepSeek V4 Pro".to_owned(),
            provider: "DeepSeek".to_owned(),
            context_length: 1_000_000,
            openai_base_url: "https://api.deepseek.com".to_owned(),
            openai_model_name: "deepseek-v4-pro".to_owned(),
            anthropic_base_url: "https://api.deepseek.com/anthropic".to_owned(),
            anthropic_model_name: "deepseek-v4-pro".to_owned(),
            responses_base_url: "https://api.deepseek.com".to_owned(),
            responses_model_name: "deepseek-v4-pro".to_owned(),
            enabled: true,
            expected_revision: None,
            lang: None,
        }
        .into_create()
        .expect("极简表单应生成模型输入");
        assert_eq!(input.name, "DeepSeek V4 Pro");
        assert_eq!(input.context_length, 1_000_000);
        assert_eq!(input.interfaces.len(), 3);
        assert_eq!(input.interfaces[0].model_name, "deepseek-v4-pro");
        assert_eq!(input.reasoning_control, ReasoningControl::Unsupported);
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
                name: "Example model<script>".to_owned(),
                provider: "example-provider".to_owned(),
                context_length: 128_000,
                openai_base_url: "https://api.example.com".to_owned(),
                openai_model_name: "example-model<script>".to_owned(),
                anthropic_base_url: "https://api.example.com/anthropic".to_owned(),
                anthropic_model_name: "example-anthropic".to_owned(),
                responses_base_url: "https://api.example.com/v1".to_owned(),
                responses_model_name: "example-responses".to_owned(),
                interface_count: 3,
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
        assert!(body.contains("name=\"openai_base_url\""));
        assert!(body.contains("name=\"openai_model_name\""));
        assert!(body.contains("name=\"anthropic_base_url\""));
        assert!(body.contains("name=\"anthropic_model_name\""));
        assert!(body.contains("name=\"responses_base_url\""));
        assert!(body.contains("name=\"responses_model_name\""));
        assert!(!body.contains("name=\"reasoning_control\""));
        assert!(!body.contains("Controllable thinking"));
        assert!(!body.contains("可控思考"));
        assert!(!body.contains("V3.2"));
        assert!(!body.contains("K2.5"));
        assert!(!body.contains("Example model&lt;script&gt;（"));
        assert!(body.contains("openai_compatible"));
        assert!(body.contains("anthropic_compatible"));
        assert!(body.contains("responses_compatible"));
        assert!(!body.contains("example-model<script>"));
        assert!(body.contains("name=\"context_length\""));
        assert!(!body.contains("name=\"capability_tags\""));
        assert!(!body.contains("name=\"default_parameters\""));
        assert!(!body.contains("name=\"sort_order\""));
        assert!(!body.contains("name=\"is_default\""));
        assert!(!body.contains("name=\"api_key\""));
        assert!(!body.contains("admin-seo-record"));
        assert!(body.contains("Only the global model catalog is managed here."));
        assert!(body.contains("client-encrypted opaque account records"));
        assert!(
            !body
                .to_ascii_lowercase()
                .contains("api keys reference ciphertext")
        );
    }
}
