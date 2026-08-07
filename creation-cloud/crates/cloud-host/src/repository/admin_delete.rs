//! 原子执行管理员发起的账号主机删除与同步历史隐藏。

use cloud_domain::{
    AdminActor, AppError, AppResult, current_request_id, mark_semantic_audit_recorded,
};
use cloud_store::PgPool;
use serde_json::json;
use uuid::Uuid;

use super::{DbTransaction, begin, commit, lock_sync_state, storage};

const TOMBSTONE_ADDRESS: &str = "deleted.invalid";
const TOMBSTONE_NAME: &str = "deleted";
const TOMBSTONE_PLATFORM: &str = "deleted";
const TOMBSTONE_PORT: i32 = 1;

pub(crate) async fn host(
    pool: &PgPool,
    actor: &AdminActor,
    account_id: Uuid,
    host_id: Uuid,
) -> AppResult<()> {
    let mut tx = begin(pool).await?;
    if !account_exists(&mut tx, account_id).await? {
        return not_found(tx, "host was not found").await;
    }
    let state = lock_sync_state(&mut tx, account_id).await?;
    let source_device_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT source_device_id
         FROM cloud_hosts
         WHERE account_id = $1 AND id = $2 AND NOT is_deleted
         FOR UPDATE",
    )
    .bind(account_id)
    .bind(host_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?;
    let Some(source_device_id) = source_device_id else {
        return not_found(tx, "host was not found").await;
    };
    let revision = state
        .current_revision
        .checked_add(1)
        .ok_or_else(|| AppError::Storage("host revision is exhausted".to_owned()))?;

    for statement in [
        "DELETE FROM cloud_sync_pull_decisions
         WHERE account_id = $1 AND resource_kind = 'host' AND resource_id = $2",
        "DELETE FROM cloud_sync_resource_deliveries
         WHERE account_id = $1 AND resource_kind = 'host' AND resource_id = $2",
        "DELETE FROM cloud_host_versions
         WHERE account_id = $1 AND host_id = $2",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .bind(host_id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
    }

    sqlx::query(
        "UPDATE cloud_hosts
         SET address = $3, port = $4, name = $5, platform = $6,
             tags = '[]'::jsonb, status = 'archived', ciphertext = NULL,
             source_device_id = $7, revision = $8, is_deleted = TRUE,
             updated_at = now()
         WHERE account_id = $1 AND id = $2 AND NOT is_deleted",
    )
    .bind(account_id)
    .bind(host_id)
    .bind(TOMBSTONE_ADDRESS)
    .bind(TOMBSTONE_PORT)
    .bind(TOMBSTONE_NAME)
    .bind(TOMBSTONE_PLATFORM)
    .bind(source_device_id)
    .bind(revision)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO cloud_host_versions
             (account_id, host_id, revision, address, port, name, platform,
              tags, status, ciphertext, source_device_id, is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,'[]'::jsonb,'archived',NULL,$8,TRUE)",
    )
    .bind(account_id)
    .bind(host_id)
    .bind(revision)
    .bind(TOMBSTONE_ADDRESS)
    .bind(TOMBSTONE_PORT)
    .bind(TOMBSTONE_NAME)
    .bind(TOMBSTONE_PLATFORM)
    .bind(source_device_id)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET current_revision = $2, updated_at = now()
         WHERE account_id = $1",
    )
    .bind(account_id)
    .bind(revision)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    audit(
        &mut tx,
        actor.account_id(),
        "host.admin_delete",
        "host",
        host_id,
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(())
}

pub(crate) async fn sync_record(
    pool: &PgPool,
    actor: &AdminActor,
    account_id: Uuid,
    record_id: &str,
) -> AppResult<()> {
    let Some(record) = SyncRecordId::parse(record_id) else {
        return Err(AppError::NotFound("sync record was not found".to_owned()));
    };
    let mut tx = begin(pool).await?;
    let affected = match record {
        SyncRecordId::Upload(mutation_id) => sqlx::query(
            "UPDATE cloud_sync_push_mutations
             SET admin_deleted_at = now()
             WHERE account_id = $1 AND client_mutation_id = $2
               AND admin_deleted_at IS NULL",
        )
        .bind(account_id)
        .bind(mutation_id)
        .execute(&mut *tx)
        .await
        .map_err(storage)?,
        SyncRecordId::Download(device_id) => sqlx::query(
            "UPDATE cloud_sync_device_checkpoints
             SET admin_deleted_at = now()
             WHERE account_id = $1 AND device_id = $2
               AND admin_deleted_at IS NULL",
        )
        .bind(account_id)
        .bind(device_id)
        .execute(&mut *tx)
        .await
        .map_err(storage)?,
    };
    if affected.rows_affected() != 1 {
        return not_found(tx, "sync record was not found").await;
    }
    audit(
        &mut tx,
        actor.account_id(),
        "sync_record.admin_delete",
        "sync_record",
        record.id(),
    )
    .await?;
    commit(tx).await?;
    mark_semantic_audit_recorded();
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyncRecordId {
    Upload(Uuid),
    Download(Uuid),
}

impl SyncRecordId {
    fn parse(value: &str) -> Option<Self> {
        if let Some(device_id) = value.strip_prefix("download:") {
            return non_nil_uuid(device_id).map(Self::Download);
        }
        non_nil_uuid(value).map(Self::Upload)
    }

    const fn id(self) -> Uuid {
        match self {
            Self::Upload(id) | Self::Download(id) => id,
        }
    }
}

fn non_nil_uuid(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok().filter(|id| !id.is_nil())
}

async fn account_exists(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)")
        .bind(account_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(storage)
}

async fn audit(
    tx: &mut DbTransaction<'_>,
    actor_account_id: Uuid,
    action: &str,
    resource_kind: &str,
    resource_id: Uuid,
) -> AppResult<()> {
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events
             (id, actor_account_id, action, resource_kind, resource_id,
              outcome, request_id, details)
         VALUES ($1,$2,$3,$4,$5,'success',$6,$7)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_account_id)
    .bind(action)
    .bind(resource_kind)
    .bind(resource_id.to_string())
    .bind(request_id)
    .bind(json!({}))
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn not_found(tx: DbTransaction<'_>, message: &str) -> AppResult<()> {
    tx.rollback().await.map_err(storage)?;
    Err(AppError::NotFound(message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::SyncRecordId;
    use uuid::Uuid;

    #[test]
    fn sync_record_ids_are_explicit_and_non_nil() {
        let id = Uuid::now_v7();
        assert_eq!(
            SyncRecordId::parse(&id.to_string()),
            Some(SyncRecordId::Upload(id))
        );
        assert_eq!(
            SyncRecordId::parse(&format!("download:{id}")),
            Some(SyncRecordId::Download(id))
        );
        assert_eq!(SyncRecordId::parse("download:not-a-uuid"), None);
        assert_eq!(SyncRecordId::parse(&Uuid::nil().to_string()), None);
    }
}
