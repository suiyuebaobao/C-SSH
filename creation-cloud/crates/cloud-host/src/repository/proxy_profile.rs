//! 封装代理线路不透明密文的当前行、版本与墓碑写入。

use cloud_domain::AppResult;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use super::{DbTransaction, storage};
use crate::{actor::DeviceActor, validation::ValidatedAiPayload};

#[derive(Clone, Debug, FromRow)]
pub(super) struct ProxyProfileRow {
    pub ciphertext: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub envelope_metadata: Option<Value>,
    pub revision: i64,
    pub is_deleted: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ProxyProfileWriteValue {
    pub ciphertext: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub envelope_metadata: Option<Value>,
    pub deleted: bool,
}

impl ProxyProfileWriteValue {
    pub(super) fn from_payload(payload: &ValidatedAiPayload) -> Self {
        Self {
            ciphertext: Some(payload.ciphertext.clone()),
            nonce: Some(payload.nonce.clone()),
            envelope_metadata: Some(payload.envelope_metadata.clone()),
            deleted: false,
        }
    }

    pub(super) const fn tombstone() -> Self {
        Self {
            ciphertext: None,
            nonce: None,
            envelope_metadata: None,
            deleted: true,
        }
    }
}

pub(super) async fn lock_current(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    resource_id: Uuid,
) -> AppResult<Option<ProxyProfileRow>> {
    sqlx::query_as::<_, ProxyProfileRow>(
        "SELECT ciphertext, nonce, envelope_metadata, revision, is_deleted
         FROM cloud_proxy_profiles WHERE account_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(account_id)
    .bind(resource_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

pub(super) fn same_value(current: &ProxyProfileRow, value: &ProxyProfileWriteValue) -> bool {
    current.ciphertext == value.ciphertext
        && current.nonce == value.nonce
        && current.envelope_metadata == value.envelope_metadata
        && current.is_deleted == value.deleted
}

pub(super) async fn write(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    resource_id: Uuid,
    revision: i64,
    value: ProxyProfileWriteValue,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_proxy_profiles
             (account_id,id,ciphertext,nonce,envelope_metadata,source_device_id,revision,is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         ON CONFLICT (account_id,id) DO UPDATE SET ciphertext=EXCLUDED.ciphertext,
             nonce=EXCLUDED.nonce,envelope_metadata=EXCLUDED.envelope_metadata,
             source_device_id=EXCLUDED.source_device_id,revision=EXCLUDED.revision,
             is_deleted=EXCLUDED.is_deleted,updated_at=now()",
    )
    .bind(actor.account_id())
    .bind(resource_id)
    .bind(&value.ciphertext)
    .bind(&value.nonce)
    .bind(&value.envelope_metadata)
    .bind(actor.device_id())
    .bind(revision)
    .bind(value.deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO cloud_proxy_profile_versions
             (account_id,resource_id,revision,ciphertext,nonce,envelope_metadata,source_device_id,is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    ).bind(actor.account_id()).bind(resource_id).bind(revision).bind(value.ciphertext)
      .bind(value.nonce).bind(value.envelope_metadata).bind(actor.device_id()).bind(value.deleted)
      .execute(&mut **tx).await.map_err(storage)?;
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET minimum_sync_contract_version=GREATEST(minimum_sync_contract_version,3), updated_at=now()
         WHERE account_id=$1",
    )
    .bind(actor.account_id())
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}
