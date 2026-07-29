//! 接收登录 JSON，写入安全会话 Cookie 并返回会话视图。

use axum::{
    Json,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Login, Service, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<Login>,
) -> AppResult<Response> {
    let issued = service.login(command).await?;
    let mut response = Json(issued.view()).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
    );
    Ok(response)
}
