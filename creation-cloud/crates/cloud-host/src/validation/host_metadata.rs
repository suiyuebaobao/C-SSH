//! 校验Host metadata显式迁移坐标、完整候选和跨端canonical请求摘要。

use cloud_domain::{AppError, AppResult};

use crate::{
    CURRENT_SYNC_CONTRACT_VERSION, HostChange, HostMetadataMigrationPreviewRequest,
    HostMetadataMigrationRequest, HostOperation,
};

use super::{
    MAX_CIPHERTEXT_BYTES, MAX_REKEY_CIPHERTEXT_BYTES, ValidatedChange, ValidatedEnvelope,
    decode_ciphertext, generation, opaque, positive_expected, protection_version, require_uuid,
};

pub(super) fn change_value(change: &HostChange) -> AppResult<ValidatedChange> {
    match change.operation {
        HostOperation::Insert => {
            if change.expected_revision.is_some() {
                return Err(AppError::Validation(
                    "insert 不得携带 expected_revision".to_owned(),
                ));
            }
            if change.metadata.is_some() {
                return Err(AppError::Validation(
                    "sync contract v4 Host insert不得携带明文metadata".to_owned(),
                ));
            }
            let ciphertext = match decode_ciphertext(change.ciphertext.as_ref())? {
                Some(Some(value)) => Some(Some(value)),
                _ => {
                    return Err(AppError::Validation(
                        "sync contract v4 Host insert必须携带不透明ciphertext".to_owned(),
                    ));
                }
            };
            Ok(ValidatedChange {
                host_id: change.host_id,
                operation: change.operation,
                ciphertext,
                expected_revision: None,
                metadata_encrypted: true,
            })
        }
        HostOperation::Update => {
            let expected_revision = positive_expected(change.expected_revision)?;
            if change.metadata.is_some() {
                return Err(AppError::Validation(
                    "sync contract v4 Host update不得携带明文metadata".to_owned(),
                ));
            }
            let ciphertext = match decode_ciphertext(change.ciphertext.as_ref())? {
                Some(Some(value)) => Some(Some(value)),
                _ => {
                    return Err(AppError::Validation(
                        "sync contract v4 Host update必须携带完整不透明ciphertext".to_owned(),
                    ));
                }
            };
            Ok(ValidatedChange {
                host_id: change.host_id,
                operation: change.operation,
                ciphertext,
                expected_revision: Some(expected_revision),
                metadata_encrypted: true,
            })
        }
        HostOperation::Delete => {
            if change.metadata.is_some() || change.ciphertext.is_some() {
                return Err(AppError::Validation(
                    "delete 只能携带主机标识和 expected_revision".to_owned(),
                ));
            }
            Ok(ValidatedChange {
                host_id: change.host_id,
                operation: change.operation,
                ciphertext: None,
                expected_revision: Some(positive_expected(change.expected_revision)?),
                metadata_encrypted: true,
            })
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedHostMetadataCandidate {
    pub host_id: uuid::Uuid,
    pub cloud_revision: i64,
    pub ciphertext: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedHostMetadataMigration {
    pub envelope: ValidatedEnvelope,
    pub hosts: Vec<ValidatedHostMetadataCandidate>,
    pub request_hash: [u8; 32],
}

pub(crate) fn preview(
    request: HostMetadataMigrationPreviewRequest,
) -> AppResult<HostMetadataMigrationPreviewRequest> {
    require_current(request.sync_contract_version)?;
    generation(request.sync_generation)?;
    protection_version(request.protection_epoch, request.protection_revision, true)?;
    if request.snapshot_revision.is_some_and(|value| value < 0) {
        return Err(AppError::Validation(
            "snapshot_revision 不能为负数".to_owned(),
        ));
    }
    if request.after_host_id.is_some_and(|value| value.is_nil()) {
        return Err(AppError::Validation(
            "after_host_id 不能是空 UUID".to_owned(),
        ));
    }
    if request.after_host_id.is_some() && request.snapshot_revision.is_none() {
        return Err(AppError::Validation(
            "metadata migration后续分页必须携带snapshot_revision".to_owned(),
        ));
    }
    if !(1..=200).contains(&request.limit) {
        return Err(AppError::Validation(
            "metadata migration preview limit 必须在1到200之间".to_owned(),
        ));
    }
    Ok(request)
}

pub(crate) fn migrate(
    request: &HostMetadataMigrationRequest,
) -> AppResult<ValidatedHostMetadataMigration> {
    require_current(request.sync_contract_version)?;
    require_uuid(request.mutation_id, "mutation_id")?;
    generation(request.sync_generation)?;
    protection_version(request.protection_epoch, request.protection_revision, true)?;
    if request.snapshot_revision < 0 {
        return Err(AppError::Validation(
            "snapshot_revision 不能为负数".to_owned(),
        ));
    }
    if request.hosts.len() > super::MAX_CURRENT_RESOURCES {
        return Err(AppError::SyncCapacityExceeded(format!(
            "metadata migration Host不能超过{}项",
            super::MAX_CURRENT_RESOURCES,
        )));
    }
    let envelope = super::protection::metadata_migration_envelope(&request.target_envelope)?;
    let mut previous = None;
    let mut total = 0_usize;
    let mut hosts = Vec::with_capacity(request.hosts.len());
    for candidate in &request.hosts {
        require_uuid(candidate.host_id, "host_id")?;
        if candidate.cloud_revision <= 0 {
            return Err(AppError::Validation("cloud_revision 必须大于0".to_owned()));
        }
        if previous.is_some_and(|value| value >= candidate.host_id) {
            return Err(AppError::Validation(
                "metadata migration hosts必须按host_id严格升序且不重复".to_owned(),
            ));
        }
        previous = Some(candidate.host_id);
        let ciphertext = candidate
            .ciphertext
            .as_ref()
            .map(|value| opaque::decode_required(value, "ciphertext", MAX_CIPHERTEXT_BYTES))
            .transpose()?;
        if let Some(value) = ciphertext.as_ref() {
            total = total.checked_add(value.len()).ok_or_else(|| {
                AppError::Validation("metadata migration ciphertext总量过大".to_owned())
            })?;
        }
        hosts.push(ValidatedHostMetadataCandidate {
            host_id: candidate.host_id,
            cloud_revision: candidate.cloud_revision,
            ciphertext,
        });
    }
    if total > MAX_REKEY_CIPHERTEXT_BYTES {
        return Err(AppError::SyncCapacityExceeded(
            "metadata migration ciphertext总量超过32 MiB".to_owned(),
        ));
    }
    Ok(ValidatedHostMetadataMigration {
        envelope,
        hosts,
        request_hash: crate::fingerprint::canonical(request)?,
    })
}

pub(crate) fn require_current(version: u16) -> AppResult<()> {
    if version == CURRENT_SYNC_CONTRACT_VERSION {
        Ok(())
    } else {
        Err(AppError::SyncContractUpgradeRequired(
            "Host全字段密文同步需要sync contract version 4".to_owned(),
        ))
    }
}
