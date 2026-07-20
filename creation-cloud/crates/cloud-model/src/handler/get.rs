//! 把已认证账号的单个模型查询映射到 get 用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{ModelProfile, Service};

pub(crate) async fn get(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(model_id): Path<Uuid>,
) -> AppResult<Json<ModelProfile>> {
    service.get(account_id, model_id).await.map(Json)
}
