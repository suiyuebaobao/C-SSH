//! 把已认证账号的模型创建请求映射到 create 用例。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{CreateModelInput, ModelProfile, Service};

pub(crate) async fn create(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Json(input): Json<CreateModelInput>,
) -> AppResult<(StatusCode, Json<ModelProfile>)> {
    service
        .create(account_id, input)
        .await
        .map(|profile| (StatusCode::CREATED, Json(profile)))
}
