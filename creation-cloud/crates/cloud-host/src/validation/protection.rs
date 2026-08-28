//! Validate the fixed v1 envelope before any database transaction or KDF-sized allocation.

use cloud_domain::{AppError, AppResult};

use crate::{
    ChangeDataProtectionRequest, DataProtectionEnvelopeInput, MigrateDataProtectionRequest,
    SetupDataProtectionRequest,
};

use super::{
    ValidatedRekeyResource, opaque, protection_version, require_uuid, resource_candidates,
};

#[derive(Clone, Debug)]
pub(crate) struct ValidatedEnvelope {
    pub(crate) salt: Vec<u8>,
    pub(crate) nonce: Vec<u8>,
    pub(crate) wrapped_data_key: Vec<u8>,
}

pub(crate) fn setup_protection(
    request: &SetupDataProtectionRequest,
) -> AppResult<ValidatedEnvelope> {
    common_transition(
        request.mutation_id,
        request.sync_generation,
        request.expected_epoch,
        request.expected_revision,
        request.current_revision,
    )?;
    envelope(&request.envelope)
}

pub(crate) fn migrate_protection(
    request: &MigrateDataProtectionRequest,
) -> AppResult<(ValidatedEnvelope, Vec<ValidatedRekeyResource>)> {
    common_transition(
        request.mutation_id,
        request.sync_generation,
        request.expected_epoch,
        request.expected_revision,
        request.current_revision,
    )?;
    if request.expected_epoch != 0 || request.expected_revision != 0 {
        return Err(AppError::Validation(
            "legacy migrate 只允许从 protection 0/0 开始".to_owned(),
        ));
    }
    let envelope = envelope(&request.envelope)?;
    let resources = resource_candidates(&request.resources, "legacy migrate")?;
    if resources.is_empty() {
        return Err(AppError::Validation(
            "legacy migrate 必须包含完整活动密文集合".to_owned(),
        ));
    }
    Ok((envelope, resources))
}

pub(crate) fn change_protection(
    request: &ChangeDataProtectionRequest,
) -> AppResult<ValidatedEnvelope> {
    common_transition(
        request.mutation_id,
        request.sync_generation,
        request.expected_epoch,
        request.expected_revision,
        request.current_revision,
    )?;
    protection_version(request.expected_epoch, request.expected_revision, true)?;
    envelope(&request.envelope)
}

fn common_transition(
    mutation_id: uuid::Uuid,
    sync_generation: i64,
    epoch: i64,
    revision: i64,
    current_revision: i64,
) -> AppResult<()> {
    super::generation(sync_generation)?;
    protection_version(epoch, revision, false)?;
    require_uuid(mutation_id, "mutation_id")?;
    if current_revision < 0 {
        return Err(AppError::Validation(
            "current_revision 不能为负数".to_owned(),
        ));
    }
    Ok(())
}

fn envelope(input: &DataProtectionEnvelopeInput) -> AppResult<ValidatedEnvelope> {
    if input.format_version != 1
        || input.kdf_algorithm != "argon2id"
        || input.kdf_version != 19
        || input.kdf_memory_kib != 19_456
        || input.kdf_iterations != 2
        || input.kdf_parallelism != 1
        || input.kdf_output_length != 32
    {
        return Err(AppError::Validation(
            "data protection envelope 参数不符合固定 v1 profile".to_owned(),
        ));
    }
    Ok(ValidatedEnvelope {
        salt: exact_base64(&input.salt, "salt", 16)?,
        nonce: exact_base64(&input.nonce, "nonce", 24)?,
        wrapped_data_key: exact_base64(&input.wrapped_data_key, "wrapped_data_key", 48)?,
    })
}

fn exact_base64(value: &str, field: &str, size: usize) -> AppResult<Vec<u8>> {
    let decoded = opaque::decode_required(value, field, size)?;
    if decoded.len() == size {
        Ok(decoded)
    } else {
        Err(AppError::Validation(format!(
            "{field} 解码后必须恰好为 {size} 字节"
        )))
    }
}
