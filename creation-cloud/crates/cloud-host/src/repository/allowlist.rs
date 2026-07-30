//! Explicit per-device host download allowlists. An absent row means deny.

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::HostDownloadAllowlist;

use super::{begin, commit, lock_active_device, require_active_device, storage};

pub(crate) async fn get(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
) -> AppResult<HostDownloadAllowlist> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, account_id, device_id).await?;
    let host_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT host_id
         FROM cloud_host_download_allowlist
         WHERE account_id = $1 AND device_id = $2
         ORDER BY host_id",
    )
    .bind(account_id)
    .bind(device_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(storage)?;
    commit(tx).await?;
    Ok(HostDownloadAllowlist {
        device_id,
        host_ids,
    })
}

pub(crate) async fn replace(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
    host_ids: &[Uuid],
) -> AppResult<HostDownloadAllowlist> {
    let mut tx = begin(pool).await?;
    lock_active_device(&mut tx, account_id, device_id).await?;
    if !host_ids.is_empty() {
        let owned_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)::BIGINT
             FROM cloud_hosts
             WHERE account_id = $1
               AND id = ANY($2)
               AND NOT is_deleted",
        )
        .bind(account_id)
        .bind(host_ids)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage)?;
        if owned_count != i64::try_from(host_ids.len()).unwrap_or(i64::MAX) {
            return Err(AppError::Validation(
                "the allowlist contains an unavailable account host".to_owned(),
            ));
        }
    }

    sqlx::query(
        "DELETE FROM cloud_host_download_allowlist
         WHERE account_id = $1 AND device_id = $2",
    )
    .bind(account_id)
    .bind(device_id)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    for host_id in host_ids {
        sqlx::query(
            "INSERT INTO cloud_host_download_allowlist
                 (account_id, device_id, host_id)
             VALUES ($1, $2, $3)",
        )
        .bind(account_id)
        .bind(device_id)
        .bind(*host_id)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }

    commit(tx).await?;
    Ok(HostDownloadAllowlist {
        device_id,
        host_ids: host_ids.to_vec(),
    })
}
