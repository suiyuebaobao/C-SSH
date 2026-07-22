//! 创建不含凭据的模型元数据，敏感字段仍由模型域校验拒绝。

use std::collections::BTreeMap;

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_model::CreateModelInput;
use cloud_site::PageId;
use serde::Deserialize;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct CreateModelForm {
    name: String,
    provider: String,
    base_url: String,
    model_name: String,
    context_length: i32,
    enabled: Option<String>,
    is_default: Option<String>,
    sort_order: i32,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateModelForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state
        .model()
        .create(
            session.account_id,
            CreateModelInput {
                name: form.name,
                provider: form.provider,
                base_url: Some(form.base_url),
                model_name: form.model_name,
                context_length: form.context_length,
                capability_tags: Vec::new(),
                default_parameters: serde_json::json!({}),
                enabled: form.enabled.is_some(),
                is_default: form.is_default.is_some(),
                sort_order: form.sort_order,
                vault_envelope_id: None,
                extra_fields: BTreeMap::new(),
            },
        )
        .await?;
    Ok(common::action_success(&headers, PageId::Models, locale))
}
