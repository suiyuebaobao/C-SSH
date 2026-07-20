//! 封装 Argon2id 密码哈希与校验，避免阻塞异步运行时工作线程。

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use cloud_domain::{AppError, AppResult};
use rand::RngCore;

pub(crate) async fn hash(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || hash_blocking(&password))
        .await
        .map_err(|_| AppError::Internal("密码处理任务失败".to_owned()))?
}

pub(crate) async fn verify(password: String, encoded_hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || verify_blocking(&password, &encoded_hash))
        .await
        .map_err(|_| AppError::Internal("密码处理任务失败".to_owned()))?
}

fn hash_blocking(password: &str) -> AppResult<String> {
    let mut salt_bytes = [0_u8; 16];
    rand::rng().fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|_| AppError::Internal("密码盐生成失败".to_owned()))?;
    argon2id()
        .hash_password(password.as_bytes(), &salt)
        .map(|value| value.to_string())
        .map_err(|_| AppError::Internal("密码哈希失败".to_owned()))
}

fn verify_blocking(password: &str, encoded_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(encoded_hash)
        .map_err(|_| AppError::Internal("密码哈希格式无效".to_owned()))?;
    Ok(argon2id()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn argon2id() -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
}
