//! Atomic explicit push, account revision allocation, idempotency, and tombstones.

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    HostConflictView, HostMetadataInput, HostOperation, PushOutcome, PushRequest,
    actor::DeviceActor, validation::ValidatedChange,
};

use super::{
    DbTransaction, begin, commit,
    hosts::{HostRow, lock_current},
    lock_sync_state, require_active_device, storage,
};

#[derive(FromRow)]
struct MutationRow {
    source_device_id: Uuid,
    request_hash: Vec<u8>,
    outcome: String,
    result_revision: i64,
    changed_count: i32,
    conflict_id: Option<Uuid>,
}

#[derive(FromRow)]
struct ConflictRow {
    id: Uuid,
    host_id: Uuid,
    client_mutation_id: Uuid,
    base_revision: i64,
    remote_revision: i64,
    proposed_operation: String,
    source_device_id: Uuid,
    created_at: DateTime<Utc>,
}

pub(super) struct WriteValue {
    pub(super) address: String,
    pub(super) port: i32,
    pub(super) name: String,
    pub(super) platform: String,
    pub(super) tags: Value,
    pub(super) status: String,
    pub(super) ciphertext: Option<Vec<u8>>,
    pub(super) deleted: bool,
}

pub(crate) async fn push(
    pool: &PgPool,
    actor: DeviceActor,
    request: &PushRequest,
    changes: &[ValidatedChange],
    request_hash: &[u8; 32],
) -> AppResult<PushOutcome> {
    let account_id = actor.account_id();
    let device_id = actor.device_id();
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, account_id, device_id).await?;
    let state = lock_sync_state(&mut tx, account_id).await?;

    if let Some(outcome) = replay_mutation(
        &mut tx,
        account_id,
        device_id,
        request.client_mutation_id,
        request_hash,
    )
    .await?
    {
        commit(tx).await?;
        return Ok(outcome);
    }
    if request.base_revision < state.compacted_through_revision {
        return Err(AppError::SyncResyncRequired(
            "the requested host revision is no longer available".to_owned(),
        ));
    }
    if request.base_revision > state.current_revision {
        return Err(AppError::Conflict(
            "base_revision is newer than the account host revision".to_owned(),
        ));
    }

    let mut current_rows = Vec::with_capacity(changes.len());
    let mut first_conflict = None;
    for change in changes {
        let current = lock_current(&mut tx, account_id, change.host_id).await?;
        if first_conflict.is_none() && conflicts(change, current.as_ref()) {
            first_conflict = Some((change, current.as_ref().map_or(0, |row| row.revision)));
        }
        current_rows.push(current);
    }
    if let Some((change, remote_revision)) = first_conflict {
        let conflict = insert_conflict(
            &mut tx,
            actor,
            request,
            change,
            remote_revision,
            request_hash,
        )
        .await?;
        insert_mutation(
            &mut tx,
            actor,
            request.client_mutation_id,
            request_hash,
            "conflict",
            state.current_revision,
            0,
            Some(conflict.id),
        )
        .await?;
        commit(tx).await?;
        return Ok(PushOutcome::Conflict {
            conflict,
            idempotent: false,
        });
    }

    let mut revision = state.current_revision;
    let mut changed_count = 0_u32;
    for (change, current) in changes.iter().zip(current_rows.into_iter()) {
        let Some(value) = write_value(change, current.as_ref()) else {
            continue;
        };
        revision += 1;
        changed_count += 1;
        write_host(&mut tx, actor, change.host_id, revision, value).await?;
    }
    if changed_count > 0 {
        sqlx::query(
            "UPDATE cloud_host_sync_states
             SET current_revision = $2, updated_at = now()
             WHERE account_id = $1",
        )
        .bind(account_id)
        .bind(revision)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }
    let label = if changed_count == 0 {
        "unchanged"
    } else {
        "applied"
    };
    insert_mutation(
        &mut tx,
        actor,
        request.client_mutation_id,
        request_hash,
        label,
        revision,
        changed_count,
        None,
    )
    .await?;
    commit(tx).await?;
    if changed_count == 0 {
        Ok(PushOutcome::Unchanged {
            revision,
            idempotent: false,
        })
    } else {
        Ok(PushOutcome::Applied {
            revision,
            changed_count,
            idempotent: false,
        })
    }
}

