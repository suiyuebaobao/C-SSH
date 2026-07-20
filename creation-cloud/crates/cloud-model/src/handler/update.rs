//! 把已认证账号的模型更新请求映射到 update 用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{ModelProfile, Service, UpdateModelInput};

pub(crate) async fn update(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(model_id): Path<Uuid>,
    Json(input): Json<UpdateModelInput>,
) -> AppResult<Json<ModelProfile>> {
    service.update(account_id, model_id, input).await.map(Json)
}
