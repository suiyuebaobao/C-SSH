//! 先完整预检 Host 与 AI 资源 CAS，再在一个事务中整批写入统一 revision 流。

use cloud_domain::{AppError, AppResult};
use cloud_notification::{AccountNotificationEvent, record_account_event};
use cloud_store::PgPool;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    AiProviderOperation, HostOperation, ProxyProfileOperation, PushOutcome, PushRequest,
    ResourceKind, ResourceRevision,
    actor::DeviceActor,
    validation::{ValidatedAiChange, ValidatedChange, ValidatedProxyProfileChange, ValidatedPush},
};

use super::{
    DbTransaction,
    ai::{self, AiRow, AiWriteValue},
    begin,
    capacity::{CapacityInputs, enforce_encrypted_resource_limit},
    commit,
    hosts::{HostRow, lock_current},
    lock_sync_state,
    proxy_profile::{self, ProxyProfileRow, ProxyProfileWriteValue},
    pull::{safe_checkpoint_revision, save_checkpoint},
    require_active_device, require_base_revision, require_configured_envelope,
    require_protection_version, require_sync_contract, require_sync_generation, storage,
};

#[derive(FromRow)]
struct MutationRow {
    source_device_id: Uuid,
    request_generation: i64,
    request_protection_epoch: i64,
    request_protection_revision: i64,
    request_hash: Vec<u8>,
    request_hash_scheme: String,
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
    pub(super) address: Option<String>,
    pub(super) port: Option<i32>,
    pub(super) name: Option<String>,
    pub(super) platform: Option<String>,
    pub(super) tags: Option<Value>,
    pub(super) status: Option<String>,
    pub(super) ciphertext: Option<Vec<u8>>,
    pub(super) deleted: bool,
    pub(super) metadata_encrypted: bool,
}

