async fn prune_delivery_window(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    snapshot_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM cloud_sync_resource_deliveries
         WHERE account_id=$1 AND device_id=$2 AND snapshot_revision<>$3",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM cloud_sync_pull_watermarks
         WHERE account_id=$1 AND device_id=$2 AND snapshot_revision<>$3",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn clear_delivery_snapshot(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    snapshot_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM cloud_sync_resource_deliveries
         WHERE account_id=$1 AND device_id=$2 AND snapshot_revision=$3",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM cloud_sync_pull_watermarks
         WHERE account_id=$1 AND device_id=$2 AND snapshot_revision=$3",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn record_deliveries(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    rows: &[PullIdentityRow],
    snapshot_revision: i64,
) -> AppResult<()> {
    for row in rows {
        sqlx::query(
            "INSERT INTO cloud_sync_resource_deliveries
                 (account_id, device_id, resource_kind, resource_id,
                  delivered_revision, snapshot_revision, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,now())
             ON CONFLICT (account_id, device_id, resource_kind, resource_id,
                          delivered_revision, snapshot_revision)
             DO UPDATE SET updated_at = now()",
        )
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(&row.resource_kind)
        .bind(row.resource_id)
        .bind(row.revision)
        .bind(snapshot_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn record_pull_watermark(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledgeable_revision: i64,
    snapshot_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_sync_pull_watermarks
             (account_id, device_id, acknowledgeable_revision,
              snapshot_revision, delivered_at)
         VALUES ($1,$2,$3,$4,now())
         ON CONFLICT (account_id, device_id, acknowledgeable_revision)
         DO UPDATE SET snapshot_revision = GREATEST(
             cloud_sync_pull_watermarks.snapshot_revision, EXCLUDED.snapshot_revision),
             delivered_at = now()",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledgeable_revision)
    .bind(snapshot_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn delivered_watermark(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    acknowledged_revision: i64,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar::<_, i64>(
        "SELECT snapshot_revision FROM cloud_sync_pull_watermarks
         WHERE account_id = $1 AND device_id = $2
           AND acknowledgeable_revision = $3",
    )
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(acknowledged_revision)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}
