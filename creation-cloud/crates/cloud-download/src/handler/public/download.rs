//! 将 Range 请求交给下载用例并返回重定向或流式响应。

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, header::RANGE},
};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Path((asset_id, source_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> AppResult<Response<Body>> {
    let range = headers
        .get(RANGE)
        .map(|value| value.to_str())
        .transpose()
        .map_err(|_| AppError::Validation("Range 请求头不是有效文本".into()))?;
    service.serve_download(asset_id, source_id, range).await
}
