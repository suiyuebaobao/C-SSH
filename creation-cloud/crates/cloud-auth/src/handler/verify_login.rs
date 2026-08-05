//! 校验普通用户或管理员登录邮箱验证码，成功后才签发会话 Cookie。

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Service, VerifyLogin, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(command): Json<VerifyLogin>,
) -> AppResult<Response> {
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    let issued = service
        .verify_login_with_metadata(command, &metadata)
        .await?;
    let mut response = Json(issued.view()).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
    );
    Ok(response)
}
