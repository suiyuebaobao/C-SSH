//! Account-scoped encrypted-sync reset, generation fencing and idempotent replay.

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{ResetSyncRequest, ResetSyncResponse, SyncStateView, actor::DeviceActor};

use super::{DbTransaction, begin, commit, lock_sync_state, require_active_device, storage};

#[derive(FromRow)]
struct PriorReset {
    source_device_id: Uuid,
    result_generation: i64,
}

#[derive(FromRow)]
struct ResetCounts {
    removed_hosts: i64,
    removed_versions: i64,
    removed_conflicts: i64,
    removed_deliveries: i64,
    removed_ack_records: i64,
    removed_tombstones: i64,
}

pub(crate) async fn state(pool: &PgPool, actor: DeviceActor) -> AppResult<SyncStateView> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    commit(tx).await?;
    Ok(SyncStateView {
        sync_generation: state.sync_generation,
        current_revision: state.current_revision,
    })
}

pub(crate) async fn reset(
    pool: &PgPool,
    actor: DeviceActor,
    request: &ResetSyncRequest,
) -> AppResult<ResetSyncResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;

    if let Some(prior) = load_prior(&mut tx, actor.account_id(), request.mutation_id).await? {
        if prior.source_device_id != actor.device_id() {
            return Err(AppError::Conflict(
                "mutation_id was already used by another device".to_owned(),
            ));
        }
        commit(tx).await?;
        return Ok(response(prior.result_generation));
    }

    let next_generation = state
        .sync_generation
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict("sync_generation cannot advance".to_owned()))?;
    let counts = count_rows(&mut tx, actor.account_id()).await?;
    purge_account_data(&mut tx, actor.account_id()).await?;

    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET current_revision = 0, compacted_through_revision = 0,
             sync_generation = $2, updated_at = now()
         WHERE account_id = $1",
    )
    .bind(actor.account_id())
    .bind(next_generation)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;

    sqlx::query(
        "INSERT INTO cloud_sync_reset_mutations
             (account_id, mutation_id, source_device_id, result_generation)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(actor.account_id())
    .bind(request.mutation_id)
    .bind(actor.device_id())
    .bind(next_generation)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;

    audit_reset(
        &mut tx,
        actor,
        request.mutation_id,
        state.sync_generation,
        next_generation,
        &counts,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(next_generation))
}

async fn load_prior(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Option<PriorReset>> {
    sqlx::query_as::<_, PriorReset>(
        "SELECT source_device_id, result_generation
         FROM cloud_sync_reset_mutations
         WHERE account_id = $1 AND mutation_id = $2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

async fn count_rows(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<ResetCounts> {
    sqlx::query_as::<_, ResetCounts>(
        "SELECT
             (SELECT count(*)::BIGINT FROM cloud_hosts
               WHERE account_id = $1) AS removed_hosts,
             (SELECT count(*)::BIGINT FROM cloud_host_versions
               WHERE account_id = $1) AS removed_versions,
             (SELECT count(*)::BIGINT FROM cloud_host_conflicts
               WHERE account_id = $1) AS removed_conflicts,
             (SELECT count(*)::BIGINT FROM cloud_host_device_deliveries
               WHERE account_id = $1) AS removed_deliveries,
             ((SELECT count(*)::BIGINT FROM cloud_host_pull_watermarks
                WHERE account_id = $1)
              + (SELECT count(*)::BIGINT FROM cloud_host_pull_decisions
                  WHERE account_id = $1)
              + (SELECT count(*)::BIGINT FROM cloud_host_device_checkpoints
                  WHERE account_id = $1)) AS removed_ack_records,
             (SELECT count(*)::BIGINT FROM cloud_hosts
               WHERE account_id = $1 AND is_deleted) AS removed_tombstones",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)
}

async fn purge_account_data(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
    // Explicit ordering keeps every deletion account-scoped and makes the
    // reset independent of incidental cascade behavior.
    for statement in [
        "DELETE FROM cloud_host_pull_decisions WHERE account_id = $1",
        "DELETE FROM cloud_host_device_deliveries WHERE account_id = $1",
        "DELETE FROM cloud_host_pull_watermarks WHERE account_id = $1",
        "DELETE FROM cloud_host_device_checkpoints WHERE account_id = $1",
        "DELETE FROM cloud_sync_rekey_mutations WHERE account_id = $1",
        "DELETE FROM cloud_host_mutations WHERE account_id = $1",
        "DELETE FROM cloud_host_conflicts WHERE account_id = $1",
        "DELETE FROM cloud_host_versions WHERE account_id = $1",
        "DELETE FROM cloud_hosts WHERE account_id = $1",
        "DELETE FROM sync_device_checkpoints WHERE account_id = $1",
        "DELETE FROM sync_mutations WHERE account_id = $1",
        "DELETE FROM sync_conflicts WHERE account_id = $1",
        "DELETE FROM sync_record_versions WHERE account_id = $1",
        "DELETE FROM sync_records WHERE account_id = $1",
        "DELETE FROM sync_states WHERE account_id = $1",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .execute(&mut **tx)
            .await
            .map_err(storage)?;
    }
    Ok(())
}

async fn audit_reset(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    previous_generation: i64,
    next_generation: i64,
    counts: &ResetCounts,
) -> AppResult<()> {
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "removed_hosts": counts.removed_hosts,
        "removed_versions": counts.removed_versions,
        "removed_conflicts": counts.removed_conflicts,
        "removed_deliveries": counts.removed_deliveries,
        "removed_ack_records": counts.removed_ack_records,
        "removed_tombstones": counts.removed_tombstones,
        "previous_sync_generation": previous_generation,
        "sync_generation": next_generation
    });
    sqlx::query(
        "INSERT INTO audit_events
             (id, actor_account_id, action, resource_kind, resource_id,
              outcome, request_id, details)
         VALUES ($1, $2, 'sync.encrypted_data_reset', 'sync_account', $3,
                 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(actor.account_id().to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| AppError::Storage("failed to persist encrypted sync reset audit".to_owned()))?;
    Ok(())
}

fn response(sync_generation: i64) -> ResetSyncResponse {
    ResetSyncResponse {
        status: "reset".to_owned(),
        sync_generation,
    }
}
