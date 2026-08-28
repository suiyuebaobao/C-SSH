async fn persist_results(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
    results: &[(ResourceRevision, i64)],
) -> AppResult<()> {
    for (result, previous_revision) in results {
        sqlx::query(
            "INSERT INTO cloud_data_protection_migration_results
                 (account_id,mutation_id,resource_kind,resource_id,
                  previous_revision,result_revision)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(account_id)
        .bind(mutation_id)
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

async fn load_results(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Vec<ResourceRevision>> {
    let rows = sqlx::query_as::<_, (String, Uuid, i64)>(
        "SELECT resource_kind,resource_id,result_revision
         FROM cloud_data_protection_migration_results
         WHERE account_id=$1 AND mutation_id=$2 ORDER BY result_revision",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    rows.into_iter()
        .map(|(kind, resource_id, cloud_revision)| {
            Ok(ResourceRevision {
                resource_kind: ResourceKind::parse(&kind)
                    .ok_or_else(super::super::invalid_stored_value)?,
                resource_id,
                cloud_revision,
            })
        })
        .collect()
}

async fn require_envelope_absent(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
    let present = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM cloud_data_protection_envelopes WHERE account_id=$1)",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if present {
        Err(AppError::Conflict(
            "legacy migration is unavailable after protection setup".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn next(value: i64, field: &str) -> AppResult<i64> {
    value
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict(format!("{field} cannot advance")))
}
