//! Host metadata客户端迁移：显式读取旧值，原子改写opaque current/history与坐标。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    CURRENT_SYNC_CONTRACT_VERSION, HostMetadataInput, HostMetadataMigrationPreviewRecord,
    HostMetadataMigrationPreviewRequest, HostMetadataMigrationPreviewResponse,
    HostMetadataMigrationReceipt, HostMetadataMigrationRequest, HostStatus, actor::DeviceActor,
    validation::ValidatedHostMetadataMigration,
};

use super::{
    DbTransaction, SyncState, begin, capacity::require_host_metadata_migration_within_limit,
    commit, lock_sync_state, protection::clear_delivery_state, require_active_device,
    require_base_revision, require_configured_envelope, require_protection_version,
    require_sync_contract, require_sync_generation, storage,
};

#[derive(FromRow)]
struct PreviewRow {
    id: Uuid,
    address: Option<String>,
    port: Option<i32>,
    name: Option<String>,
    platform: Option<String>,
    tags: Option<Value>,
    status: Option<String>,
    ciphertext: Option<Vec<u8>>,
    source_device_id: Uuid,
    revision: i64,
    is_deleted: bool,
    metadata_encrypted: bool,
}

#[derive(FromRow)]
struct CurrentHost {
    id: Uuid,
    revision: i64,
    is_deleted: bool,
}

mod persistence;
use persistence::{audit, load_receipt, persist_receipt, receipt, validate_replay};

pub(crate) async fn preview(
    pool: &PgPool,
    actor: DeviceActor,
    request: HostMetadataMigrationPreviewRequest,
) -> AppResult<HostMetadataMigrationPreviewResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    require_sync_contract(state, request.sync_contract_version)?;
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.protection_epoch, request.protection_revision)?;
    require_configured_envelope(&mut tx, actor.account_id(), state).await?;
    if state.minimum_sync_contract_version >= i32::from(CURRENT_SYNC_CONTRACT_VERSION) {
        return Err(AppError::Conflict(
            "Host metadata已经完成opaque迁移".to_owned(),
        ));
    }
    let snapshot = request.snapshot_revision.unwrap_or(state.current_revision);
    require_base_revision(state, snapshot)?;
    let rows = sqlx::query_as::<_, PreviewRow>(
        "SELECT id,address,port,name,platform,tags,status,ciphertext,
                source_device_id,revision,is_deleted,metadata_encrypted
         FROM cloud_hosts
         WHERE account_id=$1 AND ($2::UUID IS NULL OR id>$2)
         ORDER BY id LIMIT $3",
    )
    .bind(actor.account_id())
    .bind(request.after_host_id)
    .bind(i64::from(request.limit) + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(storage)?;
    let has_more = rows.len() > request.limit as usize;
    let mut records = rows
        .into_iter()
        .take(request.limit as usize)
        .map(preview_record)
        .collect::<AppResult<Vec<_>>>()?;
    let next_after_host_id = has_more.then(|| {
        records
            .last()
            .expect("has_more requires one returned Host")
            .host_id
    });
    commit(tx).await?;
    Ok(HostMetadataMigrationPreviewResponse {
        sync_contract_version: CURRENT_SYNC_CONTRACT_VERSION,
        sync_generation: state.sync_generation,
        protection_epoch: state.protection_epoch,
        protection_revision: state.protection_revision,
        snapshot_revision: snapshot,
        records: std::mem::take(&mut records),
        next_after_host_id,
        has_more,
    })
}

