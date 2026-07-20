//! 把已认证账号的密文信封创建请求映射到 create 用例。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{CreateVaultEnvelopeInput, Service, VaultEnvelope};

pub(crate) async fn create(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Json(input): Json<CreateVaultEnvelopeInput>,
) -> AppResult<(StatusCode, Json<VaultEnvelope>)> {
    service
        .create(account_id, input)
        .await
        .map(|envelope| (StatusCode::CREATED, Json(envelope)))
}