fn conflicts(change: &ValidatedChange, current: Option<&HostRow>) -> bool {
    match change.operation {
        HostOperation::Insert => current.is_some(),
        HostOperation::Update | HostOperation::Delete => {
            current.map(|row| row.revision) != change.expected_revision
        }
    }
}

fn write_value(change: &ValidatedChange, current: Option<&HostRow>) -> Option<WriteValue> {
    match change.operation {
        HostOperation::Insert | HostOperation::Update => {
            let metadata = change.metadata.as_ref()?;
            let ciphertext = match &change.ciphertext {
                Some(value) => value.clone(),
                None if change.operation == HostOperation::Update => {
                    current.and_then(|row| row.ciphertext.clone())
                }
                None => None,
            };
            let value = from_metadata(metadata, ciphertext);
            if current.is_some_and(|row| same_value(row, &value)) {
                None
            } else {
                Some(value)
            }
        }
        HostOperation::Delete => {
            let current = current?;
            if current.is_deleted {
                None
            } else {
                Some(WriteValue {
                    address: current.address.clone(),
                    port: current.port,
                    name: current.name.clone(),
                    platform: current.platform.clone(),
                    tags: current.tags.clone(),
                    status: current.status.clone(),
                    ciphertext: None,
                    deleted: true,
                })
            }
        }
    }
}

pub(super) fn from_metadata(
    metadata: &HostMetadataInput,
    ciphertext: Option<Vec<u8>>,
) -> WriteValue {
    WriteValue {
        address: metadata.address.clone(),
        port: i32::from(metadata.port),
        name: metadata.name.clone(),
        platform: metadata.platform.clone(),
        tags: serde_json::json!(metadata.tags),
        status: metadata.status.as_str().to_owned(),
        ciphertext,
        deleted: false,
    }
}

pub(super) fn same_value(current: &HostRow, value: &WriteValue) -> bool {
    current.address == value.address
        && current.port == value.port
        && current.name == value.name
        && current.platform == value.platform
        && current.tags == value.tags
        && current.status == value.status
        && current.ciphertext == value.ciphertext
        && current.is_deleted == value.deleted
}

