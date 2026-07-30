//! 匿名签发管理员登录 CAPTCHA 图像，禁止缓存并用 HttpOnly Cookie 绑定浏览器挑战。

use axum::{
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};

use crate::{Service, cookie};

const CAPTCHA_ID_HEADER: &str = "x-admin-captcha-id";

pub(crate) async fn handle(State(service): State<Service>) -> AppResult<Response> {
    let issued = service.issue_admin_captcha().await?;
    let mut response = (StatusCode::OK, issued.svg).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    response
        .headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    response.headers_mut().insert(
        CAPTCHA_ID_HEADER,
        HeaderValue::from_str(&issued.challenge_id.to_string())
            .map_err(|_| AppError::Internal("图形验证码标识响应无效".to_owned()))?,
    );
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::admin_captcha_header(issued.challenge_id, issued.expires_at)?,
    );
    Ok(response)
}
