//! 保存不含密文正文的 rekey 语义审计。

use super::super::DbTransaction;
use crate::actor::DeviceActor;
use cloud_domain::{AppError, AppResult, current_request_id};
use serde_json::json;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub(super) async fn record(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    mutation_id: Uuid,
    previous_generation: i64,
    next_generation: i64,
    result_revision: i64,
    changed_hosts: usize,
    changed_ai: usize,
    changed_proxy_profiles: usize,
) -> AppResult<()> {
    let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "mutation_id": mutation_id,
        "device_id": actor.device_id(),
        "changed_hosts": i64::try_from(changed_hosts).unwrap_or(i64::MAX),
        "changed_ai_providers": i64::try_from(changed_ai).unwrap_or(i64::MAX),
        "changed_proxy_profiles": i64::try_from(changed_proxy_profiles).unwrap_or(i64::MAX),
        "result_revision": result_revision,
        "previous_sync_generation": previous_generation,
        "sync_generation": next_generation
    });
    sqlx::query(
        "INSERT INTO audit_events
             (id,actor_account_id,action,resource_kind,resource_id,outcome,request_id,details)
         VALUES ($1,$2,'sync.encrypted_data_rekey_v2','sync_account',$3,'success',$4,$5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(actor.account_id().to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| AppError::Storage("failed to persist encrypted sync rekey audit".to_owned()))?;
    Ok(())
}
