//! Envelope projection plus setup and wrapper-only password change transactions.

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    ChangeDataProtectionRequest, DataProtectionMutationResponse, ResourceRevision,
    SetupDataProtectionRequest, actor::DeviceActor, validation::ValidatedEnvelope,
};

use super::projection::active_encrypted_count;

use super::super::{
    DbTransaction, SyncState, begin, commit, lock_sync_state, require_active_device,
    require_base_revision, require_configured_envelope, require_protection_version,
    require_sync_generation, storage,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::repository) enum DataProtectionOperation {
    Setup,
    Migrate,
    Change,
    Reset,
}

impl DataProtectionOperation {
    pub(in crate::repository) const fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Migrate => "migrate",
            Self::Change => "change",
            Self::Reset => "reset",
        }
    }
}

#[derive(FromRow)]
pub(in crate::repository) struct PriorProtectionMutation {
    pub(in crate::repository) operation: String,
    pub(in crate::repository) source_device_id: Uuid,
    pub(in crate::repository) request_generation: i64,
    pub(in crate::repository) request_epoch: i64,
    pub(in crate::repository) request_revision: i64,
    pub(in crate::repository) request_current_revision: i64,
    pub(in crate::repository) request_hash: Vec<u8>,
    pub(in crate::repository) result_generation: i64,
    pub(in crate::repository) result_epoch: i64,
    pub(in crate::repository) result_revision: i64,
    pub(in crate::repository) result_current_revision: i64,
}

struct ReplayRequest<'a> {
    actor: DeviceActor,
    operation: DataProtectionOperation,
    generation: i64,
    epoch: i64,
    revision: i64,
    current_revision: i64,
    request_hash: &'a [u8; 32],
}

pub(crate) async fn setup(
    pool: &PgPool,
    actor: DeviceActor,
    request: &SetupDataProtectionRequest,
    envelope: &ValidatedEnvelope,
    request_hash: &[u8; 32],
) -> AppResult<DataProtectionMutationResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    if let Some(prior) =
        load_prior_mutation(&mut tx, actor.account_id(), request.mutation_id).await?
    {
        let result = replay(
            &prior,
            ReplayRequest {
                actor,
                operation: DataProtectionOperation::Setup,
                generation: request.sync_generation,
                epoch: request.expected_epoch,
                revision: request.expected_revision,
                current_revision: request.current_revision,
                request_hash,
            },
            state,
        )?;
        commit(tx).await?;
        return Ok(result);
    }
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.expected_epoch, request.expected_revision)?;
    require_base_revision(state, request.current_revision)?;
    require_envelope_absent(&mut tx, actor.account_id()).await?;
    let encrypted = active_encrypted_count(&mut tx, actor.account_id()).await?;
    if encrypted != 0 {
        return Err(AppError::Conflict(
            "legacy encrypted resources require the migration endpoint".to_owned(),
        ));
    }
    let result_generation = checked_next(state.sync_generation, "sync_generation")?;
    let result_epoch = checked_next(state.protection_epoch, "protection_epoch")?;
    let result_revision = checked_next(state.protection_revision, "protection_revision")?;
    clear_delivery_state(&mut tx, actor.account_id()).await?;
    purge_prior_ciphertext_versions(&mut tx, actor.account_id(), state.current_revision).await?;
    update_state(
        &mut tx,
        actor.account_id(),
        result_generation,
        result_epoch,
        result_revision,
        state.current_revision,
    )
    .await?;
    insert_envelope(
        &mut tx,
        actor,
        result_generation,
        result_epoch,
        result_revision,
        envelope,
    )
    .await?;
    persist_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Setup,
        state,
        request_hash,
        result_generation,
        result_epoch,
        result_revision,
        state.current_revision,
        0,
    )
    .await?;
    audit_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Setup,
        "not_applicable",
        state,
        result_generation,
        result_epoch,
        result_revision,
        state.current_revision,
        0,
        0,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(
        "configured",
        result_generation,
        result_epoch,
        result_revision,
        state.current_revision,
        Vec::new(),
        false,
    ))
}

