//! 修改密码后撤销旧版本会话并通过响应 Cookie 签发当前新会话。

use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{AuthenticatedSession, ChangePassword, Service, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Json(command): Json<ChangePassword>,
) -> AppResult<Response> {
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    let issued = service
        .change_password_with_metadata(&session, command, &metadata)
        .await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
    );
    Ok(response)
}
