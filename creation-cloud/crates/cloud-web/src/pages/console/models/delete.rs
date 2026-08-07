//! 删除本人模型元数据，不触碰被引用的保险库密文。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct DeleteModelForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteModelForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state.model().delete(session.account_id, model_id).await?;
    Ok(common::action_success(&headers, PageId::Models, locale))
}