pub(crate) async fn migrate(
    pool: &PgPool,
    actor: DeviceActor,
    request: &HostMetadataMigrationRequest,
    validated: &ValidatedHostMetadataMigration,
) -> AppResult<HostMetadataMigrationReceipt> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    if let Some(prior) = load_receipt(&mut tx, actor.account_id(), request.mutation_id).await? {
        validate_replay(&prior, actor, &validated.request_hash)?;
        commit(tx).await?;
        return receipt(prior, true);
    }
    require_sync_contract(state, request.sync_contract_version)?;
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.protection_epoch, request.protection_revision)?;
    require_configured_envelope(&mut tx, actor.account_id(), state).await?;
    require_base_revision(state, request.snapshot_revision)?;
    if state.minimum_sync_contract_version >= i32::from(CURRENT_SYNC_CONTRACT_VERSION) {
        return Err(AppError::Conflict(
            "Host metadata已经完成opaque迁移".to_owned(),
        ));
    }
    let current = lock_hosts(&mut tx, actor.account_id()).await?;
    require_complete(&current, &validated.hosts)?;
    let active_hosts = current.iter().filter(|host| !host.is_deleted).count();
    let host_ciphertext_bytes =
        current
            .iter()
            .zip(&validated.hosts)
            .try_fold(0_usize, |total, (stored, candidate)| {
                if stored.is_deleted != candidate.ciphertext.is_none() {
                    return Err(AppError::Validation(
                        "tombstone必须显式提交null ciphertext，活动Host必须提交密文".to_owned(),
                    ));
                }
                total
                    .checked_add(candidate.ciphertext.as_ref().map_or(0, Vec::len))
                    .ok_or_else(|| {
                        AppError::Validation("metadata migration密文总量过大".to_owned())
                    })
            })?;
    require_host_metadata_migration_within_limit(
        &mut tx,
        actor.account_id(),
        active_hosts,
        host_ciphertext_bytes,
    )
    .await?;
    let result_generation = next(state.sync_generation, "sync_generation")?;
    let result_protection_revision = next(state.protection_revision, "protection_revision")?;
    let mut revision = state.current_revision;

    sqlx::query("DELETE FROM cloud_host_versions WHERE account_id=$1")
        .bind(actor.account_id())
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    let mut results = Vec::with_capacity(current.len());
    for (stored, candidate) in current.iter().zip(&validated.hosts) {
        revision = next(revision, "current_revision")?;
        write_opaque_host(
            &mut tx,
            actor,
            stored.id,
            revision,
            candidate.ciphertext.as_deref(),
            stored.is_deleted,
        )
        .await?;
        results.push((stored.id, stored.revision, revision));
    }
    replace_envelope(
        &mut tx,
        actor,
        state,
        result_generation,
        result_protection_revision,
        validated,
    )
    .await?;
    clear_delivery_state(&mut tx, actor.account_id()).await?;
    update_state(
        &mut tx,
        actor.account_id(),
        result_generation,
        state.protection_epoch,
        result_protection_revision,
        revision,
    )
    .await?;
    let host_count = i32::try_from(results.len()).map_err(|_| {
        AppError::SyncCapacityExceeded("metadata migration Host数量过大".to_owned())
    })?;
    let receipt_row = persist_receipt(
        &mut tx,
        actor,
        request,
        validated.request_hash,
        result_generation,
        result_protection_revision,
        revision,
        host_count,
        &results,
    )
    .await?;
    audit(
        &mut tx,
        actor,
        request.mutation_id,
        state,
        result_generation,
        result_protection_revision,
        revision,
        host_count,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    receipt(receipt_row, false)
}

pub(crate) async fn get_receipt(
    pool: &PgPool,
    actor: DeviceActor,
    mutation_id: Uuid,
) -> AppResult<HostMetadataMigrationReceipt> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let row = load_receipt(&mut tx, actor.account_id(), mutation_id)
        .await?
        .ok_or_else(|| AppError::NotFound("metadata migration receipt不存在".to_owned()))?;
    commit(tx).await?;
    receipt(row, true)
}

