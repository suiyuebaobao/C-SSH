//! 先完整预检 Host 与 AI 资源 CAS，再在一个事务中整批写入统一 revision 流。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    AiProviderOperation, HostMetadataInput, HostOperation, PushOutcome, PushRequest, ResourceKind,
    ResourceRevision,
    actor::DeviceActor,
    validation::{ValidatedAiChange, ValidatedChange, ValidatedPush},
};

use super::{
    DbTransaction,
    ai::{self, AiRow, AiWriteValue},
    begin,
    capacity::enforce_encrypted_resource_limit,
    commit,
    hosts::{HostRow, lock_current},
    lock_sync_state,
    pull::{safe_checkpoint_revision, save_checkpoint},
    require_active_device, require_base_revision, require_sync_generation, storage,
};

#[derive(FromRow)]
struct MutationRow {
    source_device_id: Uuid,
    request_generation: i64,
    request_hash: Vec<u8>,
    outcome: String,
    result_revision: i64,
    changed_count: i32,
}

#[derive(FromRow)]
struct RevisionRow {
    resource_kind: String,
    resource_id: Uuid,
    result_revision: i64,
}

struct MutationResult<'a> {
    outcome: &'a str,
    revision: i64,
    changed_count: usize,
    revisions: &'a [ResourceRevision],
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
    changes: &ValidatedPush,
    request_hash: &[u8; 32],
) -> AppResult<PushOutcome> {
    let account_id = actor.account_id();
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, account_id, actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, account_id).await?;
    require_sync_generation(state, request.sync_generation)?;

    if let Some(outcome) = replay_mutation(
        &mut tx,
        actor,
        request.sync_generation,
        request.client_mutation_id,
        request_hash,
    )
    .await?
    {
        commit(tx).await?;
        return Ok(outcome);
    }
    require_base_revision(state, request.base_revision)?;

    // 所有行锁与 expected revision 检查必须在任何业务写入之前完成。
    let host_rows = precheck_hosts(&mut tx, account_id, &changes.host_changes).await?;
    let ai_rows = precheck_ai(&mut tx, account_id, &changes.ai_changes).await?;
    enforce_encrypted_resource_limit(
        &mut tx,
        account_id,
        &changes.host_changes,
        &host_rows,
        &changes.ai_changes,
        &ai_rows,
    )
    .await?;

    let mut revision = state.current_revision;
    let mut changed_count = 0_usize;
    let mut revisions = Vec::with_capacity(changes.host_changes.len() + changes.ai_changes.len());
    for (change, current) in changes.host_changes.iter().zip(host_rows.iter()) {
        let result_revision = if let Some(value) = host_write_value(change, current.as_ref()) {
            revision = next_revision(revision)?;
            write_host(&mut tx, actor, change.host_id, revision, value).await?;
            changed_count += 1;
            revision
        } else {
            current
                .as_ref()
                .map(|row| row.revision)
                .ok_or_else(super::invalid_stored_value)?
        };
        revisions.push(ResourceRevision {
            resource_kind: ResourceKind::Host,
            resource_id: change.host_id,
            cloud_revision: result_revision,
        });
    }
    for (change, current) in changes.ai_changes.iter().zip(ai_rows.iter()) {
        let result_revision = if let Some(value) = ai_write_value(change, current.as_ref()) {
            revision = next_revision(revision)?;
            ai::write(&mut tx, actor, change.resource_id, revision, value).await?;
            changed_count += 1;
            revision
        } else {
            current
                .as_ref()
                .map(|row| row.revision)
                .ok_or_else(super::invalid_stored_value)?
        };
        revisions.push(ResourceRevision {
            resource_kind: ResourceKind::AiProviderAccount,
            resource_id: change.resource_id,
            cloud_revision: result_revision,
        });
    }
    revisions.sort_unstable_by_key(|result| result.cloud_revision);

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
    let outcome = if changed_count == 0 {
        "unchanged"
    } else {
        "applied"
    };
    settle_source_device_resources(&mut tx, actor, &revisions).await?;
    let safe_revision = safe_checkpoint_revision(&mut tx, actor, revision).await?;
    save_checkpoint(&mut tx, actor, safe_revision).await?;
    let mutation = MutationResult {
        outcome,
        revision,
        changed_count,
        revisions: &revisions,
    };
    insert_mutation(&mut tx, actor, request, request_hash, &mutation).await?;
    commit(tx).await?;
    Ok(response(
        request.sync_generation,
        revision,
        changed_count,
        revisions,
        false,
    ))
}

