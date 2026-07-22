//! 对已完成类型校验的同步请求生成稳定 SHA-256 指纹。

use cloud_domain::{AppError, AppResult};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) fn json<T: Serialize>(value: &T, message: &'static str) -> AppResult<String> {
    let encoded = serde_json::to_vec(value).map_err(|_| AppError::Internal(message.to_owned()))?;
    Ok(hex::encode(Sha256::digest(encoded)))
}
