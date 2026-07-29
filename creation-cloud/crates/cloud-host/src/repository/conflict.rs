//! Explicit persisted conflict resolution. Server-side merge is intentionally absent.

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, Page, PageQuery};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    HostMetadataInput, HostOperation, HostStatus, RemoteResolution, ResolveConflictOutcome,
    ResolveConflictRequest, actor::DeviceActor,
};

use super::{
    DbTransaction, begin, commit,
    hosts::{HostRow, lock_current},
    lock_sync_state,
    push::{WriteValue, from_metadata, same_value, write_host},
    require_active_device, storage,
};

#[derive(FromRow)]
struct ConflictResolutionRow {
    id: Uuid,
    host_id: Uuid,
    proposed_operation: String,
    proposed_address: Option<String>,
    proposed_port: Option<i32>,
    proposed_name: Option<String>,
    proposed_platform: Option<String>,
    proposed_tags: Option<Value>,
    proposed_status: Option<String>,
    proposed_ciphertext_is_set: bool,
    proposed_ciphertext: Option<Vec<u8>>,
    resolution_action: Option<String>,
    resolution_mutation_id: Option<Uuid>,
    resolution_hash: Option<Vec<u8>>,
    resolved_device_id: Option<Uuid>,
    resolved_revision: Option<i64>,
    resolved_at: Option<DateTime<Utc>>,
}

pub(crate) async fn resolve(
    pool: &PgPool,
    actor: DeviceActor,
    conflict_id: Uuid,
    request: &ResolveConflictRequest,
    request_hash: &[u8; 32],
) -> AppResult<ResolveConflictOutcome> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    reject_reused_resolution_id(&mut tx, actor, conflict_id, request).await?;
    let row = load_locked(&mut tx, actor.account_id(), conflict_id).await?;

    if row.resolved_at.is_some() {
        let outcome = replay_resolution(row, actor, request, request_hash)?;
        commit(tx).await?;
        return Ok(outcome);
    }

    let current = lock_current(&mut tx, actor.account_id(), row.host_id).await?;
    let remote_revision = current.as_ref().map_or(0, |host| host.revision);
    if remote_revision != request.expected_revision {
        return Err(AppError::Conflict(
            "the remote host revision changed before conflict resolution".to_owned(),
        ));
    }

    let mut result_revision = state.current_revision;
    if request.action == RemoteResolution::ReplaceRemote
        && let Some(value) = replacement_value(&row, current.as_ref())?
    {
        result_revision += 1;
        write_host(&mut tx, actor, row.host_id, result_revision, value).await?;
        sqlx::query(
            "UPDATE cloud_host_sync_states
             SET current_revision = $2, updated_at = now()
             WHERE account_id = $1",
        )
        .bind(actor.account_id())
        .bind(result_revision)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }

    let resolved_at = Utc::now();
    sqlx::query(
        "UPDATE cloud_host_conflicts
         SET resolution_action = $3, resolution_mutation_id = $4,
             resolution_hash = $5, resolved_device_id = $6,
             resolved_revision = $7, resolved_at = $8
         WHERE account_id = $1 AND id = $2 AND resolved_at IS NULL",
    )
    .bind(actor.account_id())
    .bind(conflict_id)
    .bind(request.action.as_str())
    .bind(request.resolution_mutation_id)
    .bind(request_hash.as_slice())
    .bind(actor.device_id())
    .bind(result_revision)
    .bind(resolved_at)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    commit(tx).await?;
    Ok(ResolveConflictOutcome {
        conflict_id,
        resolution_mutation_id: request.resolution_mutation_id,
        action: request.action,
        revision: result_revision,
        idempotent: false,
        resolved_at,
    })
}

