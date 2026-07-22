//! 通过已认证会话授权下载并把最小事件归属到当前账号。

use axum::{
    Extension,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, header::RANGE},
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((asset_id, source_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> AppResult<Response<Body>> {
    let range = headers
        .get(RANGE)
        .map(|value| value.to_str())
        .transpose()
        .map_err(|_| AppError::Validation("Range 请求头不是有效文本".into()))?;
    service
        .serve_account_download(&session, asset_id, source_id, range)
        .await
}
