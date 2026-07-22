//! 仅从 AuthenticatedSession 获取身份并映射包装密钥创建请求。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{CreateVaultKeyWrapperInput, Service, VaultKeyWrapper};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateVaultKeyWrapperInput>,
) -> AppResult<(StatusCode, Json<VaultKeyWrapper>)> {
    service
        .create_wrapper(&session, input)
        .await
        .map(|wrapper| (StatusCode::CREATED, Json(wrapper)))
}
