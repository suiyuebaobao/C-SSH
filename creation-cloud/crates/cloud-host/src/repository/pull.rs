//! 在统一 revision 快照中拉取 Host 与 AI 资源，并记录类型化交付和选择性确认。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    HostStatus, PullAckRequest, PullAiProviderRecord, PullHostRecord, PullMode, PullPurpose,
    PullRequest, PullResponse, ResourceKind, actor::DeviceActor,
};

use super::{
    DbTransaction, begin, commit, invalid_stored_value, lock_sync_state, require_active_device,
    require_configured_envelope, require_protection_version, require_retained_revision,
    require_sync_generation, storage,
};

mod completion;
pub(super) use completion::record_rekey_snapshot;

#[derive(Clone, FromRow)]
struct PullIdentityRow {
    resource_kind: String,
    resource_id: Uuid,
    revision: i64,
}

#[derive(FromRow)]
struct HostVersionRow {
    host_id: Uuid,
    revision: i64,
    address: String,
    port: i32,
    name: String,
    platform: String,
    tags: Value,
    status: String,
    ciphertext: Option<Vec<u8>>,
    source_device_id: Uuid,
    is_deleted: bool,
    recorded_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct AiVersionRow {
    resource_id: Uuid,
    revision: i64,
    ciphertext: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    envelope_metadata: Option<Value>,
    source_device_id: Uuid,
    is_deleted: bool,
    recorded_at: DateTime<Utc>,
}
pub(crate) async fn pull(
    pool: &PgPool,
    actor: DeviceActor,
    request: PullRequest,
) -> AppResult<PullResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.protection_epoch, request.protection_revision)?;
    require_configured_envelope(&mut tx, actor.account_id(), state).await?;
    let snapshot = request.snapshot_revision.unwrap_or(state.current_revision);
    match request.mode {
        PullMode::Incremental => require_retained_revision(state, request.since_revision)?,
        PullMode::Full => require_retained_revision(state, snapshot)?,
    }
    if snapshot > state.current_revision {
        return Err(AppError::Conflict(
            "snapshot_revision is newer than the account revision".to_owned(),
        ));
    }

    let mut identities = load_identities(&mut tx, actor, &request, snapshot).await?;
    let has_more = identities.len() > request.limit as usize;
    if has_more {
        identities.pop();
    }
    let cursor = identities
        .last()
        .map_or(request.after_revision.unwrap_or(0), |row| row.revision);
    let next_revision = if has_more { cursor } else { snapshot };
    let mut host_records = Vec::new();
    let mut ai_records = Vec::new();
    for identity in &identities {
        match ResourceKind::parse(&identity.resource_kind).ok_or_else(invalid_stored_value)? {
            ResourceKind::Host => host_records.push(
                load_host_record(
                    &mut tx,
                    actor.account_id(),
                    identity.resource_id,
                    identity.revision,
                )
                .await?,
            ),
            ResourceKind::AiProviderAccount => ai_records.push(
                load_ai_record(
                    &mut tx,
                    actor.account_id(),
                    identity.resource_id,
                    identity.revision,
                )
                .await?,
            ),
        }
    }
    if request.purpose == PullPurpose::Download {
        prune_delivery_window(&mut tx, actor, snapshot).await?;
        record_deliveries(&mut tx, actor, &identities, snapshot).await?;
        record_pull_watermark(&mut tx, actor, next_revision, snapshot).await?;
    }
    commit(tx).await?;
    Ok(PullResponse {
        sync_generation: state.sync_generation,
        protection_epoch: state.protection_epoch,
        protection_revision: state.protection_revision,
        purpose: request.purpose,
        mode: request.mode,
        host_records,
        ai_records,
        snapshot_revision: snapshot,
        next_revision,
        has_more,
    })
}
async fn load_identities(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &PullRequest,
    snapshot: i64,
) -> AppResult<Vec<PullIdentityRow>> {
    let fetch_limit = i64::from(request.limit) + 1;
    sqlx::query_as::<_, PullIdentityRow>(
        "WITH resources AS (
             SELECT 'host'::TEXT AS resource_kind,
                    latest.host_id AS resource_id, latest.revision
             FROM (
                 SELECT DISTINCT ON (versions.host_id)
                        versions.host_id, versions.revision
                 FROM cloud_host_versions AS versions
                 WHERE versions.account_id = $1 AND versions.revision <= $3
                 ORDER BY versions.host_id, versions.revision DESC
             ) AS latest
             UNION ALL
             SELECT 'ai_provider_account'::TEXT AS resource_kind,
                    latest.resource_id, latest.revision
             FROM (
                 SELECT DISTINCT ON (versions.resource_id)
                        versions.resource_id, versions.revision
                 FROM cloud_ai_provider_config_versions AS versions
                 WHERE versions.account_id = $1 AND versions.revision <= $3
                 ORDER BY versions.resource_id, versions.revision DESC
             ) AS latest
         )
         SELECT resources.resource_kind, resources.resource_id, resources.revision
         FROM resources
         WHERE ($4
                OR resources.revision > $5
                OR NOT EXISTS (
                    SELECT 1 FROM cloud_sync_pull_decisions AS decision
                    WHERE decision.account_id = $1 AND decision.device_id = $2
                      AND decision.resource_kind = resources.resource_kind
                      AND decision.resource_id = resources.resource_id
                      AND decision.revision = resources.revision
                ))
           AND ($6::BIGINT IS NULL OR resources.revision > $6)
         ORDER BY resources.revision
         LIMIT $7",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot)
    .bind(request.mode == PullMode::Full)
    .bind(request.since_revision)
    .bind(request.after_revision)
    .bind(fetch_limit)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)
}
async fn load_host_record(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    host_id: Uuid,
    revision: i64,
) -> AppResult<PullHostRecord> {
    let row = sqlx::query_as::<_, HostVersionRow>(
        "SELECT host_id, revision, address, port, name, platform, tags, status,
                ciphertext, source_device_id, is_deleted, recorded_at
         FROM cloud_host_versions
         WHERE account_id = $1 AND host_id = $2 AND revision = $3",
    )
    .bind(account_id)
    .bind(host_id)
    .bind(revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(PullHostRecord {
        host_id: row.host_id,
        revision: row.revision,
        address: row.address,
        port: u16::try_from(row.port).map_err(|_| invalid_stored_value())?,
        name: row.name,
        platform: row.platform,
        tags: serde_json::from_value(row.tags).map_err(|_| invalid_stored_value())?,
        status: HostStatus::parse(&row.status).ok_or_else(invalid_stored_value)?,
        ciphertext: row.ciphertext.map(|value| STANDARD.encode(value)),
        source_device_id: row.source_device_id,
        deleted: row.is_deleted,
        updated_at: row.recorded_at,
    })
}

async fn load_ai_record(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    resource_id: Uuid,
    revision: i64,
) -> AppResult<PullAiProviderRecord> {
    let row = sqlx::query_as::<_, AiVersionRow>(
        "SELECT resource_id, revision, ciphertext, nonce, envelope_metadata,
                source_device_id, is_deleted, recorded_at
         FROM cloud_ai_provider_config_versions
         WHERE account_id = $1 AND resource_id = $2 AND revision = $3",
    )
    .bind(account_id)
    .bind(resource_id)
    .bind(revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(PullAiProviderRecord {
        resource_id: row.resource_id,
        revision: row.revision,
        ciphertext: row.ciphertext.map(|value| STANDARD.encode(value)),
        nonce: row.nonce.map(|value| STANDARD.encode(value)),
        envelope_metadata: row.envelope_metadata,
        source_device_id: row.source_device_id,
        deleted: row.is_deleted,
        updated_at: row.recorded_at,
    })
}

pub(crate) async fn ack(
    pool: &PgPool,
    actor: DeviceActor,
    request: &PullAckRequest,
) -> AppResult<()> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.protection_epoch, request.protection_revision)?;
    require_configured_envelope(&mut tx, actor.account_id(), state).await?;
    require_retained_revision(state, request.acknowledged_revision)?;
    if request.acknowledged_revision > state.current_revision {
        return Err(AppError::Conflict(
            "acknowledged_revision is newer than the account revision".to_owned(),
        ));
    }
    let delivered_snapshot =
        delivered_watermark(&mut tx, actor, request.acknowledged_revision).await?;
    if delivered_snapshot.is_none() {
        if acknowledgement_already_applied(&mut tx, actor, request).await? {
            commit(tx).await?;
            return Ok(());
        }
        return Err(AppError::Validation(
            "the acknowledgement does not match a delivered pull watermark".to_owned(),
        ));
    }
    let prior = load_checkpoint(&mut tx, actor).await?;
    if request.acknowledged_revision < prior {
        return Err(AppError::Conflict(
            "device acknowledgement cannot move backwards".to_owned(),
        ));
    }

    for decision in &request.decisions {
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT action FROM cloud_sync_pull_decisions
             WHERE account_id = $1 AND device_id = $2
               AND resource_kind = $3 AND resource_id = $4 AND revision = $5",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(decision.resource_kind.as_str())
        .bind(decision.resource_id)
        .bind(decision.cloud_revision)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?;
        if let Some(action) = existing.as_deref() {
            if action == decision.action.as_str() {
                continue;
            }
            return Err(AppError::Conflict(
                "a different local decision was already recorded".to_owned(),
            ));
        }
        require_delivered_identity(
            &mut tx,
            actor,
            decision.resource_kind,
            decision.resource_id,
            decision.cloud_revision,
            request.acknowledged_revision,
        )
        .await?;
        sqlx::query(
            "INSERT INTO cloud_sync_pull_decisions
                 (account_id, device_id, resource_kind, resource_id, revision, action)
             VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (account_id, device_id, resource_kind, resource_id, revision)
             DO NOTHING",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(decision.resource_kind.as_str())
        .bind(decision.resource_id)
        .bind(decision.cloud_revision)
        .bind(decision.action.as_str())
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }
    let safe_revision =
        safe_checkpoint_revision(&mut tx, actor, request.acknowledged_revision).await?;
    save_checkpoint(&mut tx, actor, safe_revision).await?;
    clear_delivery_snapshot(
        &mut tx,
        actor,
        delivered_snapshot.expect("checked delivered snapshot"),
    )
    .await?;
    completion::record_download_completed(&mut tx, actor).await?;
    commit(tx).await
}

async fn acknowledgement_already_applied(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &PullAckRequest,
) -> AppResult<bool> {
    if request.decisions.is_empty() {
        return load_checkpoint(tx, actor)
            .await
            .map(|checkpoint| checkpoint >= request.acknowledged_revision);
    }
    for decision in &request.decisions {
        let action = sqlx::query_scalar::<_, String>(
            "SELECT action FROM cloud_sync_pull_decisions
             WHERE account_id=$1 AND device_id=$2 AND resource_kind=$3
               AND resource_id=$4 AND revision=$5",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(decision.resource_kind.as_str())
        .bind(decision.resource_id)
        .bind(decision.cloud_revision)
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage)?;
        if action.as_deref() != Some(decision.action.as_str()) {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) async fn safe_checkpoint_revision(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledged_revision: i64,
) -> AppResult<i64> {
    let first_unresolved = sqlx::query_scalar::<_, Option<i64>>(
        "WITH resources AS (
             SELECT 'host'::TEXT AS resource_kind,
                    latest.host_id AS resource_id, latest.revision
             FROM (
                 SELECT DISTINCT ON (versions.host_id)
                        versions.host_id, versions.revision
                 FROM cloud_host_versions AS versions
                 WHERE versions.account_id = $1 AND versions.revision <= $3
                 ORDER BY versions.host_id, versions.revision DESC
             ) AS latest
             UNION ALL
             SELECT 'ai_provider_account'::TEXT AS resource_kind,
                    latest.resource_id, latest.revision
             FROM (
                 SELECT DISTINCT ON (versions.resource_id)
                        versions.resource_id, versions.revision
                 FROM cloud_ai_provider_config_versions AS versions
                 WHERE versions.account_id = $1 AND versions.revision <= $3
                 ORDER BY versions.resource_id, versions.revision DESC
             ) AS latest
         )
         SELECT MIN(resources.revision)
         FROM resources
         WHERE NOT EXISTS (
             SELECT 1 FROM cloud_sync_pull_decisions AS decision
             WHERE decision.account_id = $1 AND decision.device_id = $2
               AND decision.resource_kind = resources.resource_kind
               AND decision.resource_id = resources.resource_id
               AND decision.revision = resources.revision
         )",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledged_revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(first_unresolved.map_or(acknowledged_revision, |revision| revision - 1))
}

async fn load_checkpoint(tx: &mut DbTransaction<'_>, actor: DeviceActor) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT acknowledged_revision FROM cloud_sync_device_checkpoints
         WHERE account_id = $1 AND device_id = $2 FOR UPDATE",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
    .map(|value| value.unwrap_or(0))
}

pub(super) async fn save_checkpoint(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledged_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_sync_device_checkpoints
             (account_id, device_id, acknowledged_revision, last_manual_sync_at, updated_at)
         VALUES ($1,$2,$3,now(),now())
         ON CONFLICT (account_id, device_id) DO UPDATE SET
             acknowledged_revision = GREATEST(
                 cloud_sync_device_checkpoints.acknowledged_revision,
                 EXCLUDED.acknowledged_revision),
             last_manual_sync_at = now(), updated_at = now(), admin_deleted_at = NULL",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledged_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn require_delivered_identity(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    kind: ResourceKind,
    resource_id: Uuid,
    revision: i64,
    acknowledged_revision: i64,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1
             FROM cloud_sync_resource_deliveries AS delivery
             JOIN cloud_sync_pull_watermarks AS watermark
               ON watermark.account_id = delivery.account_id
              AND watermark.device_id = delivery.device_id
              AND watermark.snapshot_revision = delivery.snapshot_revision
             WHERE delivery.account_id = $1 AND delivery.device_id = $2
               AND delivery.resource_kind = $3 AND delivery.resource_id = $4
               AND delivery.delivered_revision = $5
               AND watermark.acknowledgeable_revision = $6)",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(kind.as_str())
    .bind(resource_id)
    .bind(revision)
    .bind(acknowledged_revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if exists {
        Ok(())
    } else {
        Err(AppError::Validation(
            "a pull decision references a record unavailable to this device".to_owned(),
        ))
    }
}
include!("pull/delivery.rs");
