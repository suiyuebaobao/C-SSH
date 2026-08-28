//! Read-only 0.7.6 snapshot and all-or-nothing legacy ciphertext migration.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    HostStatus, LegacyPullCursor, LegacyPullRequest, LegacyPullResponse,
    MigrateDataProtectionRequest, PullAiProviderRecord, PullHostRecord, ResourceKind,
    ResourceRevision,
    actor::DeviceActor,
    validation::{ValidatedEnvelope, ValidatedRekeyResource},
};

use super::super::{
    DbTransaction, SyncState,
    ai::{self, AiWriteValue},
    begin,
    capacity::require_current_within_limit,
    commit, lock_sync_state,
    push::{WriteValue, write_host},
    require_active_device, require_base_revision, require_protection_version,
    require_sync_generation, storage,
};
use super::{
    DataProtectionOperation, PriorProtectionMutation, audit_mutation, clear_delivery_state,
    insert_envelope, load_prior_mutation, persist_mutation, purge_prior_ciphertext_versions,
    response,
};
use super::{core::update_state, projection::active_encrypted_count};

#[derive(Clone, FromRow)]
struct LegacyIdentity {
    resource_kind: String,
    resource_id: Uuid,
    revision: i64,
}

#[derive(FromRow)]
struct LegacyHostRow {
    id: Uuid,
    address: String,
    port: i32,
    name: String,
    platform: String,
    tags: Value,
    status: String,
    ciphertext: Vec<u8>,
    source_device_id: Uuid,
    revision: i64,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct LegacyAiRow {
    id: Uuid,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    envelope_metadata: Value,
    source_device_id: Uuid,
    revision: i64,
    updated_at: DateTime<Utc>,
}

enum LegacyResource {
    Host(LegacyHostRow),
    Ai(LegacyAiRow),
}

impl LegacyResource {
    const fn kind(&self) -> ResourceKind {
        match self {
            Self::Host(_) => ResourceKind::Host,
            Self::Ai(_) => ResourceKind::AiProviderAccount,
        }
    }

    const fn id(&self) -> Uuid {
        match self {
            Self::Host(row) => row.id,
            Self::Ai(row) => row.id,
        }
    }

    const fn revision(&self) -> i64 {
        match self {
            Self::Host(row) => row.revision,
            Self::Ai(row) => row.revision,
        }
    }
}

pub(crate) async fn pull(
    pool: &PgPool,
    actor: DeviceActor,
    request: LegacyPullRequest,
) -> AppResult<LegacyPullResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    require_legacy_state(&mut tx, actor.account_id(), state, &request).await?;
    let snapshot = request.snapshot_revision.unwrap_or(state.current_revision);
    if snapshot != state.current_revision {
        return Err(AppError::SyncStateChanged(
            "legacy snapshot revision changed".to_owned(),
        ));
    }
    let mut identities = load_identities(&mut tx, actor.account_id(), &request).await?;
    let has_more = identities.len() > request.limit as usize;
    if has_more {
        identities.pop();
    }
    let next_cursor = has_more.then(|| {
        let row = identities
            .last()
            .expect("non-empty page before continuation");
        LegacyPullCursor {
            revision: row.revision,
            resource_kind: ResourceKind::parse(&row.resource_kind)
                .expect("database resource kind constraint"),
            resource_id: row.resource_id,
        }
    });
    let mut host_records = Vec::new();
    let mut ai_records = Vec::new();
    for identity in identities {
        match ResourceKind::parse(&identity.resource_kind)
            .ok_or_else(super::super::invalid_stored_value)?
        {
            ResourceKind::Host => host_records
                .push(load_host(&mut tx, actor.account_id(), identity.resource_id).await?),
            ResourceKind::AiProviderAccount => {
                ai_records.push(load_ai(&mut tx, actor.account_id(), identity.resource_id).await?)
            }
        }
    }
    commit(tx).await?;
    Ok(LegacyPullResponse {
        sync_generation: state.sync_generation,
        protection_epoch: 0,
        protection_revision: 0,
        snapshot_revision: snapshot,
        host_records,
        ai_records,
        next_cursor,
        has_more,
    })
}

