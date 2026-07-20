//! 从路径读取资料 ID 并返回无内容删除结果。

use axum::{Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    service.delete(&session, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
