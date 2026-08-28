async fn replay_mutation(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    sync_generation: i64,
    protection_epoch: i64,
    protection_revision: i64,
    mutation_id: Uuid,
    request_hash: &[u8; 32],
) -> AppResult<Option<PushOutcome>> {
    let row = sqlx::query_as::<_, MutationRow>(
        "SELECT source_device_id, request_generation, request_protection_epoch,
                request_protection_revision, request_hash, outcome,
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
        || row.request_protection_epoch != protection_epoch
        || row.request_protection_revision != protection_revision
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
            protection_epoch,
            protection_revision,
            revision: row.result_revision,
            changed_count: u32::try_from(row.changed_count)
                .map_err(|_| super::invalid_stored_value())?,
            revisions,
            idempotent: true,
        },
        "unchanged" => PushOutcome::Unchanged {
            sync_generation,
            protection_epoch,
            protection_revision,
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
              request_generation, request_protection_epoch,
              request_protection_revision, base_revision, request_hash, outcome,
              result_revision, changed_count)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    )
    .bind(actor.account_id())
    .bind(request.client_mutation_id)
    .bind(actor.device_id())
    .bind(request.sync_generation)
    .bind(request.protection_epoch)
    .bind(request.protection_revision)
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
    protection_epoch: i64,
    protection_revision: i64,
    revision: i64,
    changed_count: usize,
    revisions: Vec<ResourceRevision>,
    idempotent: bool,
) -> PushOutcome {
    if changed_count == 0 {
        PushOutcome::Unchanged {
            sync_generation,
            protection_epoch,
            protection_revision,
            revision,
            revisions,
            idempotent,
        }
    } else {
        PushOutcome::Applied {
            sync_generation,
            protection_epoch,
            protection_revision,
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
