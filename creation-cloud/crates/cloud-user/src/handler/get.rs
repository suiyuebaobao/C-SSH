//! 从路径读取资料 ID 并返回当前账号拥有的资料。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::{Profile, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
) -> AppResult<Json<Profile>> {
    Ok(Json(service.get(&session, account_id).await?))
}
