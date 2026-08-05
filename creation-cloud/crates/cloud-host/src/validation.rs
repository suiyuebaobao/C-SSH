//! 在事务前校验主机元数据、base64 密文和手动同步游标边界。

use std::{collections::HashSet, net::IpAddr};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{
    HostChange, HostMetadataInput, HostOperation, PullAckRequest, PullRequest, RekeySyncRequest,
    ResetSyncRequest, ResolveConflictRequest,
};

pub(crate) const MAX_CIPHERTEXT_BYTES: usize = 256 * 1024;
const MAX_PULL_DECISIONS: usize = 200;
pub(crate) const MAX_REKEY_HOSTS: usize = 2_000;
pub(crate) const MAX_REKEY_CIPHERTEXT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct ValidatedChange {
    pub host_id: Uuid,
    pub operation: HostOperation,
    pub metadata: Option<HostMetadataInput>,
    pub ciphertext: Option<Option<Vec<u8>>>,
    pub expected_revision: Option<i64>,
}

pub(crate) fn push(
    sync_generation: i64,
    base_revision: i64,
    mutation_id: Uuid,
    changes: &[HostChange],
) -> AppResult<Vec<ValidatedChange>> {
    generation(sync_generation)?;
    if base_revision < 0 {
        return Err(AppError::Validation("base_revision 不能为负数".to_owned()));
    }
    require_uuid(mutation_id, "client_mutation_id")?;
    if changes.len() != 1 {
        return Err(AppError::Validation(
            "each client_mutation_id must contain exactly one host change".to_owned(),
        ));
    }

    let mut host_ids = HashSet::with_capacity(changes.len());
    let mut validated = Vec::with_capacity(changes.len());
    for change in changes {
        require_uuid(change.host_id, "host_id")?;
        if !host_ids.insert(change.host_id) {
            return Err(AppError::Validation(
                "同一次 mutation 不得重复修改同一主机".to_owned(),
            ));
        }
        validated.push(change_value(change)?);
    }
    Ok(validated)
}

