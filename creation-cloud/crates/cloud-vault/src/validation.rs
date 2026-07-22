//! 提供保险库密文结构共用校验，并拒绝明文、凭据和主机字段。

use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use uuid::Uuid;

use crate::types::{
    CreateEnvelope, CreateVaultEnvelopeInput, KdfMetadata, UpdateEnvelope, UpdateVaultEnvelopeInput,
};

const MAX_CIPHERTEXT_BYTES: usize = 1024 * 1024;

pub(crate) fn account(account_id: Uuid) -> AppResult<()> {
    if account_id.is_nil() {
        return Err(AppError::Validation("账号标识不能为空".to_owned()));
    }
    Ok(())
}

pub(crate) fn envelope_id(envelope_id: Uuid) -> AppResult<()> {
    if envelope_id.is_nil() {
        return Err(AppError::Validation("密文信封标识不能为空".to_owned()));
    }
    Ok(())
}

pub(crate) fn create(input: CreateVaultEnvelopeInput) -> AppResult<CreateEnvelope> {
    reject_extra_fields(&input.extra_fields)?;
    if input.envelope_key.is_nil() {
        return Err(AppError::Validation("envelope_key 不能为空".to_owned()));
    }
    versions(input.schema_version, input.key_version)?;
    let cipher_suite = cipher_suite(input.cipher_suite)?;
    let kdf = kdf(input.kdf)?;
    let nonce = decode_bounded("nonce", &input.nonce, 24, 24)?;
    let ciphertext = decode_bounded("ciphertext", &input.ciphertext, 16, MAX_CIPHERTEXT_BYTES)?;
    Ok(CreateEnvelope {
        id: Uuid::now_v7(),
        envelope_key: input.envelope_key,
        schema_version: input.schema_version,
        key_version: input.key_version,
        cipher_suite,
        kdf,
        nonce,
        ciphertext,
    })
}

pub(crate) fn update(input: UpdateVaultEnvelopeInput) -> AppResult<UpdateEnvelope> {
    reject_extra_fields(&input.extra_fields)?;
    revision(input.expected_revision)?;
    versions(input.schema_version, input.key_version)?;
    let cipher_suite = cipher_suite(input.cipher_suite)?;
    let kdf = kdf(input.kdf)?;
    let nonce = decode_bounded("nonce", &input.nonce, 24, 24)?;
    let ciphertext = decode_bounded("ciphertext", &input.ciphertext, 16, MAX_CIPHERTEXT_BYTES)?;
    Ok(UpdateEnvelope {
        expected_revision: input.expected_revision,
        schema_version: input.schema_version,
        key_version: input.key_version,
        cipher_suite,
        kdf,
        nonce,
        ciphertext,
    })
}

pub(crate) fn revision(value: i64) -> AppResult<()> {
    if value < 1 {
        return Err(AppError::Validation(
            "expected_revision 必须大于零".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn versions(schema_version: i32, key_version: i32) -> AppResult<()> {
    if !(1..=65_535).contains(&schema_version) || !(1..=65_535).contains(&key_version) {
        return Err(AppError::Validation(
            "schema_version 和 key_version 必须在 1 到 65535 之间".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn cipher_suite(value: String) -> AppResult<String> {
    let value = value.trim().to_ascii_lowercase();
    if value != "xchacha20-poly1305" {
        return Err(AppError::Validation(
            "cipher_suite 仅允许 xchacha20-poly1305".to_owned(),
        ));
    }
    Ok(value)
}

pub(crate) fn kdf(mut value: KdfMetadata) -> AppResult<KdfMetadata> {
    value.algorithm = value.algorithm.trim().to_ascii_lowercase();
    if value.algorithm != "argon2id" {
        return Err(AppError::Validation(
            "KDF algorithm 仅允许 argon2id".to_owned(),
        ));
    }
    let salt = decode_bounded("kdf.salt", &value.salt, 16, 64)?;
    value.salt = STANDARD.encode(salt);
    if !(19_456..=1_048_576).contains(&value.memory_kib)
        || !(1..=20).contains(&value.iterations)
        || !(1..=16).contains(&value.parallelism)
    {
        return Err(AppError::Validation("KDF 参数超出允许范围".to_owned()));
    }
    Ok(value)
}

pub(crate) fn decode_bounded(
    name: &str,
    encoded: &str,
    minimum: usize,
    maximum: usize,
) -> AppResult<Vec<u8>> {
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::Validation(format!("{name} 必须是有效 base64")))?;
    if !(minimum..=maximum).contains(&decoded.len()) {
        return Err(AppError::Validation(format!(
            "{name} 解码后长度必须在 {minimum} 到 {maximum} 字节之间"
        )));
    }
    Ok(decoded)
}

fn reject_extra_fields(fields: &BTreeMap<String, Value>) -> AppResult<()> {
    if let Some(field) = fields.keys().find(|field| is_plaintext_field(field)) {
        return Err(AppError::Validation(format!(
            "保险库接口禁止接收明文或凭据字段 {field}"
        )));
    }
    if let Some(field) = fields.keys().next() {
        return Err(AppError::Validation(format!("未知密文信封字段 {field}")));
    }
    Ok(())
}

fn is_plaintext_field(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "plaintext",
        "password",
        "passphrase",
        "apikey",
        "token",
        "credential",
        "privatekey",
        "masterkey",
        "wrappingkey",
        "host",
        "address",
        "ssh",
        "terminal",
        "command",
        "aiprompt",
        "airesponse",
    ]
    .iter()
    .any(|blocked| normalized.contains(blocked))
}
