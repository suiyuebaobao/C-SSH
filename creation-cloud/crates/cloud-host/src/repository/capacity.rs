//! 在账号同步状态锁内统一限制 Host/AI 当前身份数、密文量和 AI 辅助数据量。

use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    AiProviderOperation, HostOperation,
    validation::{
        MAX_CURRENT_RESOURCES, MAX_REKEY_CIPHERTEXT_BYTES, MAX_REKEY_RESOURCES,
        MAX_SYNC_AUXILIARY_BYTES, ValidatedAiChange, ValidatedAiPayload, ValidatedChange,
        canonical_ai_auxiliary_size,
    },
};

use super::{DbTransaction, ai::AiRow, hosts::HostRow, storage};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Capacity {
    identities: i64,
    encrypted_resources: i64,
    ciphertext_bytes: i64,
    auxiliary_bytes: i64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResourceState {
    identity: bool,
    encrypted: bool,
    ciphertext_bytes: usize,
    auxiliary_bytes: usize,
}

pub(super) async fn require_current_within_limit(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<()> {
    require_within_limit(load_current(tx, account_id).await?)
}

pub(super) async fn enforce_encrypted_resource_limit(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    host_changes: &[ValidatedChange],
    host_rows: &[Option<HostRow>],
    ai_changes: &[ValidatedAiChange],
    ai_rows: &[Option<AiRow>],
) -> AppResult<()> {
    // 调用方已持有账号同步状态行锁；所有 Host/AI 写路径都按该锁串行，
    // 因此当前快照与净变化共同构成事务内的账号级容量门禁。
    let current = load_current(tx, account_id).await?;
    let mut delta = Capacity::default();
    for (change, row) in host_changes.iter().zip(host_rows) {
        delta = checked_add(
            delta,
            resource_delta(host_current(row.as_ref()), host_final(change, row.as_ref()))?,
        )?;
    }
    for (change, row) in ai_changes.iter().zip(ai_rows) {
        delta = checked_add(
            delta,
            resource_delta(ai_current(row.as_ref())?, ai_final(change)?)?,
        )?;
    }
    require_within_limit(checked_add(current, delta)?)
}

async fn load_current(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<Capacity> {
    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT
             (SELECT count(*)::BIGINT FROM cloud_hosts WHERE account_id = $1)
           + (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs WHERE account_id = $1),
             (SELECT count(*)::BIGINT FROM cloud_hosts
              WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL)
           + (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs
              WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL),
             (SELECT COALESCE(sum(octet_length(ciphertext)), 0)::BIGINT
              FROM cloud_hosts
              WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL)
           + (SELECT COALESCE(sum(octet_length(ciphertext)), 0)::BIGINT
              FROM cloud_ai_provider_configs
              WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL)",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;

    let ai_rows = sqlx::query_as::<_, (Vec<u8>, Value)>(
        "SELECT nonce, envelope_metadata
         FROM cloud_ai_provider_configs
         WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let mut auxiliary_bytes = 0_usize;
    for (nonce, metadata) in ai_rows {
        auxiliary_bytes = auxiliary_bytes
            .checked_add(nonce.len())
            .and_then(|size| {
                serde_json::to_vec(&metadata)
                    .ok()
                    .and_then(|encoded| size.checked_add(encoded.len()))
            })
            .ok_or_else(super::invalid_stored_value)?;
    }

    Ok(Capacity {
        identities: row.0,
        encrypted_resources: row.1,
        ciphertext_bytes: row.2,
        auxiliary_bytes: i64::try_from(auxiliary_bytes)
            .map_err(|_| super::invalid_stored_value())?,
    })
}

fn require_within_limit(capacity: Capacity) -> AppResult<()> {
    if capacity.identities < 0
        || capacity.encrypted_resources < 0
        || capacity.ciphertext_bytes < 0
        || capacity.auxiliary_bytes < 0
    {
        return Err(super::invalid_stored_value());
    }
    if capacity.identities > usize_limit(MAX_CURRENT_RESOURCES) {
        return Err(capacity_error(format!(
            "current Host and AI identities cannot exceed {MAX_CURRENT_RESOURCES}"
        )));
    }
    if capacity.encrypted_resources > usize_limit(MAX_REKEY_RESOURCES) {
        return Err(capacity_error(format!(
            "active encrypted resources cannot exceed {MAX_REKEY_RESOURCES}"
        )));
    }
    if capacity.ciphertext_bytes > usize_limit(MAX_REKEY_CIPHERTEXT_BYTES) {
        return Err(capacity_error(
            "active encrypted ciphertext cannot exceed 32 MiB".to_owned(),
        ));
    }
    if capacity.auxiliary_bytes > usize_limit(MAX_SYNC_AUXILIARY_BYTES) {
        return Err(capacity_error(
            "active AI envelope auxiliary data cannot exceed 512 KiB".to_owned(),
        ));
    }
    Ok(())
}

fn usize_limit(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn capacity_error(message: String) -> AppError {
    AppError::SyncCapacityExceeded(message)
}

fn checked_add(left: Capacity, right: Capacity) -> AppResult<Capacity> {
    Ok(Capacity {
        identities: left
            .identities
            .checked_add(right.identities)
            .ok_or_else(super::invalid_stored_value)?,
        encrypted_resources: left
            .encrypted_resources
            .checked_add(right.encrypted_resources)
            .ok_or_else(super::invalid_stored_value)?,
        ciphertext_bytes: left
            .ciphertext_bytes
            .checked_add(right.ciphertext_bytes)
            .ok_or_else(super::invalid_stored_value)?,
        auxiliary_bytes: left
            .auxiliary_bytes
            .checked_add(right.auxiliary_bytes)
            .ok_or_else(super::invalid_stored_value)?,
    })
}

fn resource_delta(before: ResourceState, after: ResourceState) -> AppResult<Capacity> {
    Ok(Capacity {
        identities: bool_delta(before.identity, after.identity),
        encrypted_resources: bool_delta(before.encrypted, after.encrypted),
        ciphertext_bytes: byte_delta(before.ciphertext_bytes, after.ciphertext_bytes)?,
        auxiliary_bytes: byte_delta(before.auxiliary_bytes, after.auxiliary_bytes)?,
    })
}

fn byte_delta(before: usize, after: usize) -> AppResult<i64> {
    let before = i64::try_from(before).map_err(|_| super::invalid_stored_value())?;
    let after = i64::try_from(after).map_err(|_| super::invalid_stored_value())?;
    after
        .checked_sub(before)
        .ok_or_else(super::invalid_stored_value)
}

fn ciphertext_state(identity: bool, value: Option<&Vec<u8>>) -> ResourceState {
    ResourceState {
        identity,
        encrypted: value.is_some(),
        ciphertext_bytes: value.map_or(0, Vec::len),
        auxiliary_bytes: 0,
    }
}

fn host_current(row: Option<&HostRow>) -> ResourceState {
    match row {
        None => ResourceState::default(),
        Some(value) if value.is_deleted => ciphertext_state(true, None),
        Some(value) => ciphertext_state(true, value.ciphertext.as_ref()),
    }
}

fn host_final(change: &ValidatedChange, current: Option<&HostRow>) -> ResourceState {
    match change.operation {
        HostOperation::Insert => {
            ciphertext_state(true, change.ciphertext.as_ref().and_then(Option::as_ref))
        }
        HostOperation::Update => change.ciphertext.as_ref().map_or_else(
            || host_current(current),
            |value| ciphertext_state(true, value.as_ref()),
        ),
        HostOperation::Delete => ciphertext_state(true, None),
    }
}

fn ai_current(row: Option<&AiRow>) -> AppResult<ResourceState> {
    match row {
        None => Ok(ResourceState::default()),
        Some(value) if value.is_deleted => Ok(ciphertext_state(true, None)),
        Some(value) => match (
            value.ciphertext.as_ref(),
            value.nonce.as_ref(),
            value.envelope_metadata.as_ref(),
        ) {
            (Some(ciphertext), Some(nonce), Some(metadata)) => Ok(ResourceState {
                identity: true,
                encrypted: true,
                ciphertext_bytes: ciphertext.len(),
                auxiliary_bytes: auxiliary_size(nonce, metadata)?,
            }),
            _ => Err(super::invalid_stored_value()),
        },
    }
}

fn ai_final(change: &ValidatedAiChange) -> AppResult<ResourceState> {
    match change.operation {
        AiProviderOperation::Insert | AiProviderOperation::Update => {
            let payload = change
                .payload
                .as_ref()
                .ok_or_else(super::invalid_stored_value)?;
            payload_state(payload)
        }
        AiProviderOperation::Delete => Ok(ciphertext_state(true, None)),
    }
}

fn payload_state(payload: &ValidatedAiPayload) -> AppResult<ResourceState> {
    Ok(ResourceState {
        identity: true,
        encrypted: true,
        ciphertext_bytes: payload.ciphertext.len(),
        auxiliary_bytes: auxiliary_size(&payload.nonce, &payload.envelope_metadata)?,
    })
}

fn auxiliary_size(nonce: &[u8], metadata: &Value) -> AppResult<usize> {
    canonical_ai_auxiliary_size(nonce, metadata).ok_or_else(super::invalid_stored_value)
}

const fn bool_delta(before: bool, after: bool) -> i64 {
    match (before, after) {
        (false, true) => 1,
        (true, false) => -1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use cloud_domain::AppError;

    use super::{Capacity, ResourceState, bool_delta, require_within_limit, resource_delta};

    const fn state(
        identity: bool,
        encrypted: bool,
        cipher: usize,
        auxiliary: usize,
    ) -> ResourceState {
        ResourceState {
            identity,
            encrypted,
            ciphertext_bytes: cipher,
            auxiliary_bytes: auxiliary,
        }
    }

    #[test]
    fn resource_transition_delta_counts_identity_ciphertext_and_auxiliary() {
        assert_eq!(bool_delta(false, true), 1);
        assert_eq!(bool_delta(true, false), -1);
        assert_eq!(
            resource_delta(state(false, false, 0, 0), state(true, true, 8, 3))
                .expect("insert delta"),
            Capacity {
                identities: 1,
                encrypted_resources: 1,
                ciphertext_bytes: 8,
                auxiliary_bytes: 3,
            }
        );
        assert_eq!(
            resource_delta(state(true, true, 8, 3), state(true, false, 0, 0)).expect("clear delta"),
            Capacity {
                identities: 0,
                encrypted_resources: -1,
                ciphertext_bytes: -8,
                auxiliary_bytes: -3,
            }
        );
    }

    #[test]
    fn every_final_capacity_overflow_uses_the_stable_error() {
        for capacity in [
            Capacity {
                identities: 5_001,
                ..Capacity::default()
            },
            Capacity {
                encrypted_resources: 2_001,
                ..Capacity::default()
            },
            Capacity {
                ciphertext_bytes: 32 * 1024 * 1024 + 1,
                ..Capacity::default()
            },
            Capacity {
                auxiliary_bytes: 512 * 1024 + 1,
                ..Capacity::default()
            },
        ] {
            assert!(matches!(
                require_within_limit(capacity),
                Err(AppError::SyncCapacityExceeded(_))
            ));
        }
    }
}
