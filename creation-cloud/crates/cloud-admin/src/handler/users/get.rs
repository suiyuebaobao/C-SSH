//! 从服务端会话派生管理员身份并返回单个脱敏用户。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminUser, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
) -> AppResult<Json<AdminUser>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_user(&actor, account_id).await?))
}
