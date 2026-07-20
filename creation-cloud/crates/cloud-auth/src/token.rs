//! 生成高熵会话令牌并派生数据库哈希与独立 CSRF 令牌。

use cloud_domain::{AppError, AppResult};
use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const TOKEN_BYTES: usize = 32;
const CSRF_CONTEXT: &[u8] = b"creation-cloud-csrf-v1\0";

pub(crate) fn issue() -> (String, Vec<u8>) {
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let raw_token = hex::encode(bytes);
    let token_hash = Sha256::digest(raw_token.as_bytes()).to_vec();
    (raw_token, token_hash)
}

pub(crate) fn hash(raw_token: &str) -> AppResult<Vec<u8>> {
    let decoded = hex::decode(raw_token)
        .map_err(|_| AppError::Unauthorized("会话无效或已过期".to_owned()))?;
    if decoded.len() != TOKEN_BYTES {
        return Err(AppError::Unauthorized("会话无效或已过期".to_owned()));
    }
    Ok(Sha256::digest(raw_token.as_bytes()).to_vec())
}

pub(crate) fn csrf(raw_token: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(CSRF_CONTEXT);
    digest.update(raw_token.as_bytes());
    hex::encode(digest.finalize())
}

pub(crate) fn csrf_matches(expected: &str, supplied: &str) -> bool {
    expected.len() == supplied.len() && bool::from(expected.as_bytes().ct_eq(supplied.as_bytes()))
}
