//! 返回当前已发布首页二维码的同源公开元数据。

use axum::{
    Json,
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::Service;

pub(crate) async fn handle(State(service): State<Service>) -> AppResult<Response> {
    let media = service.current_home_qr().await?;
    let mut response = Json(media).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=60, must-revalidate"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