pub(super) async fn write_host(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    host_id: Uuid,
    revision: i64,
    value: WriteValue,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_hosts
             (account_id, id, address, port, name, platform, tags, status,
              ciphertext, source_device_id, revision, is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
         ON CONFLICT (account_id, id) DO UPDATE SET
             address = EXCLUDED.address, port = EXCLUDED.port,
             name = EXCLUDED.name, platform = EXCLUDED.platform,
             tags = EXCLUDED.tags,
             status = EXCLUDED.status, ciphertext = EXCLUDED.ciphertext,
             source_device_id = EXCLUDED.source_device_id,
             revision = EXCLUDED.revision, is_deleted = EXCLUDED.is_deleted,
             updated_at = now()",
    )
    .bind(actor.account_id())
    .bind(host_id)
    .bind(&value.address)
    .bind(value.port)
    .bind(&value.name)
    .bind(&value.platform)
    .bind(&value.tags)
    .bind(&value.status)
    .bind(&value.ciphertext)
    .bind(actor.device_id())
    .bind(revision)
    .bind(value.deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO cloud_host_versions
             (account_id, host_id, revision, address, port, name, platform,
              tags, status, ciphertext, source_device_id, is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
    )
    .bind(actor.account_id())
    .bind(host_id)
    .bind(revision)
    .bind(value.address)
    .bind(value.port)
    .bind(value.name)
    .bind(value.platform)
    .bind(value.tags)
    .bind(value.status)
    .bind(value.ciphertext)
    .bind(actor.device_id())
    .bind(value.deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn replay_mutation(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
    mutation_id: Uuid,
    request_hash: &[u8; 32],
) -> AppResult<Option<PushOutcome>> {
    let row = sqlx::query_as::<_, MutationRow>(
        "SELECT source_device_id, request_hash, outcome, result_revision,
                changed_count, conflict_id
         FROM cloud_host_mutations
         WHERE account_id = $1 AND client_mutation_id = $2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    if row.source_device_id != device_id || row.request_hash.as_slice() != request_hash {
        return Err(AppError::Conflict(
            "client_mutation_id was already used for another request".to_owned(),
        ));
    }
    let outcome = match row.outcome.as_str() {
        "applied" => PushOutcome::Applied {
            revision: row.result_revision,
            changed_count: u32::try_from(row.changed_count).unwrap_or(0),
            idempotent: true,
        },
        "unchanged" => PushOutcome::Unchanged {
            revision: row.result_revision,
            idempotent: true,
        },
        "conflict" => {
            let id = row.conflict_id.ok_or_else(super::invalid_stored_value)?;
            PushOutcome::Conflict {
                conflict: load_conflict(tx, account_id, id).await?,
                idempotent: true,
            }
        }
        _ => return Err(super::invalid_stored_value()),
    };
    Ok(Some(outcome))
}

async fn insert_conflict(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &PushRequest,
    change: &ValidatedChange,
    remote_revision: i64,
    request_hash: &[u8; 32],
) -> AppResult<HostConflictView> {
    let metadata = change.metadata.as_ref();
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO cloud_host_conflicts
             (id, account_id, host_id, client_mutation_id, source_device_id,
              base_revision, remote_revision, proposed_operation,
              proposed_address, proposed_port, proposed_name, proposed_platform,
              proposed_tags, proposed_status, proposed_ciphertext_is_set,
              proposed_ciphertext, proposed_expected_revision, request_hash)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
    )
    .bind(id)
    .bind(actor.account_id())
    .bind(change.host_id)
    .bind(request.client_mutation_id)
    .bind(actor.device_id())
    .bind(request.base_revision)
    .bind(remote_revision)
    .bind(change.operation.as_str())
    .bind(metadata.map(|value| value.address.as_str()))
    .bind(metadata.map(|value| i32::from(value.port)))
    .bind(metadata.map(|value| value.name.as_str()))
    .bind(metadata.map(|value| value.platform.as_str()))
    .bind(metadata.map(|value| serde_json::json!(value.tags)))
    .bind(metadata.map(|value| value.status.as_str()))
    .bind(change.ciphertext.is_some())
    .bind(change.ciphertext.as_ref().and_then(|value| value.as_ref()))
    .bind(change.expected_revision)
    .bind(request_hash.as_slice())
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    load_conflict(tx, actor.account_id(), id).await
}

async fn load_conflict(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    id: Uuid,
) -> AppResult<HostConflictView> {
    let row = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, host_id, client_mutation_id, base_revision, remote_revision,
                proposed_operation, source_device_id, created_at
         FROM cloud_host_conflicts
         WHERE account_id = $1 AND id = $2",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(HostConflictView {
        id: row.id,
        host_id: row.host_id,
        client_mutation_id: row.client_mutation_id,
        base_revision: row.base_revision,
        remote_revision: row.remote_revision,
        proposed_operation: HostOperation::parse(&row.proposed_operation)
            .ok_or_else(super::invalid_stored_value)?,
        source_device_id: row.source_device_id,
        created_at: row.created_at,
    })
}

#[allow(clippy::too_many_arguments)]
async fn insert_mutation(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    request_hash: &[u8; 32],
    outcome: &str,
    revision: i64,
    changed_count: u32,
    conflict_id: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_host_mutations
             (account_id, client_mutation_id, source_device_id, request_hash,
              outcome, result_revision, changed_count, conflict_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(actor.account_id())
    .bind(mutation_id)
    .bind(actor.device_id())
    .bind(request_hash.as_slice())
    .bind(outcome)
    .bind(revision)
    .bind(i32::try_from(changed_count).unwrap_or(i32::MAX))
    .bind(conflict_id)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}
