//! Host metadata迁移的幂等receipt、结果映射与无正文审计持久化。

use std::fmt::Write as _;

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, current_request_id};
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{HostMetadataMigrationReceipt, HostMetadataMigrationRequest, actor::DeviceActor};

use super::super::{DbTransaction, SyncState, storage};

#[derive(FromRow)]
pub(super) struct ReceiptRow {
    mutation_id: Uuid,
    source_device_id: Uuid,
    request_generation: i64,
    request_protection_epoch: i64,
    request_protection_revision: i64,
    request_snapshot_revision: i64,
    result_generation: i64,
    result_protection_epoch: i64,
    result_protection_revision: i64,
    result_current_revision: i64,
    request_hash: Vec<u8>,
    host_count: i32,
    completed_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn persist_receipt(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    request: &HostMetadataMigrationRequest,
    request_hash: [u8; 32],
    result_generation: i64,
    result_protection_revision: i64,
    result_current_revision: i64,
    host_count: i32,
    results: &[(Uuid, i64, i64)],
) -> AppResult<ReceiptRow> {
    let completed_at = Utc::now();
    sqlx::query(
        "INSERT INTO cloud_host_metadata_migrations
             (account_id,mutation_id,source_device_id,request_generation,
              request_protection_epoch,request_protection_revision,
              request_snapshot_revision,result_generation,result_protection_epoch,
              result_protection_revision,result_current_revision,request_hash,
              host_count,completed_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
    )
    .bind(actor.account_id())
    .bind(request.mutation_id)
    .bind(actor.device_id())
    .bind(request.sync_generation)
    .bind(request.protection_epoch)
    .bind(request.protection_revision)
    .bind(request.snapshot_revision)
    .bind(result_generation)
    .bind(request.protection_epoch)
    .bind(result_protection_revision)
    .bind(result_current_revision)
    .bind(request_hash.as_slice())
    .bind(host_count)
    .bind(completed_at)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    for (host_id, previous_revision, result_revision) in results {
        sqlx::query(
            "INSERT INTO cloud_host_metadata_migration_results
                 (account_id,mutation_id,host_id,previous_revision,result_revision)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(actor.account_id())
        .bind(request.mutation_id)
        .bind(host_id)
        .bind(previous_revision)
        .bind(result_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    }
    Ok(ReceiptRow {
        mutation_id: request.mutation_id,
        source_device_id: actor.device_id(),
        request_generation: request.sync_generation,
        request_protection_epoch: request.protection_epoch,
        request_protection_revision: request.protection_revision,
        request_snapshot_revision: request.snapshot_revision,
        result_generation,
        result_protection_epoch: request.protection_epoch,
        result_protection_revision,
        result_current_revision,
        request_hash: request_hash.to_vec(),
        host_count,
        completed_at,
    })
}

pub(super) async fn load_receipt(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Option<ReceiptRow>> {
    sqlx::query_as(
        "SELECT mutation_id,source_device_id,request_generation,
                request_protection_epoch,request_protection_revision,
                request_snapshot_revision,result_generation,result_protection_epoch,
                result_protection_revision,result_current_revision,request_hash,
                host_count,completed_at
         FROM cloud_host_metadata_migrations
         WHERE account_id=$1 AND mutation_id=$2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

pub(super) fn validate_replay(
    row: &ReceiptRow,
    actor: DeviceActor,
    hash: &[u8; 32],
) -> AppResult<()> {
    if row.source_device_id == actor.device_id() && row.request_hash.as_slice() == hash {
        Ok(())
    } else {
        Err(AppError::Conflict(
            "mutation_id已由不同metadata migration请求使用".to_owned(),
        ))
    }
}

pub(super) fn receipt(
    row: ReceiptRow,
    idempotent: bool,
) -> AppResult<HostMetadataMigrationReceipt> {
    if row.request_hash.len() != 32 {
        return Err(super::super::invalid_stored_value());
    }
    Ok(HostMetadataMigrationReceipt {
        status: "migrated".to_owned(),
        mutation_id: row.mutation_id,
        source_device_id: row.source_device_id,
        request_hash: lower_hex(&row.request_hash),
        source_sync_generation: row.request_generation,
        source_protection_epoch: row.request_protection_epoch,
        source_protection_revision: row.request_protection_revision,
        source_snapshot_revision: row.request_snapshot_revision,
        result_sync_generation: row.result_generation,
        result_protection_epoch: row.result_protection_epoch,
        result_protection_revision: row.result_protection_revision,
        result_current_revision: row.result_current_revision,
        host_count: row.host_count,
        completed_at: row.completed_at,
        idempotent,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn audit(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    source: SyncState,
    result_generation: i64,
    result_protection_revision: i64,
    result_current_revision: i64,
    host_count: i32,
) -> AppResult<()> {
    let details = json!({
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "previous_sync_generation": source.sync_generation,
        "sync_generation": result_generation,
        "protection_epoch": source.protection_epoch,
        "previous_protection_revision": source.protection_revision,
        "protection_revision": result_protection_revision,
        "previous_current_revision": source.current_revision,
        "current_revision": result_current_revision,
        "host_count": host_count,
    });
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events
             (id,actor_account_id,action,resource_kind,resource_id,
              outcome,request_id,details)
         VALUES ($1,$2,'sync.host_metadata_migrate_v1','sync_account',$3,
                 'success',$4,$5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(actor.account_id().to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