fn preview_record(row: PreviewRow) -> AppResult<HostMetadataMigrationPreviewRecord> {
    let legacy_metadata = if row.metadata_encrypted {
        if row.address.is_some()
            || row.port.is_some()
            || row.name.is_some()
            || row.platform.is_some()
            || row.tags.is_some()
            || row.status.is_some()
        {
            return Err(super::invalid_stored_value());
        }
        None
    } else {
        Some(HostMetadataInput {
            address: row.address.ok_or_else(super::invalid_stored_value)?,
            port: u16::try_from(row.port.ok_or_else(super::invalid_stored_value)?)
                .map_err(|_| super::invalid_stored_value())?,
            name: row.name.ok_or_else(super::invalid_stored_value)?,
            platform: row.platform.ok_or_else(super::invalid_stored_value)?,
            tags: serde_json::from_value(row.tags.ok_or_else(super::invalid_stored_value)?)
                .map_err(|_| super::invalid_stored_value())?,
            status: HostStatus::parse(&row.status.ok_or_else(super::invalid_stored_value)?)
                .ok_or_else(super::invalid_stored_value)?,
        })
    };
    Ok(HostMetadataMigrationPreviewRecord {
        host_id: row.id,
        cloud_revision: row.revision,
        source_device_id: row.source_device_id,
        deleted: row.is_deleted,
        metadata_encrypted: row.metadata_encrypted,
        legacy_metadata,
        ciphertext: row.ciphertext.map(|value| STANDARD.encode(value)),
    })
}

async fn lock_hosts(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<Vec<CurrentHost>> {
    sqlx::query_as(
        "SELECT id,revision,is_deleted FROM cloud_hosts
         WHERE account_id=$1 ORDER BY id FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)
}

fn require_complete(
    current: &[CurrentHost],
    candidates: &[crate::validation::ValidatedHostMetadataCandidate],
) -> AppResult<()> {
    if current.len() == candidates.len()
        && current.iter().zip(candidates).all(|(stored, candidate)| {
            stored.id == candidate.host_id && stored.revision == candidate.cloud_revision
        })
    {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "metadata migration候选不是冻结快照的完整current Host集合".to_owned(),
        ))
    }
}

async fn write_opaque_host(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    host_id: Uuid,
    revision: i64,
    ciphertext: Option<&[u8]>,
    deleted: bool,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE cloud_hosts
         SET address=NULL,port=NULL,name=NULL,platform=NULL,tags=NULL,status=NULL,
             ciphertext=$3,source_device_id=$4,revision=$5,is_deleted=$6,
             metadata_encrypted=TRUE,updated_at=now()
         WHERE account_id=$1 AND id=$2",
    )
    .bind(actor.account_id())
    .bind(host_id)
    .bind(ciphertext)
    .bind(actor.device_id())
    .bind(revision)
    .bind(deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO cloud_host_versions
             (account_id,host_id,revision,address,port,name,platform,tags,status,
              ciphertext,source_device_id,is_deleted,metadata_encrypted)
         VALUES ($1,$2,$3,NULL,NULL,NULL,NULL,NULL,NULL,$4,$5,$6,TRUE)",
    )
    .bind(actor.account_id())
    .bind(host_id)
    .bind(revision)
    .bind(ciphertext)
    .bind(actor.device_id())
    .bind(deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn replace_envelope(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    source: SyncState,
    generation: i64,
    revision: i64,
    validated: &ValidatedHostMetadataMigration,
) -> AppResult<()> {
    let changed = sqlx::query(
        "UPDATE cloud_data_protection_envelopes
         SET sync_generation=$5,protection_revision=$6,
             salt=$7,nonce=$8,wrapped_data_key=$9,source_device_id=$10,updated_at=now()
         WHERE account_id=$1 AND sync_generation=$2 AND protection_epoch=$3
           AND protection_revision=$4",
    )
    .bind(actor.account_id())
    .bind(source.sync_generation)
    .bind(source.protection_epoch)
    .bind(source.protection_revision)
    .bind(generation)
    .bind(revision)
    .bind(&validated.envelope.salt)
    .bind(&validated.envelope.nonce)
    .bind(&validated.envelope.wrapped_data_key)
    .bind(actor.device_id())
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    if changed.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "metadata migration目标envelope源坐标已变化".to_owned(),
        ))
    }
}

async fn update_state(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    generation: i64,
    epoch: i64,
    protection_revision: i64,
    current_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET sync_generation=$2,protection_epoch=$3,protection_revision=$4,
             current_revision=$5,minimum_sync_contract_version=4,updated_at=now()
         WHERE account_id=$1",
    )
    .bind(account_id)
    .bind(generation)
    .bind(epoch)
    .bind(protection_revision)
    .bind(current_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

fn next(value: i64, field: &str) -> AppResult<i64> {
    value
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict(format!("{field}不能继续推进")))
}
