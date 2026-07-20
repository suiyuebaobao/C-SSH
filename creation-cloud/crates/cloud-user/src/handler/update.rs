//! 从路径与 JSON 提取资料变更并调用更新用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::{Profile, Service, UpdateProfile};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Json(command): Json<UpdateProfile>,
) -> AppResult<Json<Profile>> {
    Ok(Json(service.update(&session, account_id, command).await?))
}
