//! 返回已发布首页二维码的受控 PNG 内容并禁止浏览器 MIME 嗅探。

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, header},
};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Path(media_id): Path<Uuid>,
) -> AppResult<Response<Body>> {
    let content = service.content(media_id).await?;
    let etag = HeaderValue::from_str(&format!("\"{}\"", content.sha256))
        .map_err(|_| AppError::Internal("站点媒体 ETag 无效".into()))?;
    let length = HeaderValue::from_str(&content.bytes.len().to_string())
        .map_err(|_| AppError::Internal("站点媒体长度无效".into()))?;
    let mut response = Response::new(Body::from(content.bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    headers.insert(header::CONTENT_LENGTH, length);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(header::ETAG, etag);
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
