//! 原子清理账号 Host 与 AI 密文资源，同时推进 generation 并保留幂等证据。

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    ResetSyncRequest, ResetSyncResponse, SyncGenerationTransition, SyncStateView,
    actor::DeviceActor,
};

use super::{
    DbTransaction, begin, commit, lock_sync_state, require_active_device, require_sync_generation,
    storage,
};

#[derive(FromRow)]
struct PriorReset {
    source_device_id: Uuid,
    result_generation: i64,
}

#[derive(FromRow)]
struct ResetCounts {
    removed_hosts: i64,
    removed_versions: i64,
    removed_ai_providers: i64,
    removed_ai_versions: i64,
    removed_deliveries: i64,
    removed_ack_records: i64,
    removed_tombstones: i64,
}

pub(crate) async fn state(pool: &PgPool, actor: DeviceActor) -> AppResult<SyncStateView> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    let secret_present = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1 FROM cloud_hosts
             WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL
         ) OR EXISTS(
             SELECT 1 FROM cloud_ai_provider_configs
             WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL
         )",
    )
    .bind(actor.account_id())
    .fetch_one(&mut *tx)
    .await
    .map_err(storage)?;
    let generation_transition =
        load_generation_transition(&mut tx, actor.account_id(), state.sync_generation).await?;
    commit(tx).await?;
    Ok(SyncStateView {
        sync_generation: state.sync_generation,
        current_revision: state.current_revision,
        compacted_through_revision: state.compacted_through_revision,
        generation_transition,
        secret_present,
    })
}

async fn load_generation_transition(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    sync_generation: i64,
) -> AppResult<SyncGenerationTransition> {
    let (reset_seen, rekey_seen) = sqlx::query_as::<_, (bool, bool)>(
        "SELECT
             EXISTS(SELECT 1 FROM cloud_sync_reset_mutations
                    WHERE account_id=$1 AND result_generation=$2)
             OR EXISTS(SELECT 1 FROM audit_events
                       WHERE actor_account_id=$1 AND outcome='success'
                         AND action IN ('sync.encrypted_data_reset',
                                        'sync.encrypted_data_reset_v2')
                         AND details->'sync_generation'=to_jsonb($2::BIGINT)),
             EXISTS(SELECT 1 FROM cloud_sync_rekey_mutations
                    WHERE account_id=$1 AND result_generation=$2)
             OR EXISTS(SELECT 1 FROM audit_events
                       WHERE actor_account_id=$1 AND outcome='success'
                         AND action IN ('sync.encrypted_data_rekey',
                                        'sync.encrypted_data_rekey_v2')
                         AND details->'sync_generation'=to_jsonb($2::BIGINT))",
    )
    .bind(account_id)
    .bind(sync_generation)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    classify_generation_transition(sync_generation, reset_seen, rekey_seen)
}

fn classify_generation_transition(
    sync_generation: i64,
    reset_seen: bool,
    rekey_seen: bool,
) -> AppResult<SyncGenerationTransition> {
    match (sync_generation, reset_seen, rekey_seen) {
        (1, false, false) => Ok(SyncGenerationTransition::Initial),
        (2.., true, false) => Ok(SyncGenerationTransition::Reset),
        (2.., false, true) => Ok(SyncGenerationTransition::Rekey),
        _ => Err(AppError::Storage(
            "current sync generation transition cannot be proven".to_owned(),
        )),
    }
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
        if request.sync_generation.checked_add(1) != Some(prior.result_generation) {
            return Err(AppError::Conflict(
                "mutation_id was already used by a different reset request".to_owned(),
            ));
        }
        if state.sync_generation != prior.result_generation {
            return Err(AppError::sync_generation_changed(
                "reset replay belongs to an older sync generation",
            ));
        }
        commit(tx).await?;
        return Ok(response(prior.result_generation));
    }

    require_sync_generation(state, request.sync_generation)?;

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
             (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs
               WHERE account_id = $1) AS removed_ai_providers,
             (SELECT count(*)::BIGINT FROM cloud_ai_provider_config_versions
               WHERE account_id = $1) AS removed_ai_versions,
             (SELECT count(*)::BIGINT FROM cloud_sync_resource_deliveries
               WHERE account_id = $1) AS removed_deliveries,
             ((SELECT count(*)::BIGINT FROM cloud_sync_pull_watermarks
                WHERE account_id = $1)
              + (SELECT count(*)::BIGINT FROM cloud_sync_pull_decisions
                  WHERE account_id = $1)
              + (SELECT count(*)::BIGINT FROM cloud_sync_device_checkpoints
                  WHERE account_id = $1)) AS removed_ack_records,
             ((SELECT count(*)::BIGINT FROM cloud_hosts
                WHERE account_id = $1 AND is_deleted)
              + (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs
                  WHERE account_id = $1 AND is_deleted)) AS removed_tombstones",
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
        "DELETE FROM cloud_sync_pull_decisions WHERE account_id = $1",
        "DELETE FROM cloud_sync_resource_deliveries WHERE account_id = $1",
        "DELETE FROM cloud_sync_pull_watermarks WHERE account_id = $1",
        "DELETE FROM cloud_sync_device_checkpoints WHERE account_id = $1",
        "DELETE FROM cloud_sync_rekey_mutations WHERE account_id = $1",
        "DELETE FROM cloud_sync_push_mutations WHERE account_id = $1",
        "DELETE FROM cloud_ai_provider_config_versions WHERE account_id = $1",
        "DELETE FROM cloud_ai_provider_configs WHERE account_id = $1",
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
        "removed_ai_providers": counts.removed_ai_providers,
        "removed_ai_versions": counts.removed_ai_versions,
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
         VALUES ($1, $2, 'sync.encrypted_data_reset_v2', 'sync_account', $3,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_transition_requires_one_durable_fact_after_initial_generation() {
        assert_eq!(
            classify_generation_transition(1, false, false).expect("initial generation"),
            SyncGenerationTransition::Initial
        );
        assert_eq!(
            classify_generation_transition(2, false, true).expect("rekey generation"),
            SyncGenerationTransition::Rekey
        );
        assert_eq!(
            classify_generation_transition(3, true, false).expect("reset generation"),
            SyncGenerationTransition::Reset
        );
        assert!(classify_generation_transition(2, false, false).is_err());
        assert!(classify_generation_transition(2, true, true).is_err());
    }
}
