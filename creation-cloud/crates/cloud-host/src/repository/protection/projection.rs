//! Authoritative configured/legacy projection and envelope read model.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    DataProtectionEnvelopeView, DataProtectionView, SyncGenerationTransition, SyncStateView,
    actor::DeviceActor,
};

use super::super::{
    DbTransaction, SyncState, begin, commit, lock_sync_state, require_active_device, storage,
};

#[derive(FromRow)]
struct EnvelopeRow {
    sync_generation: i64,
    protection_epoch: i64,
    protection_revision: i64,
    format_version: i16,
    kdf_algorithm: String,
    kdf_version: i32,
    kdf_memory_kib: i32,
    kdf_iterations: i32,
    kdf_parallelism: i32,
    kdf_output_length: i32,
    salt: Vec<u8>,
    nonce: Vec<u8>,
    wrapped_data_key: Vec<u8>,
    source_device_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub(crate) async fn get(pool: &PgPool, actor: DeviceActor) -> AppResult<DataProtectionView> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    let envelope = load_envelope(&mut tx, actor.account_id()).await?;
    let view = state_view(&mut tx, actor.account_id(), state, envelope.is_some()).await?;
    let envelope = envelope.map(envelope_view).transpose()?;
    commit(tx).await?;
    Ok(DataProtectionView {
        state: view,
        envelope,
    })
}

pub(super) async fn active_encrypted_count(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT
             (SELECT count(*)::BIGINT FROM cloud_hosts
              WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL)
           + (SELECT count(*)::BIGINT FROM cloud_ai_provider_configs
              WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL)",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)
}

async fn state_view(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    state: SyncState,
    configured: bool,
) -> AppResult<SyncStateView> {
    let secret_present = active_encrypted_count(tx, account_id).await? > 0;
    let legacy_migration_required = !configured
        && state.protection_epoch == 0
        && state.protection_revision == 0
        && secret_present;
    if !configured && secret_present && !legacy_migration_required {
        return Err(AppError::Storage(
            "encrypted sync state has no matching protection envelope".to_owned(),
        ));
    }
    let generation_transition = load_transition(tx, account_id, state.sync_generation).await?;
    Ok(SyncStateView {
        sync_generation: state.sync_generation,
        protection_epoch: state.protection_epoch,
        protection_revision: state.protection_revision,
        current_revision: state.current_revision,
        compacted_through_revision: state.compacted_through_revision,
        generation_transition,
        data_protection_configured: configured,
        legacy_migration_required,
        secret_present,
    })
}

async fn load_transition(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    generation: i64,
) -> AppResult<SyncGenerationTransition> {
    if let Some(operation) = sqlx::query_scalar::<_, String>(
        "SELECT operation FROM cloud_data_protection_mutations
         WHERE account_id=$1 AND result_generation=$2
           AND operation IN ('setup','migrate','reset')",
    )
    .bind(account_id)
    .bind(generation)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?
    {
        return match operation.as_str() {
            "setup" => Ok(SyncGenerationTransition::ProtectionSetup),
            "migrate" => Ok(SyncGenerationTransition::LegacyMigration),
            "reset" => Ok(SyncGenerationTransition::Reset),
            _ => Err(super::super::invalid_stored_value()),
        };
    }
    let (reset_seen, rekey_seen) = sqlx::query_as::<_, (bool, bool)>(
        "SELECT EXISTS(SELECT 1 FROM cloud_sync_reset_mutations
                       WHERE account_id=$1 AND result_generation=$2),
                EXISTS(SELECT 1 FROM cloud_sync_rekey_mutations
                       WHERE account_id=$1 AND result_generation=$2)",
    )
    .bind(account_id)
    .bind(generation)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    match (generation, reset_seen, rekey_seen) {
        (1, false, false) => Ok(SyncGenerationTransition::Initial),
        (2.., true, false) => Ok(SyncGenerationTransition::Reset),
        (2.., false, true) => Ok(SyncGenerationTransition::Rekey),
        (2.., false, false) => load_audit_transition(tx, account_id, generation).await,
        _ => transition_unproven(),
    }
}

