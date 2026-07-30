//! 匿名签发认证 CAPTCHA 图像，禁止缓存并用 HttpOnly Cookie 绑定浏览器挑战。

use axum::{
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};
use serde::Deserialize;

use crate::{Service, captcha::CaptchaPurpose, cookie};

const CAPTCHA_ID_HEADER: &str = "x-auth-captcha-id";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptchaQuery {
    purpose: String,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    Query(query): Query<CaptchaQuery>,
) -> AppResult<Response> {
    let purpose = CaptchaPurpose::parse(&query.purpose)
        .ok_or_else(|| AppError::Validation("图形验证码用途无效".to_owned()))?;
    let issued = service.issue_captcha(purpose).await?;
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
        cookie::captcha_header(purpose, issued.challenge_id, issued.expires_at)?,
    );
    Ok(response)
}
