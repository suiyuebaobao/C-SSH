//! 将受限角色状态输入映射到带管理员身份的用户更新用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminUpdateUserInput, AdminUser, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Json(input): Json<AdminUpdateUserInput>,
) -> AppResult<Json<AdminUser>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update_user(&actor, account_id, input).await?))
}
