//! 暴露已有管理员认证链下的只读维护状态 API，不提供远程任务触发入口。

use std::str::FromStr;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    routing::get,
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};

use crate::{MaintenanceStatus, MaintenanceTask, Service};

#[must_use = "路由必须挂载到已有管理员认证链才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list))
        .route("/{task}", get(get_one))
        .with_state(service)
}

async fn list(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<Vec<MaintenanceStatus>>> {
    AdminActor::from_session(&session)?;
    Ok(Json(service.statuses().await?))
}

async fn get_one(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(task): Path<String>,
) -> AppResult<Json<MaintenanceStatus>> {
    AdminActor::from_session(&session)?;
    let task = MaintenanceTask::from_str(&task)
        .map_err(|()| AppError::NotFound("维护任务不存在".to_owned()))?;
    Ok(Json(service.status(task).await?))
}