pub(crate) async fn migrate(
    pool: &PgPool,
    actor: DeviceActor,
    request: &MigrateDataProtectionRequest,
    envelope: &ValidatedEnvelope,
    candidates: &[ValidatedRekeyResource],
    request_hash: &[u8; 32],
) -> AppResult<crate::DataProtectionMutationResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    if let Some(prior) =
        load_prior_mutation(&mut tx, actor.account_id(), request.mutation_id).await?
    {
        validate_replay(&prior, actor, request, request_hash, state)?;
        let revisions = load_results(&mut tx, actor.account_id(), request.mutation_id).await?;
        commit(tx).await?;
        return Ok(response(
            "migrated",
            prior.result_generation,
            prior.result_epoch,
            prior.result_revision,
            prior.result_current_revision,
            revisions,
            true,
        ));
    }
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.expected_epoch, request.expected_revision)?;
    require_base_revision(state, request.current_revision)?;
    if state.protection_epoch != 0 || state.protection_revision != 0 {
        return Err(AppError::Conflict(
            "account is not in legacy protection state".to_owned(),
        ));
    }
    require_envelope_absent(&mut tx, actor.account_id()).await?;
    require_current_within_limit(&mut tx, actor.account_id()).await?;
    let current = lock_resources(&mut tx, actor.account_id()).await?;
    require_complete(&current, candidates)?;
    let result_generation = next(state.sync_generation, "sync_generation")?;
    let result_epoch = 1;
    let result_protection_revision = 1;
    let mut current_revision = state.current_revision;
    let mut results = Vec::with_capacity(current.len());
    for (stored, candidate) in current.into_iter().zip(candidates) {
        current_revision = next(current_revision, "current_revision")?;
        let previous_revision = stored.revision();
        let kind = stored.kind();
        let resource_id = stored.id();
        match (stored, candidate) {
            (LegacyResource::Host(host), ValidatedRekeyResource::Host { ciphertext, .. }) => {
                write_host(
                    &mut tx,
                    actor,
                    host.id,
                    current_revision,
                    WriteValue {
                        address: host.address,
                        port: host.port,
                        name: host.name,
                        platform: host.platform,
                        tags: host.tags,
                        status: host.status,
                        ciphertext: Some(ciphertext.clone()),
                        deleted: false,
                    },
                )
                .await?;
            }
            (LegacyResource::Ai(_), ValidatedRekeyResource::AiProviderAccount { payload, .. }) => {
                ai::write(
                    &mut tx,
                    actor,
                    resource_id,
                    current_revision,
                    AiWriteValue::from_payload(payload),
                )
                .await?;
            }
            _ => return Err(super::super::invalid_stored_value()),
        }
        results.push((
            ResourceRevision {
                resource_kind: kind,
                resource_id,
                cloud_revision: current_revision,
            },
            previous_revision,
        ));
    }
    purge_prior_ciphertext_versions(&mut tx, actor.account_id(), state.current_revision).await?;
    clear_delivery_state(&mut tx, actor.account_id()).await?;
    update_state(
        &mut tx,
        actor.account_id(),
        result_generation,
        result_epoch,
        result_protection_revision,
        current_revision,
    )
    .await?;
    insert_envelope(
        &mut tx,
        actor,
        result_generation,
        result_epoch,
        result_protection_revision,
        envelope,
    )
    .await?;
    persist_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Migrate,
        state,
        request_hash,
        result_generation,
        result_epoch,
        result_protection_revision,
        current_revision,
        results.len(),
    )
    .await?;
    persist_results(&mut tx, actor.account_id(), request.mutation_id, &results).await?;
    audit_mutation(
        &mut tx,
        actor,
        request.mutation_id,
        DataProtectionOperation::Migrate,
        "client_local_check",
        state,
        result_generation,
        result_epoch,
        result_protection_revision,
        current_revision,
        results.len(),
        0,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(
        "migrated",
        result_generation,
        result_epoch,
        result_protection_revision,
        current_revision,
        results.into_iter().map(|value| value.0).collect(),
        false,
    ))
}

async fn require_legacy_state(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    state: SyncState,
    request: &LegacyPullRequest,
) -> AppResult<()> {
    require_sync_generation(state, request.sync_generation)?;
    require_protection_version(state, request.expected_epoch, request.expected_revision)?;
    if state.protection_epoch != 0
        || state.protection_revision != 0
        || active_encrypted_count(tx, account_id).await? == 0
    {
        return Err(AppError::Conflict(
            "legacy migration snapshot is not available".to_owned(),
        ));
    }
    require_envelope_absent(tx, account_id).await
}

async fn load_identities(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    request: &LegacyPullRequest,
) -> AppResult<Vec<LegacyIdentity>> {
    let after_revision = request.after_revision.unwrap_or(0);
    let after_kind = request.after_resource_kind.map_or("", ResourceKind::as_str);
    let after_id = request.after_resource_id.unwrap_or(Uuid::nil());
    sqlx::query_as(
        "WITH active AS (
             SELECT 'host'::TEXT resource_kind, id resource_id, revision
             FROM cloud_hosts
             WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL
             UNION ALL
             SELECT 'ai_provider_account'::TEXT, id, revision
             FROM cloud_ai_provider_configs
             WHERE account_id=$1 AND NOT is_deleted AND ciphertext IS NOT NULL
         )
         SELECT resource_kind, resource_id, revision FROM active
         WHERE (revision, resource_kind, resource_id) > ($2, $3, $4)
         ORDER BY revision, resource_kind, resource_id
         LIMIT $5",
    )
    .bind(account_id)
    .bind(after_revision)
    .bind(after_kind)
    .bind(after_id)
    .bind(i64::from(request.limit) + 1)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)
}

