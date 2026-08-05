//! 校验匿名查询并返回与公开启动 API 一致的更新结果。

use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
};
use cloud_domain::{AppError, AppResult};

use crate::{Service, UpdateCheckQuery, UpdateCheckResponse};

pub(crate) async fn handle(
    State(service): State<Service>,
    query: Result<Query<UpdateCheckQuery>, QueryRejection>,
) -> AppResult<Json<UpdateCheckResponse>> {
    let Query(query) = query.map_err(|_| AppError::Validation("更新检查查询参数无效".into()))?;
    Ok(Json(service.check_update(query).await?))
}
