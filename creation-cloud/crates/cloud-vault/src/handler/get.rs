//! 把已认证账号的单个密文信封查询映射到 get 用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, VaultEnvelope};

pub(crate) async fn get(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(envelope_id): Path<Uuid>,
) -> AppResult<Json<VaultEnvelope>> {
    service.get(account_id, envelope_id).await.map(Json)
}