async fn load_host(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    id: Uuid,
) -> AppResult<PullHostRecord> {
    let row = sqlx::query_as::<_, LegacyHostRow>(
        "SELECT id,address,port,name,platform,tags,status,ciphertext,
                source_device_id,revision,updated_at
         FROM cloud_hosts WHERE account_id=$1 AND id=$2
           AND NOT is_deleted AND ciphertext IS NOT NULL",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(PullHostRecord {
        host_id: row.id,
        revision: row.revision,
        address: row.address,
        port: u16::try_from(row.port).map_err(|_| super::super::invalid_stored_value())?,
        name: row.name,
        platform: row.platform,
        tags: serde_json::from_value(row.tags).map_err(|_| super::super::invalid_stored_value())?,
        status: HostStatus::parse(&row.status).ok_or_else(super::super::invalid_stored_value)?,
        ciphertext: Some(STANDARD.encode(row.ciphertext)),
        source_device_id: row.source_device_id,
        deleted: false,
        updated_at: row.updated_at,
    })
}

async fn load_ai(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    id: Uuid,
) -> AppResult<PullAiProviderRecord> {
    let row = sqlx::query_as::<_, LegacyAiRow>(
        "SELECT id,ciphertext,nonce,envelope_metadata,source_device_id,revision,updated_at
         FROM cloud_ai_provider_configs WHERE account_id=$1 AND id=$2
           AND NOT is_deleted AND ciphertext IS NOT NULL",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(PullAiProviderRecord {
        resource_id: row.id,
        revision: row.revision,
        ciphertext: Some(STANDARD.encode(row.ciphertext)),
        nonce: Some(STANDARD.encode(row.nonce)),
        envelope_metadata: Some(row.envelope_metadata),
        source_device_id: row.source_device_id,
        deleted: false,
        updated_at: row.updated_at,
    })
}

async fn lock_resources(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<Vec<LegacyResource>> {
    let hosts = sqlx::query_as::<_, LegacyHostRow>(
        "SELECT id,address,port,name,platform,tags,status,ciphertext,
                source_device_id,revision,updated_at
         FROM cloud_hosts WHERE account_id=$1 AND NOT is_deleted
           AND ciphertext IS NOT NULL ORDER BY id FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let ai = sqlx::query_as::<_, LegacyAiRow>(
        "SELECT id,ciphertext,nonce,envelope_metadata,source_device_id,revision,updated_at
         FROM cloud_ai_provider_configs WHERE account_id=$1 AND NOT is_deleted
           AND ciphertext IS NOT NULL ORDER BY id FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let mut resources = hosts
        .into_iter()
        .map(LegacyResource::Host)
        .chain(ai.into_iter().map(LegacyResource::Ai))
        .collect::<Vec<_>>();
    resources.sort_unstable_by_key(|value| (value.kind().as_str(), value.id()));
    Ok(resources)
}

fn require_complete(
    current: &[LegacyResource],
    candidates: &[ValidatedRekeyResource],
) -> AppResult<()> {
    if current.len() == candidates.len()
        && current.iter().zip(candidates).all(|(stored, candidate)| {
            stored.kind() == candidate.resource_kind()
                && stored.id() == candidate.resource_id()
                && stored.revision() == candidate.cloud_revision()
        })
    {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "legacy migration candidate is not the complete active ciphertext snapshot".to_owned(),
        ))
    }
}

fn validate_replay(
    prior: &PriorProtectionMutation,
    actor: DeviceActor,
    request: &MigrateDataProtectionRequest,
    request_hash: &[u8; 32],
    state: SyncState,
) -> AppResult<()> {
    if prior.operation != DataProtectionOperation::Migrate.as_str()
        || prior.source_device_id != actor.device_id()
        || prior.request_generation != request.sync_generation
        || prior.request_epoch != request.expected_epoch
        || prior.request_revision != request.expected_revision
        || prior.request_current_revision != request.current_revision
        || prior.request_hash.as_slice() != request_hash
    {
        return Err(AppError::Conflict(
            "mutation_id was already used by a different migration request".to_owned(),
        ));
    }
    if state.sync_generation == prior.result_generation
        && state.protection_epoch == prior.result_epoch
        && state.protection_revision == prior.result_revision
        && state.current_revision == prior.result_current_revision
    {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "migration replay belongs to an older protection state".to_owned(),
        ))
    }
}

include!("legacy/persistence.rs");
