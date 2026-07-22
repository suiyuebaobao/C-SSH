//! 把已认证会话的冲突解决请求映射到独立解决用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ResolveConflictOutcome, ResolveConflictRequest, Service};

pub(crate) async fn resolve_conflict(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(conflict_id): Path<Uuid>,
    Json(request): Json<ResolveConflictRequest>,
) -> AppResult<Json<ResolveConflictOutcome>> {
    service
        .resolve_conflict(&session, conflict_id, request)
        .await
        .map(Json)
}
