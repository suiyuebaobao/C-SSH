//! 负责安全会话 Cookie 的写入、清除和原始令牌提取。

use axum::http::{HeaderMap, HeaderValue, header};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

const SESSION_COOKIE: &str = "creation_session";
const PASSWORD_RESET_EMAIL_COOKIE: &str = "creation_password_reset_email";
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

pub(crate) fn captcha_header(
    purpose: crate::captcha::CaptchaPurpose,
    challenge_id: Uuid,
    expires_at: DateTime<Utc>,
) -> AppResult<HeaderValue> {
    let max_age_seconds = expires_at.signed_duration_since(Utc::now()).num_seconds();
    if max_age_seconds <= 0 {
        return Err(AppError::Internal("图形验证码已在响应前过期".to_owned()));
    }
    header_value(&format!(
        "{}={challenge_id}; Path={COOKIE_PATH}; Max-Age={max_age_seconds}; Secure; HttpOnly; SameSite=Strict",
        captcha_cookie_name(purpose)
    ))
}

pub(crate) fn clear_captcha_header(
    purpose: crate::captcha::CaptchaPurpose,
) -> AppResult<HeaderValue> {
    header_value(&format!(
        "{}=; Path={COOKIE_PATH}; Max-Age=0; Secure; HttpOnly; SameSite=Strict",
        captcha_cookie_name(purpose)
    ))
}

pub(crate) fn password_reset_email_header(normalized_email: &str) -> AppResult<HeaderValue> {
    let email = crate::validation::normalize_email(normalized_email)?;
    let value = hex::encode(email.as_bytes());
    let max_age_seconds = crate::verification::CODE_TTL_MINUTES * 60;
    header_value(&format!(
        "{PASSWORD_RESET_EMAIL_COOKIE}={value}; Path={COOKIE_PATH}; Max-Age={max_age_seconds}; Secure; HttpOnly; SameSite=Strict"
    ))
}

pub(crate) fn clear_password_reset_email_header() -> AppResult<HeaderValue> {
    header_value(&format!(
        "{PASSWORD_RESET_EMAIL_COOKIE}=; Path={COOKIE_PATH}; Max-Age=0; Secure; HttpOnly; SameSite=Strict"
    ))
}

pub(crate) fn read_password_reset_email(headers: &HeaderMap) -> Option<String> {
    let encoded = read_named(headers, PASSWORD_RESET_EMAIL_COOKIE)?;
    let decoded = hex::decode(encoded).ok()?;
    let email = String::from_utf8(decoded).ok()?;
    let normalized = crate::validation::normalize_email(&email).ok()?;
    (normalized == email).then_some(normalized)
}

pub(crate) fn read_captcha(
    headers: &HeaderMap,
    purpose: crate::captcha::CaptchaPurpose,
) -> Option<Uuid> {
    read_named(headers, captcha_cookie_name(purpose)).and_then(|value| Uuid::parse_str(&value).ok())
}

const fn captcha_cookie_name(purpose: crate::captcha::CaptchaPurpose) -> &'static str {
    match purpose {
        crate::captcha::CaptchaPurpose::Register => "creation_captcha_register",
        crate::captcha::CaptchaPurpose::Login => "creation_captcha_login",
        crate::captcha::CaptchaPurpose::AdminLogin => "creation_captcha_admin",
        crate::captcha::CaptchaPurpose::PasswordReset => "creation_captcha_password_reset",
    }
}

pub(crate) fn read(headers: &HeaderMap) -> AppResult<String> {
    read_named(headers, SESSION_COOKIE)
        .ok_or_else(|| AppError::Unauthorized("需要有效会话".to_owned()))
}

fn read_named(headers: &HeaderMap, expected_name: &str) -> Option<String> {
    for value in headers.get_all(header::COOKIE) {
        let Ok(value) = value.to_str() else {
            continue;
        };
        for pair in value.split(';') {
            let Some((name, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if name == expected_name && !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

fn header_value(value: &str) -> AppResult<HeaderValue> {
    HeaderValue::from_str(value).map_err(|_| AppError::Internal("会话 Cookie 构造失败".to_owned()))
}
