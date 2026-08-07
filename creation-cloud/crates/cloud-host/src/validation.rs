//! 在事务前校验 Host、AI 密文资源、批次上限和手动同步游标。

use std::{collections::HashSet, net::IpAddr};

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{
    HostChange, HostMetadataInput, HostOperation, PullAckRequest, PullMode, PullRequest,
    PushRequest, RekeyResourceCandidate, RekeySyncRequest, ResetSyncRequest, ResourceKind,
};

mod opaque;

#[cfg(test)]
pub(crate) use opaque::MAX_NONCE_BYTES;
pub(crate) use opaque::{ValidatedAiChange, ValidatedAiPayload};

pub(crate) const MAX_CIPHERTEXT_BYTES: usize = 256 * 1024;
const MAX_PULL_DECISIONS: usize = MAX_REKEY_RESOURCES;
pub(crate) const MAX_REKEY_RESOURCES: usize = 2_000;
pub(crate) const MAX_CURRENT_RESOURCES: usize = 5_000;
const MAX_PUSH_CHANGES: usize = MAX_REKEY_RESOURCES;
pub(crate) const MAX_REKEY_CIPHERTEXT_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_SYNC_AUXILIARY_BYTES: usize = 512 * 1024;

pub(crate) fn canonical_ai_auxiliary_size(
    nonce: &[u8],
    envelope_metadata: &serde_json::Value,
) -> Option<usize> {
    nonce
        .len()
        .checked_add(serde_json::to_vec(envelope_metadata).ok()?.len())
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedChange {
    pub host_id: Uuid,
    pub operation: HostOperation,
    pub metadata: Option<HostMetadataInput>,
    pub ciphertext: Option<Option<Vec<u8>>>,
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedPush {
    pub host_changes: Vec<ValidatedChange>,
    pub ai_changes: Vec<ValidatedAiChange>,
}

pub(crate) fn push(request: &PushRequest) -> AppResult<ValidatedPush> {
    generation(request.sync_generation)?;
    if request.base_revision < 0 {
        return Err(AppError::Validation("base_revision 不能为负数".to_owned()));
    }
    require_uuid(request.client_mutation_id, "client_mutation_id")?;
    let count = request
        .host_changes
        .len()
        .checked_add(request.ai_changes.len())
        .ok_or_else(|| AppError::Validation("同步变更数量过大".to_owned()))?;
    if count == 0 {
        return Err(AppError::Validation(
            "每次 push 必须包含至少 1 项变更".to_owned(),
        ));
    }
    if count > MAX_PUSH_CHANGES {
        return Err(AppError::SyncCapacityExceeded(format!(
            "每次 push 不能超过 {MAX_PUSH_CHANGES} 项变更"
        )));
    }

    let mut host_ids = HashSet::with_capacity(request.host_changes.len());
    let mut host_changes = Vec::with_capacity(request.host_changes.len());
    for change in &request.host_changes {
        require_uuid(change.host_id, "host_id")?;
        if !host_ids.insert(change.host_id) {
            return Err(AppError::Validation(
                "同一次 mutation 不得重复修改同一主机".to_owned(),
            ));
        }
        host_changes.push(host_change_value(change)?);
    }

    let mut ai_ids = HashSet::with_capacity(request.ai_changes.len());
    let mut ai_changes = Vec::with_capacity(request.ai_changes.len());
    for change in &request.ai_changes {
        require_uuid(change.resource_id, "resource_id")?;
        if !ai_ids.insert(change.resource_id) {
            return Err(AppError::Validation(
                "同一次 mutation 不得重复修改同一 AI provider 账号".to_owned(),
            ));
        }
        ai_changes.push(opaque::change_value(change, MAX_CIPHERTEXT_BYTES)?);
    }
    enforce_payload_totals(&host_changes, &ai_changes, "push")?;
    Ok(ValidatedPush {
        host_changes,
        ai_changes,
    })
}

pub(crate) fn pull(request: PullRequest) -> AppResult<PullRequest> {
    generation(request.sync_generation)?;
    if request.since_revision < 0 {
        return Err(AppError::Validation("since_revision 不能为负数".to_owned()));
    }
    if request.mode == PullMode::Full && request.since_revision != 0 {
        return Err(AppError::Validation(
            "full pull 的 since_revision 必须为 0".to_owned(),
        ));
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
    if let Some(revision) = request.after_revision {
        let snapshot = request
            .snapshot_revision
            .ok_or_else(|| AppError::Validation("后续分页必须携带 snapshot_revision".to_owned()))?;
        if revision < 0 || revision > snapshot {
            return Err(AppError::Validation(
                "after_revision 必须是 snapshot 范围内的非负页内游标".to_owned(),
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
        require_uuid(decision.resource_id, "resource_id")?;
        if decision.cloud_revision <= 0 {
            return Err(AppError::Validation("cloud_revision 必须大于 0".to_owned()));
        }
        if decision.cloud_revision > request.acknowledged_revision {
            return Err(AppError::Validation(
                "决策 revision 不能超过确认水位".to_owned(),
            ));
        }
        if !identities.insert((
            decision.resource_kind,
            decision.resource_id,
            decision.cloud_revision,
        )) {
            return Err(AppError::Validation(
                "同一资源 revision 只能提交一个本地决定".to_owned(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) enum ValidatedRekeyResource {
    Host {
        resource_id: Uuid,
        cloud_revision: i64,
        ciphertext: Vec<u8>,
    },
    AiProviderAccount {
        resource_id: Uuid,
        cloud_revision: i64,
        payload: ValidatedAiPayload,
    },
}

impl ValidatedRekeyResource {
    pub(crate) const fn resource_kind(&self) -> ResourceKind {
        match self {
            Self::Host { .. } => ResourceKind::Host,
            Self::AiProviderAccount { .. } => ResourceKind::AiProviderAccount,
        }
    }

    pub(crate) const fn resource_id(&self) -> Uuid {
        match self {
            Self::Host { resource_id, .. } | Self::AiProviderAccount { resource_id, .. } => {
                *resource_id
            }
        }
    }

    pub(crate) const fn cloud_revision(&self) -> i64 {
        match self {
            Self::Host { cloud_revision, .. } | Self::AiProviderAccount { cloud_revision, .. } => {
                *cloud_revision
            }
        }
    }
}

pub(crate) fn reset(request: &ResetSyncRequest) -> AppResult<()> {
    generation(request.sync_generation)?;
    require_uuid(request.mutation_id, "mutation_id")
}

pub(crate) fn rekey(request: &RekeySyncRequest) -> AppResult<Vec<ValidatedRekeyResource>> {
    generation(request.sync_generation)?;
    require_uuid(request.mutation_id, "mutation_id")?;
    if request.resources.len() > MAX_REKEY_RESOURCES {
        return Err(AppError::SyncCapacityExceeded(format!(
            "resources 不能超过 {MAX_REKEY_RESOURCES} 项"
        )));
    }
    let mut unique = HashSet::with_capacity(request.resources.len());
    let mut ciphertext_total = 0_usize;
    let mut auxiliary_total = 0_usize;
    let mut validated = Vec::with_capacity(request.resources.len());
    for resource in &request.resources {
        let (kind, resource_id, cloud_revision) = resource_identity(resource);
        require_uuid(resource_id, "resource_id")?;
        if !unique.insert((kind, resource_id)) {
            return Err(AppError::Validation(
                "rekey resources 不得包含重复资源".to_owned(),
            ));
        }
        if cloud_revision <= 0 {
            return Err(AppError::Validation("cloud_revision 必须大于 0".to_owned()));
        }
        let value = match resource {
            RekeyResourceCandidate::Host { ciphertext, .. } => {
                let ciphertext =
                    opaque::decode_required(ciphertext, "ciphertext", MAX_CIPHERTEXT_BYTES)?;
                ciphertext_total = add_total(ciphertext_total, ciphertext.len(), "ciphertext")?;
                ValidatedRekeyResource::Host {
                    resource_id,
                    cloud_revision,
                    ciphertext,
                }
            }
            RekeyResourceCandidate::AiProviderAccount {
                ciphertext,
                nonce,
                envelope_metadata,
                ..
            } => {
                let payload = opaque::payload_parts(
                    ciphertext,
                    nonce,
                    envelope_metadata,
                    MAX_CIPHERTEXT_BYTES,
                )?;
                ciphertext_total =
                    add_total(ciphertext_total, payload.ciphertext.len(), "ciphertext")?;
                auxiliary_total = add_total(
                    auxiliary_total,
                    opaque::auxiliary_size(&payload)?,
                    "AI envelope",
                )?;
                ValidatedRekeyResource::AiProviderAccount {
                    resource_id,
                    cloud_revision,
                    payload,
                }
            }
        };
        enforce_rekey_totals(ciphertext_total, auxiliary_total)?;
        validated.push(value);
    }
    validated.sort_unstable_by_key(|resource| {
        (resource.resource_kind().as_str(), resource.resource_id())
    });
    Ok(validated)
}

fn resource_identity(resource: &RekeyResourceCandidate) -> (ResourceKind, Uuid, i64) {
    match resource {
        RekeyResourceCandidate::Host {
            resource_id,
            cloud_revision,
            ..
        } => (ResourceKind::Host, *resource_id, *cloud_revision),
        RekeyResourceCandidate::AiProviderAccount {
            resource_id,
            cloud_revision,
            ..
        } => (
            ResourceKind::AiProviderAccount,
            *resource_id,
            *cloud_revision,
        ),
    }
}

fn host_change_value(change: &HostChange) -> AppResult<ValidatedChange> {
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

fn enforce_payload_totals(
    host_changes: &[ValidatedChange],
    ai_changes: &[ValidatedAiChange],
    scope: &str,
) -> AppResult<()> {
    let mut ciphertext_total = 0_usize;
    for size in host_changes
        .iter()
        .filter_map(|change| change.ciphertext.as_ref()?.as_ref().map(Vec::len))
        .chain(ai_changes.iter().filter_map(|change| {
            change
                .payload
                .as_ref()
                .map(|payload| payload.ciphertext.len())
        }))
    {
        ciphertext_total = add_total(ciphertext_total, size, "ciphertext")?;
    }
    let mut auxiliary_total = 0_usize;
    for payload in ai_changes
        .iter()
        .filter_map(|change| change.payload.as_ref())
    {
        auxiliary_total = add_total(
            auxiliary_total,
            opaque::auxiliary_size(payload)?,
            "AI envelope",
        )?;
    }
    enforce_totals(ciphertext_total, auxiliary_total, scope)
}

fn enforce_totals(ciphertext: usize, auxiliary: usize, scope: &str) -> AppResult<()> {
    if ciphertext > MAX_REKEY_CIPHERTEXT_BYTES {
        return Err(AppError::SyncCapacityExceeded(format!(
            "{scope} ciphertext 总量超过 32 MiB"
        )));
    }
    if auxiliary > MAX_SYNC_AUXILIARY_BYTES {
        return Err(AppError::SyncCapacityExceeded(format!(
            "{scope} AI envelope 辅助数据总量超过 512 KiB"
        )));
    }
    Ok(())
}

fn enforce_rekey_totals(ciphertext: usize, auxiliary: usize) -> AppResult<()> {
    if ciphertext > MAX_REKEY_CIPHERTEXT_BYTES {
        return Err(AppError::SyncCapacityExceeded(
            "rekey ciphertext 总量超过 32 MiB".to_owned(),
        ));
    }
    if auxiliary > MAX_SYNC_AUXILIARY_BYTES {
        return Err(AppError::SyncCapacityExceeded(
            "rekey AI envelope 辅助数据总量超过 512 KiB".to_owned(),
        ));
    }
    Ok(())
}

fn add_total(total: usize, value: usize, field: &str) -> AppResult<usize> {
    total
        .checked_add(value)
        .ok_or_else(|| AppError::Validation(format!("{field} 总量过大")))
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
    address
        .parse::<IpAddr>()
        .map(|_| ())
        .map_err(|_| AppError::Validation("address 必须是纯 IPv4 或 IPv6 地址".to_owned()))
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
    opaque::decode_required(value, "ciphertext", MAX_CIPHERTEXT_BYTES)
        .map(|value| Some(Some(value)))
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

pub(crate) fn host_id(host_id: Uuid) -> AppResult<()> {
    require_uuid(host_id, "host_id")
}
