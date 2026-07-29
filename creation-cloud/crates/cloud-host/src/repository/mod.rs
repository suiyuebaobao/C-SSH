//! Database boundary for account-owned hosts and explicit manual synchronization.

mod allowlist;
mod conflict;
mod hosts;
mod pull;
mod push;

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(crate) use allowlist::{get as get_allowlist, replace as replace_allowlist};
pub(crate) use conflict::{
    get as get_conflict, list_open as list_open_conflicts, resolve as resolve_conflict,
};
pub(crate) use hosts::{count, get, list};
pub(crate) use pull::{ack, pull};
pub(crate) use push::push;

pub(crate) type DbTransaction<'a> = Transaction<'a, Postgres>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyncState {
    pub current_revision: i64,
    pub compacted_through_revision: i64,
}

pub(crate) async fn begin(pool: &PgPool) -> AppResult<DbTransaction<'_>> {
    pool.begin().await.map_err(storage)
}

pub(crate) async fn lock_sync_state(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
) -> AppResult<SyncState> {
    sqlx::query(
        "INSERT INTO cloud_host_sync_states (account_id)
         VALUES ($1)
         ON CONFLICT (account_id) DO NOTHING",
    )
    .bind(account_id)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;

    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT current_revision, compacted_through_revision
         FROM cloud_host_sync_states
         WHERE account_id = $1
         FOR UPDATE",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(SyncState {
        current_revision: row.0,
        compacted_through_revision: row.1,
    })
}

pub(crate) async fn require_active_device(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
) -> AppResult<()> {
    require_active_device_with_lock(tx, account_id, device_id, false).await
}

pub(crate) async fn lock_active_device(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
) -> AppResult<()> {
    require_active_device_with_lock(tx, account_id, device_id, true).await
}

async fn require_active_device_with_lock(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
    exclusive: bool,
) -> AppResult<()> {
    let account_active = sqlx::query_scalar::<_, Uuid>(
        "SELECT id
         FROM accounts
         WHERE id = $1 AND status = 'active'
         FOR SHARE",
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?
    .is_some();
    if !account_active {
        return Err(AppError::Unauthorized(
            "the account is not active".to_owned(),
        ));
    }
    let statement = if exclusive {
        "SELECT id
         FROM devices
         WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL
         FOR UPDATE"
    } else {
        "SELECT id
         FROM devices
         WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL
         FOR SHARE"
    };
    let active = sqlx::query_scalar::<_, Uuid>(statement)
        .bind(account_id)
        .bind(device_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage)?
        .is_some();
    if active {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "the bound device is not active for this account".to_owned(),
        ))
    }
}

pub(crate) async fn commit(tx: DbTransaction<'_>) -> AppResult<()> {
    tx.commit().await.map_err(storage)
}

pub(crate) fn storage(_: sqlx::Error) -> AppError {
    AppError::Storage("host synchronization storage operation failed".to_owned())
}

pub(crate) fn invalid_stored_value() -> AppError {
    AppError::Storage("stored host synchronization data is invalid".to_owned())
}