pub(super) async fn settle_source_device_resources(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    revisions: &[ResourceRevision],
) -> AppResult<()> {
    for revision in revisions {
        sqlx::query(
            "INSERT INTO cloud_sync_pull_decisions
                 (account_id, device_id, resource_kind, resource_id, revision, action)
             VALUES ($1,$2,$3,$4,$5,'keep_local')
             ON CONFLICT (account_id, device_id, resource_kind, resource_id, revision)
             DO NOTHING",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(revision.resource_kind.as_str())
        .bind(revision.resource_id)
        .bind(revision.cloud_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn precheck_hosts(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    changes: &[ValidatedChange],
) -> AppResult<Vec<Option<HostRow>>> {
    let mut rows = Vec::with_capacity(changes.len());
    for change in changes {
        let current = lock_current(tx, account_id, change.host_id).await?;
        let matches = match change.operation {
            HostOperation::Insert => current.is_none(),
            HostOperation::Update | HostOperation::Delete => {
                current.as_ref().map(|row| row.revision) == change.expected_revision
            }
        };
        if !matches {
            return Err(AppError::SyncStateChanged(
                "host expected_revision no longer matches cloud state".to_owned(),
            ));
        }
        rows.push(current);
    }
    Ok(rows)
}

async fn precheck_ai(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    changes: &[ValidatedAiChange],
) -> AppResult<Vec<Option<AiRow>>> {
    let mut rows = Vec::with_capacity(changes.len());
    for change in changes {
        let current = ai::lock_current(tx, account_id, change.resource_id).await?;
        let matches = match change.operation {
            AiProviderOperation::Insert => current.is_none(),
            AiProviderOperation::Update | AiProviderOperation::Delete => {
                current.as_ref().map(|row| row.revision) == change.expected_revision
            }
        };
        if !matches {
            return Err(AppError::SyncStateChanged(
                "AI provider expected_revision no longer matches cloud state".to_owned(),
            ));
        }
        rows.push(current);
    }
    Ok(rows)
}

fn host_write_value(change: &ValidatedChange, current: Option<&HostRow>) -> Option<WriteValue> {
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
            (!current.is_some_and(|row| same_value(row, &value))).then_some(value)
        }
        HostOperation::Delete => {
            let current = current?;
            (!current.is_deleted).then(|| WriteValue {
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

fn ai_write_value(change: &ValidatedAiChange, current: Option<&AiRow>) -> Option<AiWriteValue> {
    match change.operation {
        AiProviderOperation::Insert | AiProviderOperation::Update => {
            let value = AiWriteValue::from_payload(change.payload.as_ref()?);
            (!current.is_some_and(|row| ai::same_value(row, &value))).then_some(value)
        }
        AiProviderOperation::Delete => {
            let current = current?;
            (!current.is_deleted).then(AiWriteValue::tombstone)
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
             tags = EXCLUDED.tags, status = EXCLUDED.status,
             ciphertext = EXCLUDED.ciphertext,
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
    actor: DeviceActor,
    sync_generation: i64,
    mutation_id: Uuid,
    request_hash: &[u8; 32],
) -> AppResult<Option<PushOutcome>> {
    let row = sqlx::query_as::<_, MutationRow>(
        "SELECT source_device_id, request_generation, request_hash, outcome,
                result_revision, changed_count
         FROM cloud_sync_push_mutations
         WHERE account_id = $1 AND client_mutation_id = $2",
    )
    .bind(actor.account_id())
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    if row.source_device_id != actor.device_id()
        || row.request_generation != sync_generation
        || row.request_hash.as_slice() != request_hash
    {
        return Err(AppError::Conflict(
            "client_mutation_id was already used for another request".to_owned(),
        ));
    }
    let revisions = load_results(tx, actor.account_id(), mutation_id).await?;
    let outcome = match row.outcome.as_str() {
        "applied" => PushOutcome::Applied {
            sync_generation,
            revision: row.result_revision,
            changed_count: u32::try_from(row.changed_count)
                .map_err(|_| super::invalid_stored_value())?,
            revisions,
            idempotent: true,
        },
        "unchanged" => PushOutcome::Unchanged {
            sync_generation,
            revision: row.result_revision,
            revisions,
            idempotent: true,
        },
        _ => return Err(super::invalid_stored_value()),
    };
    Ok(Some(outcome))
}

async fn load_results(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Vec<ResourceRevision>> {
    let rows = sqlx::query_as::<_, RevisionRow>(
        "SELECT resource_kind, resource_id, result_revision
         FROM cloud_sync_push_results
         WHERE account_id = $1 AND client_mutation_id = $2
         ORDER BY result_revision",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    rows.into_iter()
        .map(|row| {
            Ok(ResourceRevision {
                resource_kind: ResourceKind::parse(&row.resource_kind)
                    .ok_or_else(super::invalid_stored_value)?,
                resource_id: row.resource_id,
                cloud_revision: row.result_revision,
            })
        })
        .collect()
}

async fn insert_mutation(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &PushRequest,
    request_hash: &[u8; 32],
    result: &MutationResult<'_>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_sync_push_mutations
             (account_id, client_mutation_id, source_device_id,
              request_generation, base_revision, request_hash, outcome,
              result_revision, changed_count)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(actor.account_id())
    .bind(request.client_mutation_id)
    .bind(actor.device_id())
    .bind(request.sync_generation)
    .bind(request.base_revision)
    .bind(request_hash.as_slice())
    .bind(result.outcome)
    .bind(result.revision)
    .bind(
        i32::try_from(result.changed_count).map_err(|_| {
            AppError::Validation("push changed_count exceeds storage limit".to_owned())
        })?,
    )
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    for revision in result.revisions {
        sqlx::query(
            "INSERT INTO cloud_sync_push_results
                 (account_id, client_mutation_id, resource_kind,
                  resource_id, result_revision)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(actor.account_id())
        .bind(request.client_mutation_id)
        .bind(revision.resource_kind.as_str())
        .bind(revision.resource_id)
        .bind(revision.cloud_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

fn response(
    sync_generation: i64,
    revision: i64,
    changed_count: usize,
    revisions: Vec<ResourceRevision>,
    idempotent: bool,
) -> PushOutcome {
    if changed_count == 0 {
        PushOutcome::Unchanged {
            sync_generation,
            revision,
            revisions,
            idempotent,
        }
    } else {
        PushOutcome::Applied {
            sync_generation,
            revision,
            changed_count: u32::try_from(changed_count).unwrap_or(u32::MAX),
            revisions,
            idempotent,
        }
    }
}

fn next_revision(current: i64) -> AppResult<i64> {
    current
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict("account revision is exhausted".to_owned()))
}
