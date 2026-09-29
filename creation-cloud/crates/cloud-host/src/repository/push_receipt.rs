//! 同设备旧push的只读receipt查询；不读取或返回原请求正文。

use std::fmt::Write as _;

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{PushReceipt, actor::DeviceActor};

use super::{
    DbTransaction, begin, commit, lock_existing_sync_state, push::load_results,
    require_active_device, storage,
};

#[derive(FromRow)]
struct PushReceiptRow {
    source_device_id: Uuid,
    request_generation: i64,
    request_protection_epoch: i64,
    request_protection_revision: i64,
    request_hash: Vec<u8>,
    request_hash_scheme: String,
    outcome: String,
    result_revision: i64,
    changed_count: i32,
    created_at: DateTime<Utc>,
}

pub(crate) async fn get(
    pool: &PgPool,
    actor: DeviceActor,
    mutation_id: Uuid,
) -> AppResult<PushReceipt> {
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    // 与 push 共用账号状态行锁。查询返回 404 前必须等待所有已取得该锁的旧写结束；
    // 配合 0056 的数据库触发器，404 才能证明该旧请求没有、也不会迟到写入明文。
    let _state = lock_existing_sync_state(&mut tx, actor.account_id()).await?;
    let row = load(&mut tx, actor.account_id(), mutation_id)
        .await?
        .ok_or_else(|| AppError::NotFound("push receipt不存在".to_owned()))?;
    if row.source_device_id != actor.device_id() {
        return Err(AppError::NotFound("push receipt不存在".to_owned()));
    }
    if row.request_hash.len() != 32 || !matches!(row.outcome.as_str(), "applied" | "unchanged") {
        return Err(super::invalid_stored_value());
    }
    let revisions = load_results(&mut tx, actor.account_id(), mutation_id).await?;
    commit(tx).await?;
    Ok(PushReceipt {
        status: row.outcome,
        mutation_id,
        source_device_id: row.source_device_id,
        request_sync_generation: row.request_generation,
        request_protection_epoch: row.request_protection_epoch,
        request_protection_revision: row.request_protection_revision,
        result_revision: row.result_revision,
        changed_count: row.changed_count,
        request_hash: lower_hex(&row.request_hash),
        request_hash_scheme: row.request_hash_scheme,
        revisions,
        created_at: row.created_at,
    })
}

async fn load(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    mutation_id: Uuid,
) -> AppResult<Option<PushReceiptRow>> {
    sqlx::query_as(
        "SELECT source_device_id,request_generation,request_protection_epoch,
                request_protection_revision,request_hash,request_hash_scheme,outcome,
                result_revision,changed_count,created_at
         FROM cloud_sync_push_receipts
         WHERE account_id=$1 AND client_mutation_id=$2",
    )
    .bind(account_id)
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
