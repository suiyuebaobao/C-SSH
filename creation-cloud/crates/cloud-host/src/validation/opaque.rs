//! 只做 AI provider 密文、nonce 与不透明 envelope 元数据的结构和大小校验。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{AiProviderChange, AiProviderOperation};

use super::positive_expected;

pub(crate) const MAX_NONCE_BYTES: usize = 4 * 1024;
pub(crate) const MAX_ENVELOPE_METADATA_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct ValidatedAiPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub envelope_metadata: serde_json::Value,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedAiChange {
    pub resource_id: Uuid,
    pub operation: AiProviderOperation,
    pub payload: Option<ValidatedAiPayload>,
    pub expected_revision: Option<i64>,
}

pub(super) fn change_value(
    change: &AiProviderChange,
    max_ciphertext_bytes: usize,
) -> AppResult<ValidatedAiChange> {
    match change.operation {
        AiProviderOperation::Insert => {
            if change.expected_revision.is_some() {
                return Err(AppError::Validation(
                    "AI provider insert 不得携带 expected_revision".to_owned(),
                ));
            }
            let payload = change.payload.as_ref().ok_or_else(|| {
                AppError::Validation("AI provider insert 必须携带 payload".to_owned())
            })?;
            Ok(ValidatedAiChange {
                resource_id: change.resource_id,
                operation: change.operation,
                payload: Some(payload_value(payload, max_ciphertext_bytes)?),
                expected_revision: None,
            })
        }
        AiProviderOperation::Update => {
            let expected_revision = positive_expected(change.expected_revision)?;
            let payload = change.payload.as_ref().ok_or_else(|| {
                AppError::Validation("AI provider update 必须携带 payload".to_owned())
            })?;
            Ok(ValidatedAiChange {
                resource_id: change.resource_id,
                operation: change.operation,
                payload: Some(payload_value(payload, max_ciphertext_bytes)?),
                expected_revision: Some(expected_revision),
            })
        }
        AiProviderOperation::Delete => {
            if change.payload.is_some() {
                return Err(AppError::Validation(
                    "AI provider delete 不得携带 payload".to_owned(),
                ));
            }
            Ok(ValidatedAiChange {
                resource_id: change.resource_id,
                operation: change.operation,
                payload: None,
                expected_revision: Some(positive_expected(change.expected_revision)?),
            })
        }
    }
}

pub(super) fn payload_value(
    payload: &crate::AiProviderPayloadInput,
    max_ciphertext_bytes: usize,
) -> AppResult<ValidatedAiPayload> {
    payload_parts(
        &payload.ciphertext,
        &payload.nonce,
        &payload.envelope_metadata,
        max_ciphertext_bytes,
    )
}

pub(super) fn payload_parts(
    ciphertext: &str,
    nonce: &str,
    envelope_metadata: &serde_json::Value,
    max_ciphertext_bytes: usize,
) -> AppResult<ValidatedAiPayload> {
    if !envelope_metadata.is_object() {
        return Err(AppError::Validation(
            "envelope_metadata 必须是不透明 JSON object".to_owned(),
        ));
    }
    let encoded = serde_json::to_vec(envelope_metadata)
        .map_err(|_| AppError::Validation("envelope_metadata 无法编码".to_owned()))?;
    if encoded.len() > MAX_ENVELOPE_METADATA_BYTES {
        return Err(AppError::Validation(
            "envelope_metadata 超过 16 KiB".to_owned(),
        ));
    }
    Ok(ValidatedAiPayload {
        ciphertext: decode_required(ciphertext, "ciphertext", max_ciphertext_bytes)?,
        nonce: decode_required(nonce, "nonce", MAX_NONCE_BYTES)?,
        envelope_metadata: envelope_metadata.clone(),
    })
}

pub(super) fn decode_required(value: &str, field: &str, max_bytes: usize) -> AppResult<Vec<u8>> {
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} 不能为空字符串")));
    }
    let decoded = STANDARD
        .decode(value)
        .map_err(|_| AppError::Validation(format!("{field} 必须是合法 base64")))?;
    if decoded.is_empty() || decoded.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "{field} 解码后长度必须在 1 到 {max_bytes} 字节之间"
        )));
    }
    Ok(decoded)
}

pub(super) fn auxiliary_size(payload: &ValidatedAiPayload) -> AppResult<usize> {
    super::canonical_ai_auxiliary_size(&payload.nonce, &payload.envelope_metadata)
        .ok_or_else(|| AppError::Validation("AI envelope 辅助数据总量过大".to_owned()))
}
