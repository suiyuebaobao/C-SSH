//! 负责安全会话 Cookie 的写入、清除和原始令牌提取。

use axum::http::{HeaderMap, HeaderValue, header};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};

const SESSION_COOKIE: &str = "creation_session";
const COOKIE_PATH: &str = "/";

pub(crate) fn session_header(raw_token: &str, expires_at: DateTime<Utc>) -> AppResult<HeaderValue> {
    session_header_at(raw_token, expires_at, Utc::now())
}

pub(crate) fn session_header_at(
    raw_token: &str,
    expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> AppResult<HeaderValue> {
    // Max-Age 从数据库会话的绝对过期时间计算，避免浏览器 Cookie 活得比服务端会话更久。
    let max_age_seconds = expires_at.signed_duration_since(now).num_seconds();
    if max_age_seconds <= 0 {
        return Err(AppError::Internal("会话已在响应前过期".to_owned()));
    }
    header_value(&format!(
        "{SESSION_COOKIE}={raw_token}; Path={COOKIE_PATH}; Max-Age={max_age_seconds}; Secure; HttpOnly; SameSite=Strict"
    ))
}

pub(crate) fn clear_header() -> AppResult<HeaderValue> {
    header_value(&format!(
        "{SESSION_COOKIE}=; Path={COOKIE_PATH}; Max-Age=0; Secure; HttpOnly; SameSite=Strict"
    ))
}

pub(crate) fn read(headers: &HeaderMap) -> AppResult<String> {
    for value in headers.get_all(header::COOKIE) {
        let Ok(value) = value.to_str() else {
            continue;
        };
        for pair in value.split(';') {
            let Some((name, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if name == SESSION_COOKIE && !value.is_empty() {
                return Ok(value.to_owned());
            }
        }
    }
    Err(AppError::Unauthorized("需要有效会话".to_owned()))
}

fn header_value(value: &str) -> AppResult<HeaderValue> {
    HeaderValue::from_str(value).map_err(|_| AppError::Internal("会话 Cookie 构造失败".to_owned()))
}