pub(crate) fn pull(request: PullRequest) -> AppResult<PullRequest> {
    generation(request.sync_generation)?;
    if request.since_revision < 0 {
        return Err(AppError::Validation("since_revision 不能为负数".to_owned()));
    }
    if !(1..=200).contains(&request.limit) {
        return Err(AppError::Validation(
            "limit 必须在 1 到 200 之间".to_owned(),
        ));
    }
    if request
        .snapshot_revision
        .is_some_and(|value| value < request.since_revision)
    {
        return Err(AppError::Validation(
            "snapshot_revision 不能早于 since_revision".to_owned(),
        ));
    }
    match (request.after_revision, request.after_host_id) {
        (Some(revision), Some(host_id)) => {
            if revision < 0 || host_id.is_nil() || request.snapshot_revision.is_none() {
                return Err(AppError::Validation(
                    "后续分页必须携带有效 snapshot 与完整游标".to_owned(),
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(AppError::Validation(
                "after_revision 与 after_host_id 必须同时提供".to_owned(),
            ));
        }
    }
    Ok(request)
}

pub(crate) fn ack(request: &PullAckRequest) -> AppResult<()> {
    generation(request.sync_generation)?;
    if request.acknowledged_revision < 0 {
        return Err(AppError::Validation(
            "acknowledged_revision 不能为负数".to_owned(),
        ));
    }
    if request.decisions.len() > MAX_PULL_DECISIONS {
        return Err(AppError::Validation(format!(
            "decisions 不能超过 {MAX_PULL_DECISIONS} 项"
        )));
    }
    let mut identities = HashSet::with_capacity(request.decisions.len());
    for decision in &request.decisions {
        require_uuid(decision.host_id, "host_id")?;
        if decision.cloud_revision <= 0 {
            return Err(AppError::Validation("cloud_revision 必须大于 0".to_owned()));
        }
        if decision.cloud_revision > request.acknowledged_revision {
            return Err(AppError::Validation(
                "决策 revision 不能超过确认水位".to_owned(),
            ));
        }
        if !identities.insert((decision.host_id, decision.cloud_revision)) {
            return Err(AppError::Validation(
                "同一主机 revision 只能提交一个本地决定".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn resolve(conflict_id: Uuid, request: &ResolveConflictRequest) -> AppResult<()> {
    require_uuid(conflict_id, "conflict_id")?;
    generation(request.sync_generation)?;
    require_uuid(request.resolution_mutation_id, "resolution_mutation_id")?;
    if request.expected_revision < 0 {
        return Err(AppError::Validation(
            "expected_revision 不能为负数".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedRekeyHost {
    pub host_id: Uuid,
    pub cloud_revision: i64,
    pub ciphertext: Vec<u8>,
}

pub(crate) fn reset(request: &ResetSyncRequest) -> AppResult<()> {
    require_uuid(request.mutation_id, "mutation_id")
}

pub(crate) fn rekey(request: &RekeySyncRequest) -> AppResult<Vec<ValidatedRekeyHost>> {
    generation(request.sync_generation)?;
    require_uuid(request.mutation_id, "mutation_id")?;
    if request.hosts.len() > MAX_REKEY_HOSTS {
        return Err(AppError::Validation(format!(
            "hosts 不能超过 {MAX_REKEY_HOSTS} 项"
        )));
    }
    let mut unique = HashSet::with_capacity(request.hosts.len());
    let mut total = 0_usize;
    let mut validated = Vec::with_capacity(request.hosts.len());
    for host in &request.hosts {
        require_uuid(host.host_id, "host_id")?;
        if !unique.insert(host.host_id) {
            return Err(AppError::Validation(
                "rekey hosts 不得包含重复主机".to_owned(),
            ));
        }
        if host.cloud_revision <= 0 {
            return Err(AppError::Validation("cloud_revision 必须大于 0".to_owned()));
        }
        let ciphertext = decode_ciphertext(Some(&Some(host.ciphertext.clone())))?
            .and_then(|value| value)
            .ok_or_else(|| AppError::Validation("ciphertext 不能为空".to_owned()))?;
        total = total
            .checked_add(ciphertext.len())
            .ok_or_else(|| AppError::Validation("rekey ciphertext 总量过大".to_owned()))?;
        if total > MAX_REKEY_CIPHERTEXT_BYTES {
            return Err(AppError::Validation(
                "rekey ciphertext 总量超过 32 MiB".to_owned(),
            ));
        }
        validated.push(ValidatedRekeyHost {
            host_id: host.host_id,
            cloud_revision: host.cloud_revision,
            ciphertext,
        });
    }
    validated.sort_unstable_by_key(|host| host.host_id);
    Ok(validated)
}

pub(crate) fn host_id(host_id: Uuid) -> AppResult<()> {
    require_uuid(host_id, "host_id")
}

pub(crate) fn conflict_id(conflict_id: Uuid) -> AppResult<()> {
    require_uuid(conflict_id, "conflict_id")
}

fn generation(value: i64) -> AppResult<()> {
    if value <= 0 {
        Err(AppError::Validation(
            "sync_generation 必须大于 0".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn change_value(change: &HostChange) -> AppResult<ValidatedChange> {
    let ciphertext = decode_ciphertext(change.ciphertext.as_ref())?;
    match change.operation {
        HostOperation::Insert => {
            if change.expected_revision.is_some() {
                return Err(AppError::Validation(
                    "insert 不得携带 expected_revision".to_owned(),
                ));
            }
            let metadata = change
                .metadata
                .clone()
                .ok_or_else(|| AppError::Validation("insert 必须携带 metadata".to_owned()))?;
            validate_metadata(&metadata)?;
            Ok(ValidatedChange {
                host_id: change.host_id,
                operation: change.operation,
                metadata: Some(metadata),
                ciphertext,
                expected_revision: None,
            })
        }
        HostOperation::Update => {
            let expected_revision = positive_expected(change.expected_revision)?;
            let metadata = change
                .metadata
                .clone()
                .ok_or_else(|| AppError::Validation("update 必须携带 metadata".to_owned()))?;
            validate_metadata(&metadata)?;
            Ok(ValidatedChange {
                host_id: change.host_id,
                operation: change.operation,
                metadata: Some(metadata),
                ciphertext,
                expected_revision: Some(expected_revision),
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
                metadata: None,
                ciphertext: None,
                expected_revision: Some(positive_expected(change.expected_revision)?),
            })
        }
    }
}

fn validate_metadata(metadata: &HostMetadataInput) -> AppResult<()> {
    validate_address(&metadata.address)?;
    if metadata.port == 0 {
        return Err(AppError::Validation(
            "port must be between 1 and 65535".to_owned(),
        ));
    }
    validate_text(&metadata.name, 128, "name")?;
    validate_text(&metadata.platform, 32, "platform")?;
    if metadata.tags.len() > 32 {
        return Err(AppError::Validation("tags 不能超过 32 项".to_owned()));
    }
    let mut tags = HashSet::with_capacity(metadata.tags.len());
    for tag in &metadata.tags {
        validate_text(tag, 48, "tag")?;
        if !tags.insert(tag) {
            return Err(AppError::Validation("tags 不得重复".to_owned()));
        }
    }
    Ok(())
}

fn validate_address(address: &str) -> AppResult<()> {
    if address.is_empty()
        || address.len() > 253
        || address.trim() != address
        || address.chars().any(char::is_control)
    {
        return Err(AppError::Validation(
            "address 必须是 IP 或不含端口的域名".to_owned(),
        ));
    }
    if address.parse::<IpAddr>().is_ok() {
        return Ok(());
    }
    if address.contains(['/', '@', ':']) {
        return Err(AppError::Validation(
            "address 必须是 IP 或不含端口的域名".to_owned(),
        ));
    }
    let valid_domain = address.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
    });
    if valid_domain {
        Ok(())
    } else {
        Err(AppError::Validation(
            "address 必须是 IP 或不含端口的域名".to_owned(),
        ))
    }
}

fn validate_text(value: &str, max_chars: usize, field: &str) -> AppResult<()> {
    let count = value.chars().count();
    if count == 0
        || count > max_chars
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation(format!("{field} 长度或字符不合法")));
    }
    Ok(())
}

fn decode_ciphertext(value: Option<&Option<String>>) -> AppResult<Option<Option<Vec<u8>>>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(value) = value.as_deref() else {
        return Ok(Some(None));
    };
    if value.is_empty() {
        return Err(AppError::Validation("ciphertext 不能为空字符串".to_owned()));
    }
    let decoded = STANDARD
        .decode(value)
        .map_err(|_| AppError::Validation("ciphertext 必须是合法 base64".to_owned()))?;
    if decoded.len() > MAX_CIPHERTEXT_BYTES {
        return Err(AppError::Validation(
            "ciphertext 解码后超过 256 KiB".to_owned(),
        ));
    }
    Ok(Some(Some(decoded)))
}

fn positive_expected(value: Option<i64>) -> AppResult<i64> {
    value
        .filter(|revision| *revision > 0)
        .ok_or_else(|| AppError::Validation("expected_revision 必须大于 0".to_owned()))
}

fn require_uuid(value: Uuid, field: &str) -> AppResult<()> {
    if value.is_nil() {
        Err(AppError::Validation(format!("{field} 不能为空")))
    } else {
        Ok(())
    }
}
