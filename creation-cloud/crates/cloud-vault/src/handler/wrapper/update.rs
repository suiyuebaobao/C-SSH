//! 仅从 AuthenticatedSession 获取身份并映射包装密钥乐观锁更新请求。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, UpdateVaultKeyWrapperInput, VaultKeyWrapper};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(wrapper_id): Path<Uuid>,
    Json(input): Json<UpdateVaultKeyWrapperInput>,
) -> AppResult<Json<VaultKeyWrapper>> {
    service
        .update_wrapper(&session, wrapper_id, input)
        .await
        .map(Json)
}
