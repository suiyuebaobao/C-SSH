//! Account-scoped incremental pulls and cloud-side acknowledgement bookkeeping.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    HostStatus, PullAckRequest, PullHostRecord, PullRequest, PullResponse, actor::DeviceActor,
};

use super::{
    DbTransaction, begin, commit, invalid_stored_value, lock_sync_state, require_active_device,
    require_sync_generation, storage,
};

#[derive(FromRow)]
struct PullRow {
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

pub(crate) async fn pull(
    pool: &PgPool,
    actor: DeviceActor,
    request: PullRequest,
) -> AppResult<PullResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    require_sync_generation(state, request.sync_generation)?;
    if request.since_revision < state.compacted_through_revision {
        return Err(AppError::SyncResyncRequired(
            "the requested host revision is no longer available".to_owned(),
        ));
    }
    let snapshot = request.snapshot_revision.unwrap_or(state.current_revision);
    if snapshot > state.current_revision {
        return Err(AppError::Conflict(
            "snapshot_revision is newer than the account host revision".to_owned(),
        ));
    }

    let fetch_limit = i64::from(request.limit) + 1;
    let mut rows = sqlx::query_as::<_, PullRow>(
        "WITH latest AS (
             SELECT DISTINCT ON (versions.host_id)
                     versions.host_id, versions.revision, versions.address,
                     versions.port, versions.name, versions.platform,
                     versions.tags, versions.status, versions.ciphertext,
                     versions.source_device_id,
                     versions.is_deleted, versions.recorded_at
             FROM cloud_host_versions AS versions
             WHERE versions.account_id = $1
               AND versions.revision <= $3
             ORDER BY versions.host_id, versions.revision DESC
         )
         SELECT latest.host_id, latest.revision, latest.address, latest.port,
                latest.name, latest.platform, latest.tags, latest.status,
                latest.ciphertext, latest.source_device_id, latest.is_deleted,
                latest.recorded_at
         FROM latest
         WHERE (latest.revision > $4
                OR NOT EXISTS (
                    SELECT 1
                    FROM cloud_host_device_deliveries AS delivery
                    WHERE delivery.account_id = $1
                      AND delivery.device_id = $2
                      AND delivery.host_id = latest.host_id
                      AND delivery.delivered_revision = latest.revision
                ))
           AND ($5::BIGINT IS NULL
                OR (latest.revision, latest.host_id) > ($5, $6))
         ORDER BY latest.revision, latest.host_id
         LIMIT $7",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot)
    .bind(request.since_revision)
    .bind(request.after_revision)
    .bind(request.after_host_id)
    .bind(fetch_limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(storage)?;
    let has_more = rows.len() > request.limit as usize;
    if has_more {
        rows.pop();
    }
    let cursor = rows.last().map(|row| (row.revision, row.host_id));
    let next_revision = if has_more {
        cursor.map_or(request.since_revision, |value| value.0)
    } else {
        snapshot
    };
    record_deliveries(&mut tx, actor, &rows).await?;
    record_pull_watermark(&mut tx, actor, next_revision, snapshot).await?;
    let records = rows
        .into_iter()
        .map(to_record)
        .collect::<AppResult<Vec<_>>>()?;
    commit(tx).await?;
    Ok(PullResponse {
        sync_generation: state.sync_generation,
        records,
        snapshot_revision: snapshot,
        next_revision,
        next_after_host_id: has_more.then(|| cursor.map(|value| value.1)).flatten(),
        has_more,
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
    if request.acknowledged_revision < state.compacted_through_revision {
        return Err(AppError::SyncResyncRequired(
            "the acknowledged host revision is no longer available".to_owned(),
        ));
    }
    if request.acknowledged_revision > state.current_revision {
        return Err(AppError::Conflict(
            "acknowledged_revision is newer than the account host revision".to_owned(),
        ));
    }
    require_delivered_watermark(&mut tx, actor, request.acknowledged_revision).await?;
    let prior = sqlx::query_scalar::<_, i64>(
        "SELECT acknowledged_revision
         FROM cloud_host_device_checkpoints
         WHERE account_id = $1 AND device_id = $2
         FOR UPDATE",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .unwrap_or(0);
    if request.acknowledged_revision < prior {
        return Err(AppError::Conflict(
            "device host acknowledgement cannot move backwards".to_owned(),
        ));
    }

    for decision in &request.decisions {
        require_delivered_identity(&mut tx, actor, decision.host_id, decision.cloud_revision)
            .await?;
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT action
             FROM cloud_host_pull_decisions
             WHERE account_id = $1 AND device_id = $2
               AND host_id = $3 AND cloud_revision = $4",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(decision.host_id)
        .bind(decision.cloud_revision)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?;
        if existing
            .as_deref()
            .is_some_and(|action| action != decision.action.as_str())
        {
            return Err(AppError::Conflict(
                "a different local decision was already recorded".to_owned(),
            ));
        }
        sqlx::query(
            "INSERT INTO cloud_host_pull_decisions
                 (account_id, device_id, host_id, cloud_revision, action)
             VALUES ($1,$2,$3,$4,$5)
             ON CONFLICT (account_id, device_id, host_id, cloud_revision)
             DO NOTHING",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(decision.host_id)
        .bind(decision.cloud_revision)
        .bind(decision.action.as_str())
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }

    sqlx::query(
        "INSERT INTO cloud_host_device_checkpoints
             (account_id, device_id, acknowledged_revision,
              last_manual_sync_at, updated_at)
         VALUES ($1,$2,$3,now(),now())
         ON CONFLICT (account_id, device_id) DO UPDATE SET
             acknowledged_revision =
                 GREATEST(cloud_host_device_checkpoints.acknowledged_revision,
                          EXCLUDED.acknowledged_revision),
             last_manual_sync_at = now(), updated_at = now(),
             admin_deleted_at = NULL",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(request.acknowledged_revision)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    commit(tx).await
}

fn to_record(row: PullRow) -> AppResult<PullHostRecord> {
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

async fn require_delivered_identity(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    host_id: Uuid,
    revision: i64,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1
             FROM cloud_host_device_deliveries
             WHERE account_id = $1
               AND device_id = $2
               AND host_id = $3
               AND delivered_revision = $4
         )",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(host_id)
    .bind(revision)
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

async fn record_deliveries(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    rows: &[PullRow],
) -> AppResult<()> {
    for row in rows {
        sqlx::query(
            "INSERT INTO cloud_host_device_deliveries
                 (account_id, device_id, host_id, delivered_revision, updated_at)
             VALUES ($1,$2,$3,$4,now())
             ON CONFLICT (account_id, device_id, host_id, delivered_revision)
             DO UPDATE SET updated_at = now()",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(row.host_id)
        .bind(row.revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn record_pull_watermark(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledgeable_revision: i64,
    snapshot_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_host_pull_watermarks
             (account_id, device_id, acknowledgeable_revision,
              snapshot_revision, delivered_at)
         VALUES ($1,$2,$3,$4,now())
         ON CONFLICT (account_id, device_id, acknowledgeable_revision)
         DO UPDATE SET
             snapshot_revision =
                 GREATEST(cloud_host_pull_watermarks.snapshot_revision,
                          EXCLUDED.snapshot_revision),
             delivered_at = now()",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledgeable_revision)
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn require_delivered_watermark(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledged_revision: i64,
) -> AppResult<()> {
    let delivered = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1
             FROM cloud_host_pull_watermarks
             WHERE account_id = $1
               AND device_id = $2
               AND acknowledgeable_revision = $3
         )",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledged_revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if !delivered {
        return Err(AppError::Validation(
            "the acknowledgement does not match a delivered pull watermark".to_owned(),
        ));
    }
    Ok(())
}
