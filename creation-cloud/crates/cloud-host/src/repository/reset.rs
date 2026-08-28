//! Destructive Cloud-only reset with monotonic protection counters and strict idempotency.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult, mark_semantic_audit_recorded};
use cloud_notification::{AccountNotificationEvent, record_account_event};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{ResetAuthorization, ResetSyncRequest, ResetSyncResponse, actor::DeviceActor};

use super::protection::{
    DataProtectionOperation, PriorProtectionMutation, audit_mutation, consume_email_authorization,
    load_prior_mutation, persist_mutation,
};
use super::{
    DbTransaction, SyncState, begin, commit, lock_sync_state, require_active_device,
    require_base_revision, require_protection_version, require_sync_generation, storage,
};

#[derive(FromRow)]
struct ResetCounts {
    removed_hosts: i64,
    removed_ai_providers: i64,
}

pub(crate) async fn reset(
    pool: &PgPool,
    actor: DeviceActor,
    request: &ResetSyncRequest,
    request_hash: &[u8; 32],
    verification_key: &[u8],
) -> AppResult<ResetSyncResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    if let Some(prior) =
        load_prior_mutation(&mut tx, actor.account_id(), request.mutation_id).await?
    {
        validate_replay(&prior, actor, request, request_hash, state)?;
        commit(tx).await?;
        return Ok(response(
            prior.result_generation,
            prior.result_epoch,
            prior.result_revision,
            true,
        ));
    }
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.expected_epoch, request.expected_revision)?;
    require_base_revision(state, request.current_revision)?;
    authorize_reset(
        &mut tx,
        actor,
        &request.authorization,
        state,
        verification_key,
    )
    .await?;

    let result_generation = next(state.sync_generation, "sync_generation")?;
    let result_epoch = next(state.protection_epoch, "protection_epoch")?;
    let result_revision = next(state.protection_revision, "protection_revision")?;
    let counts = count_rows(&mut tx, actor.account_id()).await?;
    purge_account_data(&mut tx, actor.account_id()).await?;
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET current_revision=0, compacted_through_revision=0,
             sync_generation=$2, protection_epoch=$3, protection_revision=$4,
             updated_at=now()
         WHERE account_id=$1",
    )
    .bind(actor.account_id())
    .bind(result_generation)
    .bind(result_epoch)
    .bind(result_revision)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    persist_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Reset,
        state,
        request_hash,
        result_generation,
        result_epoch,
        result_revision,
        0,
        0,
    )
    .await?;
    audit_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Reset,
        request.authorization.audit_mode(),
        state,
        result_generation,
        result_epoch,
        result_revision,
        0,
        0,
        counts.removed_hosts + counts.removed_ai_providers,
    )
    .await?;
    record_account_event(
        &mut tx,
        actor.account_id(),
        AccountNotificationEvent::SyncResetCompleted {
            mutation_id: request.mutation_id,
        },
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(
        result_generation,
        result_epoch,
        result_revision,
        false,
    ))
}

async fn authorize_reset(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    authorization: &ResetAuthorization,
    state: SyncState,
    verification_key: &[u8],
) -> AppResult<()> {
    match authorization {
        ResetAuthorization::KnownPasswordClientConfirmation => {
            let has_protected_data = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(
                     SELECT 1 FROM cloud_data_protection_envelopes WHERE account_id=$1
                 ) OR EXISTS(
                     SELECT 1 FROM cloud_hosts
                     WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL
                 ) OR EXISTS(
                     SELECT 1 FROM cloud_ai_provider_configs
                     WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL
                 )",
            )
            .bind(actor.account_id())
            .fetch_one(&mut **tx)
            .await
            .map_err(storage)?;
            if has_protected_data {
                Ok(())
            } else {
                Err(AppError::Conflict(
                    "there is no protected Cloud data to reset".to_owned(),
                ))
            }
        }
        ResetAuthorization::EmailVerification {
            challenge_id,
            authorization_token,
        } => {
            let mut token = STANDARD
                .decode(authorization_token)
                .map_err(|_| AppError::Validation("reset authorization is invalid".to_owned()))?;
            let result = consume_email_authorization(
                tx,
                actor,
                *challenge_id,
                state,
                &token,
                verification_key,
            )
            .await;
            token.fill(0);
            result
        }
    }
}

fn validate_replay(
    prior: &PriorProtectionMutation,
    actor: DeviceActor,
    request: &ResetSyncRequest,
    request_hash: &[u8; 32],
    state: SyncState,
) -> AppResult<()> {
    if prior.operation != DataProtectionOperation::Reset.as_str()
        || prior.source_device_id != actor.device_id()
        || prior.request_generation != request.sync_generation
        || prior.request_epoch != request.expected_epoch
        || prior.request_revision != request.expected_revision
        || prior.request_current_revision != request.current_revision
        || prior.request_hash.as_slice() != request_hash
    {
        return Err(AppError::Conflict(
            "mutation_id was already used by a different reset request".to_owned(),
        ));
    }
    if state.sync_generation == prior.result_generation
        && state.protection_epoch == prior.result_epoch
        && state.protection_revision == prior.result_revision
        && state.current_revision == 0
    {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "reset replay belongs to an older protection state".to_owned(),
        ))
    }
}

async fn count_rows(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<ResetCounts> {
    sqlx::query_as(
        "SELECT
             (SELECT count(*)::BIGINT FROM cloud_hosts WHERE account_id=$1)
                 AS removed_hosts,
             (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs WHERE account_id=$1)
                 AS removed_ai_providers",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)
}

async fn purge_account_data(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
    for statement in [
        "DELETE FROM cloud_sync_pull_decisions WHERE account_id=$1",
        "DELETE FROM cloud_sync_resource_deliveries WHERE account_id=$1",
        "DELETE FROM cloud_sync_pull_watermarks WHERE account_id=$1",
        "DELETE FROM cloud_sync_device_checkpoints WHERE account_id=$1",
        "DELETE FROM cloud_sync_rekey_mutations WHERE account_id=$1",
        "DELETE FROM cloud_sync_reset_mutations WHERE account_id=$1",
        "DELETE FROM cloud_sync_push_mutations WHERE account_id=$1",
        "DELETE FROM cloud_data_protection_envelopes WHERE account_id=$1",
        "DELETE FROM cloud_data_protection_mutations WHERE account_id=$1",
        "DELETE FROM cloud_ai_provider_config_versions WHERE account_id=$1",
        "DELETE FROM cloud_ai_provider_configs WHERE account_id=$1",
        "DELETE FROM cloud_host_versions WHERE account_id=$1",
        "DELETE FROM cloud_hosts WHERE account_id=$1",
        "DELETE FROM sync_device_checkpoints WHERE account_id=$1",
        "DELETE FROM sync_mutations WHERE account_id=$1",
        "DELETE FROM sync_conflicts WHERE account_id=$1",
        "DELETE FROM sync_record_versions WHERE account_id=$1",
        "DELETE FROM sync_records WHERE account_id=$1",
        "DELETE FROM sync_states WHERE account_id=$1",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .execute(&mut **tx)
            .await
            .map_err(storage)?;
    }
    Ok(())
}

fn response(generation: i64, epoch: i64, revision: i64, idempotent: bool) -> ResetSyncResponse {
    ResetSyncResponse {
        status: "reset".to_owned(),
        sync_generation: generation,
        protection_epoch: epoch,
        protection_revision: revision,
        current_revision: 0,
        data_protection_configured: false,
        idempotent,
    }
}

fn next(value: i64, field: &str) -> AppResult<i64> {
    value
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict(format!("{field} cannot advance")))
}
