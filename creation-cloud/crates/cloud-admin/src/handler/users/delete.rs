//! 将管理员账号删除请求映射到永久删除用例。

use axum::{Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_user(&actor, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
