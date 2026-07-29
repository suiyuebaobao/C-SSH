//! 设备绑定时生成新会话令牌、CSRF 派生值和持久 Cookie。

use axum::http::HeaderValue;
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use rand::RngCore;
use sha2::{Digest, Sha256};

const TOKEN_BYTES: usize = 32;
const CSRF_CONTEXT: &[u8] = b"creation-cloud-csrf-v1\0";

pub(crate) fn issue() -> (String, Vec<u8>) {
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let raw = hex::encode(bytes);
    let hash = Sha256::digest(raw.as_bytes()).to_vec();
    (raw, hash)
}

pub(crate) fn csrf(raw: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(CSRF_CONTEXT);
    digest.update(raw.as_bytes());
    hex::encode(digest.finalize())
}

pub(crate) fn cookie(raw: &str, idle_expires_at: DateTime<Utc>) -> AppResult<HeaderValue> {
    let max_age = idle_expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    if max_age <= 0 {
        return Err(AppError::Internal("设备会话已在响应前过期".to_owned()));
    }
    HeaderValue::from_str(&format!(
        "creation_session={raw}; Path=/; Max-Age={max_age}; Secure; HttpOnly; SameSite=Strict"
    ))
    .map_err(|_| AppError::Internal("设备会话 Cookie 构造失败".to_owned()))
}