pub(crate) async fn push(
    pool: &PgPool,
    actor: DeviceActor,
    request: &PushRequest,
    changes: &ValidatedPush,
    request_hash: &[u8; 32],
    request_hash_scheme: &str,
) -> AppResult<PushOutcome> {
    let account_id = actor.account_id();
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, account_id, actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, account_id).await?;
    require_sync_generation(state, request.sync_generation)?;
    require_sync_contract(state, request.sync_contract_version)?;
    if !changes.host_changes.is_empty() {
        let legacy_host_present = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM cloud_hosts
                           WHERE account_id=$1 AND NOT metadata_encrypted)
                 OR EXISTS(SELECT 1 FROM cloud_host_versions
                           WHERE account_id=$1 AND NOT metadata_encrypted)",
        )
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage)?;
        if legacy_host_present {
            return Err(AppError::SyncContractUpgradeRequired(
                "Host metadata迁移完成前不能提交普通Host变更".to_owned(),
            ));
        }
    }
    if let Some(outcome) =
        replay_mutation(&mut tx, actor, request, request_hash, request_hash_scheme).await?
    {
        commit(tx).await?;
        return Ok(outcome);
    }
    require_protection_version(state, request.protection_epoch, request.protection_revision)?;
    require_configured_envelope(&mut tx, account_id, state).await?;
    require_base_revision(state, request.base_revision)?;

    // 所有行锁与 expected revision 检查必须在任何业务写入之前完成。
    let host_rows = precheck_hosts(&mut tx, account_id, &changes.host_changes).await?;
    let ai_rows = precheck_ai(&mut tx, account_id, &changes.ai_changes).await?;
    let proxy_rows =
        precheck_proxy_profiles(&mut tx, account_id, &changes.proxy_profile_changes).await?;
    enforce_encrypted_resource_limit(
        &mut tx,
        account_id,
        CapacityInputs {
            host_changes: &changes.host_changes,
            host_rows: &host_rows,
            ai_changes: &changes.ai_changes,
            ai_rows: &ai_rows,
            proxy_changes: &changes.proxy_profile_changes,
            proxy_rows: &proxy_rows,
        },
    )
    .await?;

    let mut revision = state.current_revision;
    let mut changed_count = 0_usize;
    let mut revisions = Vec::with_capacity(
        changes.host_changes.len() + changes.ai_changes.len() + changes.proxy_profile_changes.len(),
    );
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
    for (change, current) in changes.proxy_profile_changes.iter().zip(proxy_rows.iter()) {
        let result_revision =
            if let Some(value) = proxy_profile_write_value(change, current.as_ref()) {
                revision = next_revision(revision)?;
                proxy_profile::write(&mut tx, actor, change.resource_id, revision, value).await?;
                changed_count += 1;
                revision
            } else {
                current
                    .as_ref()
                    .map(|row| row.revision)
                    .ok_or_else(super::invalid_stored_value)?
            };
        revisions.push(ResourceRevision {
            resource_kind: ResourceKind::ProxyProfile,
            resource_id: change.resource_id,
            cloud_revision: result_revision,
        });
    }
    revisions.sort_unstable_by_key(|result| result.cloud_revision);

    if changed_count > 0 || !changes.host_changes.is_empty() {
        sqlx::query(
            "UPDATE cloud_host_sync_states
             SET current_revision = $2,
                 minimum_sync_contract_version = CASE WHEN $3
                     THEN GREATEST(minimum_sync_contract_version, 4)
                     ELSE minimum_sync_contract_version END,
                 updated_at = now()
             WHERE account_id = $1",
        )
        .bind(account_id)
        .bind(revision)
        .bind(!changes.host_changes.is_empty())
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
    insert_mutation(
        &mut tx,
        actor,
        request,
        request_hash,
        request_hash_scheme,
        &mutation,
    )
    .await?;
    record_account_event(
        &mut tx,
        account_id,
        AccountNotificationEvent::SyncUploadCompleted {
            mutation_id: request.client_mutation_id,
        },
    )
    .await?;
    commit(tx).await?;
    Ok(response(
        request.sync_generation,
        request.protection_epoch,
        request.protection_revision,
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

async fn precheck_proxy_profiles(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    changes: &[ValidatedProxyProfileChange],
) -> AppResult<Vec<Option<ProxyProfileRow>>> {
    let mut rows = Vec::with_capacity(changes.len());
    for change in changes {
        let current = proxy_profile::lock_current(tx, account_id, change.resource_id).await?;
        let matches = match change.operation {
            ProxyProfileOperation::Insert => current.is_none(),
            ProxyProfileOperation::Update | ProxyProfileOperation::Delete => {
                current.as_ref().map(|row| row.revision) == change.expected_revision
            }
        };
        if !matches {
            return Err(AppError::SyncStateChanged(
                "proxy profile expected_revision no longer matches cloud state".to_owned(),
            ));
        }
        rows.push(current);
    }
    Ok(rows)
}

fn host_write_value(change: &ValidatedChange, current: Option<&HostRow>) -> Option<WriteValue> {
    match change.operation {
        HostOperation::Insert | HostOperation::Update => {
            let ciphertext = change.ciphertext.as_ref()?.clone()?;
            let value = WriteValue {
                address: None,
                port: None,
                name: None,
                platform: None,
                tags: None,
                status: None,
                ciphertext: Some(ciphertext),
                deleted: false,
                metadata_encrypted: change.metadata_encrypted,
            };
            (!current.is_some_and(|row| same_value(row, &value))).then_some(value)
        }
        HostOperation::Delete => {
            let current = current?;
            (!current.is_deleted).then_some(WriteValue {
                address: None,
                port: None,
                name: None,
                platform: None,
                tags: None,
                status: None,
                ciphertext: None,
                deleted: true,
                metadata_encrypted: true,
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

fn proxy_profile_write_value(
    change: &ValidatedProxyProfileChange,
    current: Option<&ProxyProfileRow>,
) -> Option<ProxyProfileWriteValue> {
    match change.operation {
        ProxyProfileOperation::Insert | ProxyProfileOperation::Update => {
            let value = ProxyProfileWriteValue::from_payload(change.payload.as_ref()?);
            (!current.is_some_and(|row| proxy_profile::same_value(row, &value))).then_some(value)
        }
        ProxyProfileOperation::Delete => {
            let current = current?;
            (!current.is_deleted).then(ProxyProfileWriteValue::tombstone)
        }
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
        && current.metadata_encrypted == value.metadata_encrypted
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
               ciphertext, source_device_id, revision, is_deleted, metadata_encrypted)
          VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
         ON CONFLICT (account_id, id) DO UPDATE SET
             address = EXCLUDED.address, port = EXCLUDED.port,
             name = EXCLUDED.name, platform = EXCLUDED.platform,
             tags = EXCLUDED.tags, status = EXCLUDED.status,
             ciphertext = EXCLUDED.ciphertext,
             source_device_id = EXCLUDED.source_device_id,
             revision = EXCLUDED.revision, is_deleted = EXCLUDED.is_deleted,
             metadata_encrypted = EXCLUDED.metadata_encrypted,
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
    .bind(value.metadata_encrypted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO cloud_host_versions
             (account_id, host_id, revision, address, port, name, platform,
               tags, status, ciphertext, source_device_id, is_deleted, metadata_encrypted)
          VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
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
    .bind(value.metadata_encrypted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

include!("push/replay.rs");
