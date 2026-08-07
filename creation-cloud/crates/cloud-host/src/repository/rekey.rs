//! 对 Host 与 AI provider 账号的完整密文集合执行原子 rekey 和 generation CAS。

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    RekeySyncRequest, RekeySyncResponse, ResourceKind, ResourceRevision, actor::DeviceActor,
    validation::ValidatedRekeyResource,
};

use super::{
    DbTransaction,
    ai::{self, AiWriteValue},
    begin,
    capacity::require_current_within_limit,
    commit, lock_sync_state,
    pull::record_rekey_snapshot,
    push::{WriteValue, write_host},
    require_active_device, require_sync_generation, storage,
};

#[derive(FromRow)]
struct CurrentEncryptedHost {
    id: Uuid,
    address: String,
    port: i32,
    name: String,
    platform: String,
    tags: Value,
    status: String,
    revision: i64,
}

#[derive(FromRow)]
struct CurrentEncryptedAi {
    id: Uuid,
    revision: i64,
}

enum CurrentEncryptedResource {
    Host(CurrentEncryptedHost),
    AiProviderAccount(CurrentEncryptedAi),
}

impl CurrentEncryptedResource {
    const fn kind(&self) -> ResourceKind {
        match self {
            Self::Host(_) => ResourceKind::Host,
            Self::AiProviderAccount(_) => ResourceKind::AiProviderAccount,
        }
    }

    const fn id(&self) -> Uuid {
        match self {
            Self::Host(value) => value.id,
            Self::AiProviderAccount(value) => value.id,
        }
    }

    const fn revision(&self) -> i64 {
        match self {
            Self::Host(value) => value.revision,
            Self::AiProviderAccount(value) => value.revision,
        }
    }
}

#[derive(FromRow)]
struct PriorRekey {
    source_device_id: Uuid,
    request_generation: i64,
    result_generation: i64,
    request_hash: Vec<u8>,
    result_revision: i64,
}

#[derive(FromRow)]
struct RekeyResultRow {
    resource_kind: String,
    resource_id: Uuid,
    result_revision: i64,
}

pub(crate) async fn rekey(
    pool: &PgPool,
    actor: DeviceActor,
    request: &RekeySyncRequest,
    candidates: &[ValidatedRekeyResource],
    request_hash: &[u8; 32],
) -> AppResult<RekeySyncResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;

    if let Some(prior) = load_prior(&mut tx, actor.account_id(), request.mutation_id).await? {
        if prior.source_device_id != actor.device_id()
            || prior.request_generation != request.sync_generation
            || prior.request_hash.as_slice() != request_hash
        {
            return Err(AppError::Conflict(
                "mutation_id was already used by a different rekey request".to_owned(),
            ));
        }
        if state.sync_generation != prior.result_generation {
            return Err(AppError::sync_generation_changed(
                "the rekey result belongs to an older sync generation",
            ));
        }
        let revisions = load_results(&mut tx, actor.account_id(), request.mutation_id).await?;
        commit(tx).await?;
        return Ok(response(
            prior.result_generation,
            prior.result_revision,
            revisions,
            true,
        ));
    }

    require_sync_generation(state, request.sync_generation)?;
    require_current_within_limit(&mut tx, actor.account_id()).await?;
    let current = lock_encrypted_resources(&mut tx, actor.account_id()).await?;
    require_complete_candidate(&current, candidates)?;
    let next_generation = state
        .sync_generation
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict("sync_generation cannot advance".to_owned()))?;

    let mut revision = state.current_revision;
    let mut revisions = Vec::with_capacity(current.len());
    let mut changed_hosts = 0_usize;
    let mut changed_ai = 0_usize;
    for (stored, candidate) in current.into_iter().zip(candidates) {
        revision = revision
            .checked_add(1)
            .ok_or_else(|| AppError::Conflict("account revision cannot advance".to_owned()))?;
        let kind = stored.kind();
        let resource_id = stored.id();
        match (stored, candidate) {
            (
                CurrentEncryptedResource::Host(host),
                ValidatedRekeyResource::Host { ciphertext, .. },
            ) => {
                write_host(
                    &mut tx,
                    actor,
                    host.id,
                    revision,
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
                changed_hosts += 1;
            }
            (
                CurrentEncryptedResource::AiProviderAccount(_),
                ValidatedRekeyResource::AiProviderAccount { payload, .. },
            ) => {
                ai::write(
                    &mut tx,
                    actor,
                    resource_id,
                    revision,
                    AiWriteValue::from_payload(payload),
                )
                .await?;
                changed_ai += 1;
            }
            _ => return Err(super::invalid_stored_value()),
        }
        revisions.push((
            ResourceRevision {
                resource_kind: kind,
                resource_id,
                cloud_revision: revision,
            },
            candidate.cloud_revision(),
        ));
    }

    purge_prior_ciphertext_versions(&mut tx, actor.account_id(), state.current_revision).await?;
    clear_stale_sync_state(&mut tx, actor.account_id()).await?;
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET current_revision = $2, sync_generation = $3, updated_at = now()
         WHERE account_id = $1",
    )
    .bind(actor.account_id())
    .bind(revision)
    .bind(next_generation)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    let settled_revisions = revisions
        .iter()
        .map(|(result, _)| *result)
        .collect::<Vec<_>>();
    // rekey 响应只证明新密文已投递给来源设备；本地是否采用仍必须由客户端 ACK，
    // 因此这里不得写 keep_local，也不得推进该设备 checkpoint。
    record_rekey_snapshot(&mut tx, actor, &settled_revisions, revision).await?;
    persist_rekey(
        &mut tx,
        actor,
        request,
        request_hash,
        next_generation,
        revision,
        &revisions,
    )
    .await?;
    audit_rekey(
        &mut tx,
        actor,
        request.mutation_id,
        state.sync_generation,
        next_generation,
        revision,
        changed_hosts,
        changed_ai,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(response(
        next_generation,
        revision,
        revisions.into_iter().map(|value| value.0).collect(),
        false,
    ))
}

