//! 更新模型显示、默认、启停和顺序元数据，不接收 API Key 等凭据。

use std::collections::BTreeMap;

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_model::UpdateModelInput;
use cloud_site::PageId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct UpdateModelForm {
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
    Path(model_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateModelForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state
        .model()
        .update(
            session.account_id,
            model_id,
            UpdateModelInput {
                name: Some(form.name),
                provider: Some(form.provider),
                base_url: Some(form.base_url),
                model_name: Some(form.model_name),
                context_length: Some(form.context_length),
                capability_tags: None,
                default_parameters: None,
                enabled: Some(form.enabled.is_some()),
                is_default: Some(form.is_default.is_some()),
                sort_order: Some(form.sort_order),
                vault_envelope_id: None,
                clear_vault_envelope: false,
                extra_fields: BTreeMap::new(),
            },
        )
        .await?;
    Ok(common::action_success(&headers, PageId::Models, locale))
}
