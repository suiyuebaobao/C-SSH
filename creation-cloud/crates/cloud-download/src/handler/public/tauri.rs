//! 将同一策略投影为 Tauri v2平台清单；无更新时按官方合同返回 204。

use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};

use crate::{Service, model::TauriUpdateQuery};

pub(crate) async fn handle(
    State(service): State<Service>,
    query: Result<Query<TauriUpdateQuery>, QueryRejection>,
) -> AppResult<Response> {
    let Query(query) = query.map_err(|_| AppError::Validation("Tauri 更新查询参数无效".into()))?;
    match service.tauri_update(query).await? {
        Some(response) => Ok(Json(response).into_response()),
        None => Ok(axum::http::StatusCode::NO_CONTENT.into_response()),
    }
}
