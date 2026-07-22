//! 把已认证账号的单个冲突查询映射到查询用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, SyncConflict};

pub(crate) async fn get_conflict(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(conflict_id): Path<Uuid>,
) -> AppResult<Json<SyncConflict>> {
    service.get_conflict(&session, conflict_id).await.map(Json)
}
