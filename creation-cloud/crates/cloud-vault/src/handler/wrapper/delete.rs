//! 仅从 AuthenticatedSession 获取身份并映射包装密钥软删除请求。

use axum::{Extension, Json, extract::Path, extract::Query, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{DeleteVaultKeyWrapperInput, DeleteVaultKeyWrapperOutcome, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(wrapper_id): Path<Uuid>,
    Query(input): Query<DeleteVaultKeyWrapperInput>,
) -> AppResult<Json<DeleteVaultKeyWrapperOutcome>> {
    service
        .delete_wrapper(&session, wrapper_id, input.expected_revision)
        .await
        .map(Json)
}
