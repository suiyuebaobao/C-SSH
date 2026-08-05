//! Database boundary for account-owned hosts and explicit manual synchronization.

mod admin_delete;
mod admin_sync;
mod conflict;
mod hosts;
mod pull;
mod push;
mod rekey;
mod reset;

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(crate) use admin_delete::{host as delete_admin_host, sync_record as delete_admin_sync_record};
pub(crate) use admin_sync::list as list_admin_sync_records;
pub(crate) use conflict::{
    get as get_conflict, list_open as list_open_conflicts, resolve as resolve_conflict,
};
pub(crate) use hosts::{count, get, list};
pub(crate) use pull::{ack, pull};
pub(crate) use push::push;
pub(crate) use rekey::rekey;
pub(crate) use reset::{reset, state};

pub(crate) type DbTransaction<'a> = Transaction<'a, Postgres>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyncState {
    pub current_revision: i64,
    pub compacted_through_revision: i64,
    pub sync_generation: i64,
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

    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT current_revision, compacted_through_revision, sync_generation
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
        sync_generation: row.2,
    })
}

pub(crate) fn require_sync_generation(state: SyncState, requested: i64) -> AppResult<()> {
    if requested == state.sync_generation {
        Ok(())
    } else {
        Err(AppError::SyncResyncRequired(
            "sync_generation does not match the current account generation".to_owned(),
        ))
    }
}

pub(crate) async fn require_active_device(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
) -> AppResult<()> {
    require_active_device_with_lock(tx, account_id, device_id).await
}

async fn require_active_device_with_lock(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    device_id: Uuid,
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
    let active = sqlx::query_scalar::<_, Uuid>(
        "SELECT id
         FROM devices
         WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL
         FOR SHARE",
    )
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