pub(crate) async fn change(
    pool: &PgPool,
    actor: DeviceActor,
    request: &ChangeDataProtectionRequest,
    envelope: &ValidatedEnvelope,
    request_hash: &[u8; 32],
) -> AppResult<DataProtectionMutationResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    if let Some(prior) =
        load_prior_mutation(&mut tx, actor.account_id(), request.mutation_id).await?
    {
        let result = replay(
            &prior,
            ReplayRequest {
                actor,
                operation: DataProtectionOperation::Change,
                generation: request.sync_generation,
                epoch: request.expected_epoch,
                revision: request.expected_revision,
                current_revision: request.current_revision,
                request_hash,
            },
            state,
        )?;
        commit(tx).await?;
        return Ok(result);
    }
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.expected_epoch, request.expected_revision)?;
    require_base_revision(state, request.current_revision)?;
    require_configured_envelope(&mut tx, actor.account_id(), state).await?;
    let result_revision = checked_next(state.protection_revision, "protection_revision")?;
    update_state(
        &mut tx,
        actor.account_id(),
        state.sync_generation,
        state.protection_epoch,
        result_revision,
        state.current_revision,
    )
    .await?;
    replace_envelope(
        &mut tx,
        actor,
        state.sync_generation,
        state.protection_epoch,
        result_revision,
        envelope,
    )
    .await?;
    persist_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Change,
        state,
        request_hash,
        state.sync_generation,
        state.protection_epoch,
        result_revision,
        state.current_revision,
        0,
    )
    .await?;
    audit_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Change,
        "client_local_check",
        state,
        state.sync_generation,
        state.protection_epoch,
        result_revision,
        state.current_revision,
        0,
        0,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(
        "changed",
        state.sync_generation,
        state.protection_epoch,
        result_revision,
        state.current_revision,
        Vec::new(),
        false,
    ))
}