pub(crate) async fn list_open(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<crate::HostConflictView>> {
    let page = page.normalized();
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::BIGINT
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND resolved_at IS NULL",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(storage)?;
    let rows = sqlx::query_as::<_, ConflictListRow>(
        "SELECT id, host_id, client_mutation_id, base_revision, remote_revision,
                proposed_operation, source_device_id, created_at
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND resolved_at IS NULL
         ORDER BY created_at, id
         LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(storage)?;
    let items = rows
        .into_iter()
        .map(ConflictListRow::view)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn get(
    pool: &PgPool,
    account_id: Uuid,
    conflict_id: Uuid,
) -> AppResult<crate::HostConflictView> {
    let row = sqlx::query_as::<_, ConflictListRow>(
        "SELECT id, host_id, client_mutation_id, base_revision, remote_revision,
                proposed_operation, source_device_id, created_at
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND id = $2",
    )
    .bind(account_id)
    .bind(conflict_id)
    .fetch_optional(pool)
    .await
    .map_err(storage)?
    .ok_or_else(|| AppError::NotFound("host conflict was not found".to_owned()))?;
    row.view()
}

#[derive(FromRow)]
struct ConflictListRow {
    id: Uuid,
    host_id: Uuid,
    client_mutation_id: Uuid,
    base_revision: i64,
    remote_revision: i64,
    proposed_operation: String,
    source_device_id: Uuid,
    created_at: DateTime<Utc>,
}

impl ConflictListRow {
    fn view(self) -> AppResult<crate::HostConflictView> {
        Ok(crate::HostConflictView {
            id: self.id,
            host_id: self.host_id,
            client_mutation_id: self.client_mutation_id,
            base_revision: self.base_revision,
            remote_revision: self.remote_revision,
            proposed_operation: HostOperation::parse(&self.proposed_operation)
                .ok_or_else(super::invalid_stored_value)?,
            source_device_id: self.source_device_id,
            created_at: self.created_at,
        })
    }
}

async fn load_locked(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    conflict_id: Uuid,
) -> AppResult<ConflictResolutionRow> {
    sqlx::query_as::<_, ConflictResolutionRow>(
        "SELECT id, host_id, proposed_operation, proposed_address,
                proposed_port, proposed_name, proposed_tags, proposed_status,
                proposed_platform, proposed_ciphertext_is_set,
                proposed_ciphertext, resolution_action,
                resolution_mutation_id, resolution_hash, resolved_device_id,
                resolved_revision, resolved_at
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND id = $2
         FOR UPDATE",
    )
    .bind(account_id)
    .bind(conflict_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?
    .ok_or_else(|| AppError::NotFound("host conflict was not found".to_owned()))
}

async fn reject_reused_resolution_id(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    conflict_id: Uuid,
    request: &ResolveConflictRequest,
) -> AppResult<()> {
    let prior = sqlx::query_scalar::<_, Uuid>(
        "SELECT id
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND resolution_mutation_id = $2",
    )
    .bind(actor.account_id())
    .bind(request.resolution_mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?;
    if prior.is_some_and(|id| id != conflict_id) {
        Err(AppError::Conflict(
            "resolution_mutation_id was already used for another conflict".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn replay_resolution(
    row: ConflictResolutionRow,
    actor: DeviceActor,
    request: &ResolveConflictRequest,
    request_hash: &[u8; 32],
) -> AppResult<ResolveConflictOutcome> {
    let matches = row.resolution_mutation_id == Some(request.resolution_mutation_id)
        && row.resolved_device_id == Some(actor.device_id())
        && row.resolution_hash.as_deref() == Some(request_hash.as_slice())
        && row.resolution_action.as_deref() == Some(request.action.as_str());
    if !matches {
        return Err(AppError::Conflict(
            "the host conflict was already resolved".to_owned(),
        ));
    }
    Ok(ResolveConflictOutcome {
        conflict_id: row.id,
        resolution_mutation_id: request.resolution_mutation_id,
        action: request.action,
        revision: row
            .resolved_revision
            .ok_or_else(super::invalid_stored_value)?,
        idempotent: true,
        resolved_at: row.resolved_at.ok_or_else(super::invalid_stored_value)?,
    })
}

fn replacement_value(
    row: &ConflictResolutionRow,
    current: Option<&HostRow>,
) -> AppResult<Option<WriteValue>> {
    let operation =
        HostOperation::parse(&row.proposed_operation).ok_or_else(super::invalid_stored_value)?;
    match operation {
        HostOperation::Delete => {
            let Some(current) = current else {
                return Ok(None);
            };
            if current.is_deleted {
                return Ok(None);
            }
            Ok(Some(WriteValue {
                address: current.address.clone(),
                port: current.port,
                name: current.name.clone(),
                platform: current.platform.clone(),
                tags: current.tags.clone(),
                status: current.status.clone(),
                ciphertext: None,
                deleted: true,
            }))
        }
        HostOperation::Insert | HostOperation::Update => {
            let metadata = proposed_metadata(row)?;
            let ciphertext = if row.proposed_ciphertext_is_set {
                row.proposed_ciphertext.clone()
            } else if operation == HostOperation::Update {
                current.and_then(|host| host.ciphertext.clone())
            } else {
                None
            };
            let value = from_metadata(&metadata, ciphertext);
            if current.is_some_and(|host| same_value(host, &value)) {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
    }
}

fn proposed_metadata(row: &ConflictResolutionRow) -> AppResult<HostMetadataInput> {
    let address = row
        .proposed_address
        .clone()
        .ok_or_else(super::invalid_stored_value)?;
    let port = row
        .proposed_port
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(super::invalid_stored_value)?;
    let name = row
        .proposed_name
        .clone()
        .ok_or_else(super::invalid_stored_value)?;
    let platform = row
        .proposed_platform
        .clone()
        .ok_or_else(super::invalid_stored_value)?;
    let tags = serde_json::from_value::<Vec<String>>(
        row.proposed_tags
            .clone()
            .ok_or_else(super::invalid_stored_value)?,
    )
    .map_err(|_| super::invalid_stored_value())?;
    let status = row
        .proposed_status
        .as_deref()
        .and_then(HostStatus::parse)
        .ok_or_else(super::invalid_stored_value)?;
    Ok(HostMetadataInput {
        address,
        port,
        name,
        platform,
        tags,
        status,
    })
}
