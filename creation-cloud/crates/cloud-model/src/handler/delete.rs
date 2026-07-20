//! 把已认证账号的模型删除请求映射到 delete 用例。

use axum::{Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::Service;

pub(crate) async fn delete(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Path(model_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    service.delete(account_id, model_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