pub(in crate::repository) async fn load_prior_mutation(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Option<PriorProtectionMutation>> {
    sqlx::query_as(
        "SELECT operation, source_device_id, request_generation, request_epoch,
                request_revision, request_current_revision, request_hash,
                result_generation, result_epoch, result_revision,
                result_current_revision
         FROM cloud_data_protection_mutations
         WHERE account_id = $1 AND mutation_id = $2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::repository) async fn persist_mutation(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    operation: DataProtectionOperation,
    state: SyncState,
    request_hash: &[u8; 32],
    result_generation: i64,
    result_epoch: i64,
    result_revision: i64,
    result_current_revision: i64,
    changed_count: usize,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_data_protection_mutations
             (account_id, mutation_id, operation, source_device_id,
              request_generation, request_epoch, request_revision,
              request_current_revision, request_hash, result_generation,
              result_epoch, result_revision, result_current_revision, changed_count)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
    )
    .bind(actor.account_id())
    .bind(mutation_id)
    .bind(operation.as_str())
    .bind(actor.device_id())
    .bind(state.sync_generation)
    .bind(state.protection_epoch)
    .bind(state.protection_revision)
    .bind(state.current_revision)
    .bind(request_hash.as_slice())
    .bind(result_generation)
    .bind(result_epoch)
    .bind(result_revision)
    .bind(result_current_revision)
    .bind(i32::try_from(changed_count).unwrap_or(i32::MAX))
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(in crate::repository) async fn audit_mutation(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    operation: DataProtectionOperation,
    authorization_mode: &str,
    state: SyncState,
    result_generation: i64,
    result_epoch: i64,
    result_revision: i64,
    result_current_revision: i64,
    changed_count: usize,
    removed_resource_count: i64,
) -> AppResult<()> {
    let details = json!({
        "operation": operation.as_str(),
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "authorization_mode": authorization_mode,
        "previous_sync_generation": state.sync_generation,
        "sync_generation": result_generation,
        "previous_protection_epoch": state.protection_epoch,
        "protection_epoch": result_epoch,
        "previous_protection_revision": state.protection_revision,
        "protection_revision": result_revision,
        "previous_current_revision": state.current_revision,
        "current_revision": result_current_revision,
        "changed_count": i64::try_from(changed_count).unwrap_or(i64::MAX),
        "removed_resource_count": removed_resource_count,
    });
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events
             (id, actor_account_id, action, resource_kind, resource_id,
              outcome, request_id, details)
         VALUES ($1,$2,'sync.data_protection_mutation_v1','sync_account',$3,
                 'success',$4,$5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(actor.account_id().to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| AppError::Storage("failed to persist data protection audit".to_owned()))?;
    Ok(())
}

pub(in crate::repository) async fn insert_envelope(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    generation: i64,
    epoch: i64,
    revision: i64,
    envelope: &ValidatedEnvelope,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_data_protection_envelopes
             (account_id, sync_generation, protection_epoch, protection_revision,
              format_version, kdf_algorithm, kdf_version, kdf_memory_kib,
              kdf_iterations, kdf_parallelism, kdf_output_length,
              salt, nonce, wrapped_data_key, source_device_id)
         VALUES ($1,$2,$3,$4,1,'argon2id',19,19456,2,1,32,$5,$6,$7,$8)",
    )
    .bind(actor.account_id())
    .bind(generation)
    .bind(epoch)
    .bind(revision)
    .bind(&envelope.salt)
    .bind(&envelope.nonce)
    .bind(&envelope.wrapped_data_key)
    .bind(actor.device_id())
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

pub(in crate::repository) async fn clear_delivery_state(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<()> {
    for statement in [
        "DELETE FROM cloud_sync_pull_decisions WHERE account_id = $1",
        "DELETE FROM cloud_sync_resource_deliveries WHERE account_id = $1",
        "DELETE FROM cloud_sync_pull_watermarks WHERE account_id = $1",
        "DELETE FROM cloud_sync_device_checkpoints WHERE account_id = $1",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .execute(&mut **tx)
            .await
            .map_err(storage)?;
    }
    Ok(())
}

pub(in crate::repository) async fn purge_prior_ciphertext_versions(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    previous_revision: i64,
) -> AppResult<()> {
    for statement in [
        "DELETE FROM cloud_host_versions
         WHERE account_id = $1 AND revision <= $2 AND ciphertext IS NOT NULL",
        "DELETE FROM cloud_ai_provider_config_versions
         WHERE account_id = $1 AND revision <= $2 AND ciphertext IS NOT NULL",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .bind(previous_revision)
            .execute(&mut **tx)
            .await
            .map_err(storage)?;
    }
    Ok(())
}

pub(in crate::repository) fn response(
    status: &str,
    generation: i64,
    epoch: i64,
    revision: i64,
    current_revision: i64,
    revisions: Vec<ResourceRevision>,
    idempotent: bool,
) -> DataProtectionMutationResponse {
    DataProtectionMutationResponse {
        status: status.to_owned(),
        sync_generation: generation,
        protection_epoch: epoch,
        protection_revision: revision,
        current_revision,
        data_protection_configured: true,
        revisions,
        idempotent,
    }
}

fn replay(
    prior: &PriorProtectionMutation,
    request: ReplayRequest<'_>,
    state: SyncState,
) -> AppResult<DataProtectionMutationResponse> {
    let ReplayRequest {
        actor,
        operation,
        generation,
        epoch,
        revision,
        current_revision,
        request_hash,
    } = request;
    if prior.operation != operation.as_str()
        || prior.source_device_id != actor.device_id()
        || prior.request_generation != generation
        || prior.request_epoch != epoch
        || prior.request_revision != revision
        || prior.request_current_revision != current_revision
        || prior.request_hash.as_slice() != request_hash
    {
        return Err(AppError::Conflict(
            "mutation_id was already used by a different protection request".to_owned(),
        ));
    }
    if state.sync_generation != prior.result_generation
        || state.protection_epoch != prior.result_epoch
        || state.protection_revision != prior.result_revision
        || state.current_revision != prior.result_current_revision
    {
        return Err(AppError::sync_generation_changed(
            "protection mutation replay belongs to an older state",
        ));
    }
    Ok(response(
        operation.as_str(),
        prior.result_generation,
        prior.result_epoch,
        prior.result_revision,
        prior.result_current_revision,
        Vec::new(),
        true,
    ))
}

include!("core/storage.rs");