async fn load_prior(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Option<PriorRekey>> {
    sqlx::query_as::<_, PriorRekey>(
        "SELECT source_device_id, request_generation, result_generation,
                request_hash, result_revision
         FROM cloud_sync_rekey_mutations
         WHERE account_id = $1 AND mutation_id = $2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

async fn load_results(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Vec<ResourceRevision>> {
    let rows = sqlx::query_as::<_, RekeyResultRow>(
        "SELECT resource_kind, resource_id, result_revision
         FROM cloud_sync_rekey_resource_results
         WHERE account_id = $1 AND mutation_id = $2
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

async fn lock_encrypted_resources(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<Vec<CurrentEncryptedResource>> {
    let hosts = sqlx::query_as::<_, CurrentEncryptedHost>(
        "SELECT id, address, port, name, platform, tags, status, revision
         FROM cloud_hosts
         WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL
         ORDER BY id FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let ai = sqlx::query_as::<_, CurrentEncryptedAi>(
        "SELECT id, revision FROM cloud_ai_provider_configs
         WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL
         ORDER BY id FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let mut resources = hosts
        .into_iter()
        .map(CurrentEncryptedResource::Host)
        .chain(
            ai.into_iter()
                .map(CurrentEncryptedResource::AiProviderAccount),
        )
        .collect::<Vec<_>>();
    resources.sort_unstable_by_key(|resource| (resource.kind().as_str(), resource.id()));
    Ok(resources)
}

fn require_complete_candidate(
    current: &[CurrentEncryptedResource],
    candidates: &[ValidatedRekeyResource],
) -> AppResult<()> {
    let matches = current.len() == candidates.len()
        && current.iter().zip(candidates).all(|(stored, candidate)| {
            stored.kind() == candidate.resource_kind()
                && stored.id() == candidate.resource_id()
                && stored.revision() == candidate.cloud_revision()
        });
    if matches {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "rekey candidate does not match the complete encrypted resource snapshot".to_owned(),
        ))
    }
}

async fn persist_rekey(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &RekeySyncRequest,
    request_hash: &[u8; 32],
    next_generation: i64,
    revision: i64,
    revisions: &[(ResourceRevision, i64)],
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_sync_rekey_mutations
             (account_id, mutation_id, source_device_id, request_generation,
              result_generation, request_hash, result_revision, changed_count)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(actor.account_id())
    .bind(request.mutation_id)
    .bind(actor.device_id())
    .bind(request.sync_generation)
    .bind(next_generation)
    .bind(request_hash.as_slice())
    .bind(revision)
    .bind(i32::try_from(revisions.len()).unwrap_or(i32::MAX))
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    for (result, previous_revision) in revisions {
        sqlx::query(
            "INSERT INTO cloud_sync_rekey_resource_results
                 (account_id, mutation_id, resource_kind, resource_id,
                  previous_revision, result_revision)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(actor.account_id())
        .bind(request.mutation_id)
        .bind(result.resource_kind.as_str())
        .bind(result.resource_id)
        .bind(previous_revision)
        .bind(result.cloud_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn clear_stale_sync_state(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
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

async fn purge_prior_ciphertext_versions(
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

#[allow(clippy::too_many_arguments)]
async fn audit_rekey(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    previous_generation: i64,
    next_generation: i64,
    result_revision: i64,
    changed_hosts: usize,
    changed_ai: usize,
) -> AppResult<()> {
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "changed_hosts": i64::try_from(changed_hosts).unwrap_or(i64::MAX),
        "changed_ai_providers": i64::try_from(changed_ai).unwrap_or(i64::MAX),
        "result_revision": result_revision,
        "previous_sync_generation": previous_generation,
        "sync_generation": next_generation
    });
    sqlx::query(
        "INSERT INTO audit_events
             (id, actor_account_id, action, resource_kind, resource_id,
              outcome, request_id, details)
         VALUES ($1,$2,'sync.encrypted_data_rekey_v2','sync_account',$3,
                 'success',$4,$5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(actor.account_id().to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| AppError::Storage("failed to persist encrypted sync rekey audit".to_owned()))?;
    Ok(())
}

fn response(
    sync_generation: i64,
    current_revision: i64,
    revisions: Vec<ResourceRevision>,
    idempotent: bool,
) -> RekeySyncResponse {
    RekeySyncResponse {
        status: "rekeyed".to_owned(),
        sync_generation,
        current_revision,
        revisions,
        idempotent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_ai(id: Uuid, revision: i64) -> CurrentEncryptedResource {
        CurrentEncryptedResource::AiProviderAccount(CurrentEncryptedAi { id, revision })
    }

    fn candidate_ai(id: Uuid, revision: i64) -> ValidatedRekeyResource {
        ValidatedRekeyResource::AiProviderAccount {
            resource_id: id,
            cloud_revision: revision,
            payload: crate::validation::ValidatedAiPayload {
                ciphertext: vec![1],
                nonce: vec![2],
                envelope_metadata: json!({"v": 1}),
            },
        }
    }

    #[test]
    fn candidate_must_cover_exact_typed_snapshot() {
        let id = Uuid::now_v7();
        assert!(require_complete_candidate(&[current_ai(id, 4)], &[candidate_ai(id, 4)]).is_ok());
        assert!(matches!(
            require_complete_candidate(&[current_ai(id, 4)], &[candidate_ai(id, 3)]),
            Err(AppError::SyncStateChanged(_))
        ));
    }
}