async fn load_audit_transition(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    generation: i64,
) -> AppResult<SyncGenerationTransition> {
    let generation = generation.to_string();
    let (setup, migrate, reset, rekey) = sqlx::query_as::<_, (bool, bool, bool, bool)>(
        "SELECT
           EXISTS(SELECT 1 FROM audit_events
                  WHERE actor_account_id=$1 AND outcome='success'
                    AND action='sync.data_protection_mutation_v1'
                    AND details->>'operation'='setup'
                    AND details->>'sync_generation'=$2),
           EXISTS(SELECT 1 FROM audit_events
                  WHERE actor_account_id=$1 AND outcome='success'
                    AND action='sync.data_protection_mutation_v1'
                    AND details->>'operation'='migrate'
                    AND details->>'sync_generation'=$2),
           EXISTS(SELECT 1 FROM audit_events
                  WHERE actor_account_id=$1 AND outcome='success'
                    AND ((action='sync.data_protection_mutation_v1'
                          AND details->>'operation'='reset')
                         OR action IN ('sync.encrypted_data_reset',
                                       'sync.encrypted_data_reset_v2'))
                    AND details->>'sync_generation'=$2),
           EXISTS(SELECT 1 FROM audit_events
                  WHERE actor_account_id=$1 AND outcome='success'
                    AND action IN ('sync.encrypted_data_rekey',
                                   'sync.encrypted_data_rekey_v2')
                    AND details->>'sync_generation'=$2)",
    )
    .bind(account_id)
    .bind(generation)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    match (setup, migrate, reset, rekey) {
        (true, false, false, false) => Ok(SyncGenerationTransition::ProtectionSetup),
        (false, true, false, false) => Ok(SyncGenerationTransition::LegacyMigration),
        (false, false, true, false) => Ok(SyncGenerationTransition::Reset),
        (false, false, false, true) => Ok(SyncGenerationTransition::Rekey),
        _ => transition_unproven(),
    }
}

fn transition_unproven<T>() -> AppResult<T> {
    Err(AppError::Storage(
        "current sync generation transition cannot be proven".to_owned(),
    ))
}

async fn load_envelope(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<Option<EnvelopeRow>> {
    sqlx::query_as(
        "SELECT sync_generation,protection_epoch,protection_revision,
                format_version,kdf_algorithm,kdf_version,kdf_memory_kib,
                kdf_iterations,kdf_parallelism,kdf_output_length,
                salt,nonce,wrapped_data_key,source_device_id,created_at,updated_at
         FROM cloud_data_protection_envelopes WHERE account_id=$1",
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

fn envelope_view(row: EnvelopeRow) -> AppResult<DataProtectionEnvelopeView> {
    let invalid = |_| super::super::invalid_stored_value();
    Ok(DataProtectionEnvelopeView {
        sync_generation: row.sync_generation,
        protection_epoch: row.protection_epoch,
        protection_revision: row.protection_revision,
        format_version: u16::try_from(row.format_version).map_err(invalid)?,
        kdf_algorithm: row.kdf_algorithm,
        kdf_version: u32::try_from(row.kdf_version).map_err(invalid)?,
        kdf_memory_kib: u32::try_from(row.kdf_memory_kib).map_err(invalid)?,
        kdf_iterations: u32::try_from(row.kdf_iterations).map_err(invalid)?,
        kdf_parallelism: u32::try_from(row.kdf_parallelism).map_err(invalid)?,
        kdf_output_length: u32::try_from(row.kdf_output_length).map_err(invalid)?,
        salt: STANDARD.encode(row.salt),
        nonce: STANDARD.encode(row.nonce),
        wrapped_data_key: STANDARD.encode(row.wrapped_data_key),
        source_device_id: row.source_device_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
