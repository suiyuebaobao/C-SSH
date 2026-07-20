//! 把已认证账号的密文信封墓碑请求映射到 delete 用例。

use axum::{Extension, Json, extract::Path, extract::Query, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{DeleteVaultInput, DeleteVaultOutcome, Service};

pub(crate) async fn delete(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(envelope_id): Path<Uuid>,
    Query(input): Query<DeleteVaultInput>,
) -> AppResult<Json<DeleteVaultOutcome>> {
    service
        .delete(account_id, envelope_id, input.expected_revision)
        .await
        .map(Json)
}
