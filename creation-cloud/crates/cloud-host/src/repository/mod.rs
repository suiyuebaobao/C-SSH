//! 封装账号主机、AI provider 密文与统一 revision 流的数据库边界。

mod admin_delete;
mod admin_sync;
mod ai;
mod capacity;
mod hosts;
mod protection;
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
pub(crate) use hosts::{count, get, list};
pub(crate) use protection::{
    cancel_challenge, change_protection, get_protection, issue_reset_challenge, legacy_pull,
    mark_challenge_sent, migrate_protection, setup_protection, verify_reset_challenge,
};
pub(crate) use pull::{ack, pull};
pub(crate) use push::push;
pub(crate) use rekey::rekey;
pub(crate) use reset::reset;

pub(crate) type DbTransaction<'a> = Transaction<'a, Postgres>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyncState {
    pub current_revision: i64,
    pub compacted_through_revision: i64,
    pub sync_generation: i64,
    pub protection_epoch: i64,
    pub protection_revision: i64,
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

    let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT current_revision, compacted_through_revision, sync_generation,
                protection_epoch, protection_revision
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
        protection_epoch: row.3,
        protection_revision: row.4,
    })
}

pub(crate) fn require_protection_version(
    state: SyncState,
    requested_epoch: i64,
    requested_revision: i64,
) -> AppResult<()> {
    if requested_epoch == state.protection_epoch && requested_revision == state.protection_revision
    {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "data protection epoch/revision changed; refresh the envelope".to_owned(),
        ))
    }
}

pub(crate) async fn require_configured_envelope(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    state: SyncState,
) -> AppResult<()> {
    let configured = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1 FROM cloud_data_protection_envelopes
             WHERE account_id = $1 AND sync_generation = $2
               AND protection_epoch = $3 AND protection_revision = $4
         )",
    )
    .bind(account_id)
    .bind(state.sync_generation)
    .bind(state.protection_epoch)
    .bind(state.protection_revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if configured {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "account data protection is not configured".to_owned(),
        ))
    }
}

pub(crate) fn require_sync_generation(state: SyncState, requested: i64) -> AppResult<()> {
    if requested == state.sync_generation {
        Ok(())
    } else {
        Err(AppError::sync_generation_changed(
            "sync_generation does not match the current account generation",
        ))
    }
}

pub(crate) fn require_retained_revision(state: SyncState, requested: i64) -> AppResult<()> {
    if requested >= state.compacted_through_revision {
        Ok(())
    } else {
        Err(AppError::sync_history_compacted(
            "the requested sync revision is below the compaction floor",
        ))
    }
}

pub(crate) fn require_base_revision(state: SyncState, requested: i64) -> AppResult<()> {
    if requested == state.current_revision {
        Ok(())
    } else {
        Err(AppError::SyncStateChanged(
            "base_revision does not match the current account revision".to_owned(),
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
