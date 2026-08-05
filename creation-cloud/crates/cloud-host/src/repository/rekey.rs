//! Atomic full-candidate ciphertext rekey with generation compare-and-swap.

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    RekeyHostRevision, RekeySyncRequest, RekeySyncResponse, actor::DeviceActor,
    validation::ValidatedRekeyHost,
};

use super::{
    DbTransaction, begin, commit, lock_sync_state,
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
struct PriorRekey {
    source_device_id: Uuid,
    request_generation: i64,
    result_generation: i64,
    request_hash: Vec<u8>,
    result_revision: i64,
}

pub(crate) async fn rekey(
    pool: &PgPool,
    actor: DeviceActor,
    request: &RekeySyncRequest,
    candidates: &[ValidatedRekeyHost],
    request_hash: &[u8; 32],
) -> AppResult<RekeySyncResponse> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;

    if let Some(prior) = load_prior(&mut tx, actor.account_id(), request.mutation_id).await? {
        let matches = prior.source_device_id == actor.device_id()
            && prior.request_generation == request.sync_generation
            && prior.request_hash.as_slice() == request_hash;
        if !matches {
            return Err(AppError::Conflict(
                "mutation_id was already used by a different rekey request".to_owned(),
            ));
        }
        if state.sync_generation != prior.result_generation {
            return Err(AppError::SyncResyncRequired(
                "the rekey result belongs to an older sync generation".to_owned(),
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
    let current = lock_encrypted_hosts(&mut tx, actor.account_id()).await?;
    require_complete_candidate(&current, candidates)?;

    let next_generation = state
        .sync_generation
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict("sync_generation cannot advance".to_owned()))?;
    let mut revision = state.current_revision;
    let mut revisions = Vec::with_capacity(current.len());
    for (host, candidate) in current.into_iter().zip(candidates) {
        revision = revision
            .checked_add(1)
            .ok_or_else(|| AppError::Conflict("host revision cannot advance".to_owned()))?;
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
                ciphertext: Some(candidate.ciphertext.clone()),
                deleted: false,
            },
        )
        .await?;
        revisions.push((
            RekeyHostRevision {
                host_id: host.id,
                cloud_revision: revision,
            },
            candidate.cloud_revision,
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
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    for (result, previous_revision) in &revisions {
        sqlx::query(
            "INSERT INTO cloud_sync_rekey_results
                 (account_id, mutation_id, host_id,
                  previous_revision, result_revision)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(actor.account_id())
        .bind(request.mutation_id)
        .bind(result.host_id)
        .bind(previous_revision)
        .bind(result.cloud_revision)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }
    audit_rekey(
        &mut tx,
        actor,
        request.mutation_id,
        state.sync_generation,
        next_generation,
        revision,
        revisions.len(),
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
) -> AppResult<Vec<RekeyHostRevision>> {
    sqlx::query_as::<_, RekeyResultRow>(
        "SELECT host_id, result_revision
         FROM cloud_sync_rekey_results
         WHERE account_id = $1 AND mutation_id = $2
         ORDER BY host_id",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)
    .map(|rows| rows.into_iter().map(RekeyResultRow::view).collect())
}

#[derive(FromRow)]
struct RekeyResultRow {
    host_id: Uuid,
    result_revision: i64,
}

impl RekeyResultRow {
    fn view(self) -> RekeyHostRevision {
        RekeyHostRevision {
            host_id: self.host_id,
            cloud_revision: self.result_revision,
        }
    }
}

async fn lock_encrypted_hosts(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<Vec<CurrentEncryptedHost>> {
    sqlx::query_as::<_, CurrentEncryptedHost>(
        "SELECT id, address, port, name, platform, tags, status, revision
         FROM cloud_hosts
         WHERE account_id = $1 AND NOT is_deleted AND ciphertext IS NOT NULL
         ORDER BY id
         FOR UPDATE",
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)
}

fn require_complete_candidate(
    current: &[CurrentEncryptedHost],
    candidates: &[ValidatedRekeyHost],
) -> AppResult<()> {
    let matches = current.len() == candidates.len()
        && current.iter().zip(candidates).all(|(stored, candidate)| {
            stored.id == candidate.host_id && stored.revision == candidate.cloud_revision
        });
    if matches {
        Ok(())
    } else {
        Err(AppError::Conflict(
            "rekey candidate does not match the complete encrypted host snapshot".to_owned(),
        ))
    }
}

async fn clear_stale_sync_state(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
    for statement in [
        "DELETE FROM cloud_host_pull_decisions WHERE account_id = $1",
        "DELETE FROM cloud_host_device_deliveries WHERE account_id = $1",
        "DELETE FROM cloud_host_pull_watermarks WHERE account_id = $1",
        "DELETE FROM cloud_host_device_checkpoints WHERE account_id = $1",
        "DELETE FROM cloud_host_conflicts WHERE account_id = $1",
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
    previous_current_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM cloud_host_versions
         WHERE account_id = $1
           AND revision <= $2
           AND ciphertext IS NOT NULL",
    )
    .bind(account_id)
    .bind(previous_current_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn audit_rekey(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    previous_generation: i64,
    next_generation: i64,
    result_revision: i64,
    changed_hosts: usize,
) -> AppResult<()> {
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "changed_hosts": i64::try_from(changed_hosts).unwrap_or(i64::MAX),
        "result_revision": result_revision,
        "previous_sync_generation": previous_generation,
        "sync_generation": next_generation
    });
    sqlx::query(
        "INSERT INTO audit_events
             (id, actor_account_id, action, resource_kind, resource_id,
              outcome, request_id, details)
         VALUES ($1, $2, 'sync.encrypted_data_rekey', 'sync_account', $3,
                 'success', $4, $5)",
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
    revisions: Vec<RekeyHostRevision>,
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

    fn current(id: Uuid, revision: i64) -> CurrentEncryptedHost {
        CurrentEncryptedHost {
            id,
            address: "node.example.com".to_owned(),
            port: 22,
            name: "node".to_owned(),
            platform: "linux".to_owned(),
            tags: json!([]),
            status: "active".to_owned(),
            revision,
        }
    }

    fn candidate(id: Uuid, revision: i64) -> ValidatedRekeyHost {
        ValidatedRekeyHost {
            host_id: id,
            cloud_revision: revision,
            ciphertext: vec![1; 32],
        }
    }

    #[test]
    fn candidate_must_cover_the_exact_encrypted_snapshot() {
        let first = Uuid::now_v7();
        let second = Uuid::now_v7();
        let mut ids = [first, second];
        ids.sort_unstable();
        let stored = vec![current(ids[0], 4), current(ids[1], 8)];
        assert!(
            require_complete_candidate(&stored, &[candidate(ids[0], 4), candidate(ids[1], 8)])
                .is_ok()
        );
        assert!(matches!(
            require_complete_candidate(&stored, &[candidate(ids[0], 4)]),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            require_complete_candidate(&stored, &[candidate(ids[0], 4), candidate(ids[1], 7)]),
            Err(AppError::Conflict(_))
        ));
    }
}
