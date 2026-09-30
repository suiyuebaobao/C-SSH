//! 从固定 revision 版本表投影代理线路不透明密文记录。

use super::super::{DbTransaction, storage};
use crate::PullProxyProfileRecord;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
struct VersionRow {
    resource_id: Uuid,
    revision: i64,
    ciphertext: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    envelope_metadata: Option<Value>,
    source_device_id: Uuid,
    is_deleted: bool,
    recorded_at: DateTime<Utc>,
}

pub(super) async fn load(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    resource_id: Uuid,
    revision: i64,
) -> AppResult<PullProxyProfileRecord> {
    let row = sqlx::query_as::<_, VersionRow>(
        "SELECT resource_id,revision,ciphertext,nonce,envelope_metadata,
                source_device_id,is_deleted,recorded_at
         FROM cloud_proxy_profile_versions
         WHERE account_id=$1 AND resource_id=$2 AND revision=$3",
    )
    .bind(account_id)
    .bind(resource_id)
    .bind(revision)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(PullProxyProfileRecord {
        resource_id: row.resource_id,
        revision: row.revision,
        ciphertext: row.ciphertext.map(|value| STANDARD.encode(value)),
        nonce: row.nonce.map(|value| STANDARD.encode(value)),
        envelope_metadata: row.envelope_metadata,
        source_device_id: row.source_device_id,
        deleted: row.is_deleted,
        updated_at: row.recorded_at,
    })
}
