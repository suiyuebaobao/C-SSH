//! epoch0保护迁移的只读receipt；只返回可核对摘要与结果坐标。

use std::fmt::Write as _;

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{DataProtectionMigrationReceipt, ResourceKind, ResourceRevision, actor::DeviceActor};

use super::super::{
    DbTransaction, begin, commit, lock_existing_sync_state, require_active_device, storage,
};

#[derive(FromRow)]
struct MigrationRow {
    source_device_id: Uuid,
    request_generation: i64,
    request_epoch: i64,
    request_revision: i64,
    request_current_revision: i64,
    request_hash: Vec<u8>,
    request_hash_scheme: String,
    result_generation: i64,
    result_epoch: i64,
    result_revision: i64,
    result_current_revision: i64,
    changed_count: i32,
    created_at: DateTime<Utc>,
}

pub(crate) async fn get(
    pool: &PgPool,
    actor: DeviceActor,
    mutation_id: Uuid,
) -> AppResult<DataProtectionMigrationReceipt> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let _state = lock_existing_sync_state(&mut tx, actor.account_id()).await?;
    let row = sqlx::query_as::<_, MigrationRow>(
        "SELECT source_device_id,request_generation,request_epoch,request_revision,
                request_current_revision,request_hash,request_hash_scheme,
                result_generation,result_epoch,
                result_revision,result_current_revision,changed_count,created_at
         FROM cloud_data_protection_mutations
         WHERE account_id=$1 AND mutation_id=$2 AND operation='migrate'",
    )
    .bind(actor.account_id())
    .bind(mutation_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or_else(|| AppError::NotFound("protection migration receipt不存在".to_owned()))?;
    if row.source_device_id != actor.device_id() {
        return Err(AppError::NotFound(
            "protection migration receipt不存在".to_owned(),
        ));
    }
    if row.request_hash.len() != 32
        || !matches!(
            row.request_hash_scheme.as_str(),
            "data_protection_server_struct_v1" | "canonical_json_v1"
        )
    {
        return Err(super::super::invalid_stored_value());
    }
    let revisions = load_results(&mut tx, actor.account_id(), mutation_id).await?;
    if usize::try_from(row.changed_count).ok() != Some(revisions.len()) {
        return Err(super::super::invalid_stored_value());
    }
    commit(tx).await?;
    Ok(DataProtectionMigrationReceipt {
        status: "migrated".to_owned(),
        mutation_id,
        source_device_id: row.source_device_id,
        request_hash: lower_hex(&row.request_hash),
        request_hash_scheme: row.request_hash_scheme,
        source_sync_generation: row.request_generation,
        source_protection_epoch: row.request_epoch,
        source_protection_revision: row.request_revision,
        source_current_revision: row.request_current_revision,
        result_sync_generation: row.result_generation,
        result_protection_epoch: row.result_epoch,
        result_protection_revision: row.result_revision,
        result_current_revision: row.result_current_revision,
        changed_count: row.changed_count,
        revisions,
        completed_at: row.created_at,
    })
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

fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
