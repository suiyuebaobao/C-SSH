//! 校验账号级 master-key 包装密文，并拒绝未知字段与明文密钥材料。

use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    CreateVaultKeyWrapperInput, UpdateVaultKeyWrapperInput,
    validation::{self, decode_bounded},
    wrapper_types::{CreateKeyWrapper, UpdateKeyWrapper},
};

const WRAPPED_MASTER_KEY_BYTES: usize = 48;

pub(crate) fn account(session: &AuthenticatedSession) -> AppResult<Uuid> {
    validation::account(session.account_id)?;
    Ok(session.account_id)
}

pub(crate) fn wrapper_id(wrapper_id: Uuid) -> AppResult<()> {
    if wrapper_id.is_nil() {
        return Err(AppError::Validation("包装密钥标识不能为空".to_owned()));
    }
    Ok(())
}

pub(crate) fn create(input: CreateVaultKeyWrapperInput) -> AppResult<CreateKeyWrapper> {
    reject_extra_fields(&input.extra_fields)?;
    if input.wrapper_key.is_nil() {
        return Err(AppError::Validation("wrapper_key 不能为空".to_owned()));
    }
    validation::versions(input.schema_version, input.key_version)?;
    let cipher_suite = validation::cipher_suite(input.cipher_suite)?;
    let kdf = validation::kdf(input.kdf)?;
    let kdf_salt = STANDARD
        .decode(&kdf.salt)
        .map_err(|_| AppError::Internal("无法读取已校验的 KDF salt".to_owned()))?;
    let nonce = decode_bounded("nonce", &input.nonce, 24, 24)?;
    let wrapped_master_key = decode_bounded(
        "wrapped_master_key",
        &input.wrapped_master_key,
        WRAPPED_MASTER_KEY_BYTES,
        WRAPPED_MASTER_KEY_BYTES,
    )?;
    Ok(CreateKeyWrapper {
        id: Uuid::now_v7(),
        wrapper_key: input.wrapper_key,
        schema_version: input.schema_version,
        key_version: input.key_version,
        cipher_suite,
        kdf_algorithm: kdf.algorithm,
        kdf_salt,
        kdf_memory_kib: kdf.memory_kib,
        kdf_iterations: kdf.iterations,
        kdf_parallelism: kdf.parallelism,
        nonce,
        wrapped_master_key,
    })
}

pub(crate) fn update(input: UpdateVaultKeyWrapperInput) -> AppResult<UpdateKeyWrapper> {
    reject_extra_fields(&input.extra_fields)?;
    validation::revision(input.expected_revision)?;
    validation::versions(input.schema_version, input.key_version)?;
    let cipher_suite = validation::cipher_suite(input.cipher_suite)?;
    let kdf = validation::kdf(input.kdf)?;
    let kdf_salt = STANDARD
        .decode(&kdf.salt)
        .map_err(|_| AppError::Internal("无法读取已校验的 KDF salt".to_owned()))?;
    let nonce = decode_bounded("nonce", &input.nonce, 24, 24)?;
    let wrapped_master_key = decode_bounded(
        "wrapped_master_key",
        &input.wrapped_master_key,
        WRAPPED_MASTER_KEY_BYTES,
        WRAPPED_MASTER_KEY_BYTES,
    )?;
    Ok(UpdateKeyWrapper {
        expected_revision: input.expected_revision,
        schema_version: input.schema_version,
        key_version: input.key_version,
        cipher_suite,
        kdf_algorithm: kdf.algorithm,
        kdf_salt,
        kdf_memory_kib: kdf.memory_kib,
        kdf_iterations: kdf.iterations,
        kdf_parallelism: kdf.parallelism,
        nonce,
        wrapped_master_key,
    })
}

fn reject_extra_fields(fields: &BTreeMap<String, Value>) -> AppResult<()> {
    if let Some(field) = fields.keys().find(|field| is_plaintext_field(field)) {
        return Err(AppError::Validation(format!(
            "包装密钥接口禁止接收明文或派生密钥字段 {field}"
        )));
    }
    if let Some(field) = fields.keys().next() {
        return Err(AppError::Validation(format!("未知包装密钥字段 {field}")));
    }
    Ok(())
}

fn is_plaintext_field(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "plaintext",
        "password",
        "passphrase",
        "masterkey",
        "wrappingkey",
        "derivedkey",
        "vaultkey",
        "privatekey",
        "credential",
        "secret",
        "token",
    ]
    .iter()
    .any(|blocked| normalized == *blocked)
}
