//! 把已认证账号的密文信封替换请求映射到 update 用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, UpdateVaultEnvelopeInput, VaultEnvelope};

pub(crate) async fn update(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(envelope_id): Path<Uuid>,
    Json(input): Json<UpdateVaultEnvelopeInput>,
) -> AppResult<Json<VaultEnvelope>> {
    service
        .update(account_id, envelope_id, input)
        .await
        .map(Json)
}
