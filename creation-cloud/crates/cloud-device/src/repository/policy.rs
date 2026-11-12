//! 账号设备限额覆盖的独立存储；策略锁、账号锁、CAS与语义审计保持同一事务。
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::error;
use crate::{AccountDevicePolicy, DeviceQuotaResolver};

async fn snapshot(
    tx: &mut Transaction<'_, Postgres>,
    account: Uuid,
    global: i64,
) -> AppResult<AccountDevicePolicy> {
    let row = sqlx::query_as::<_, (Option<i32>, i64)>(
        "SELECT maximum, revision FROM account_device_policies WHERE account_id=$1",
    )
    .bind(account)
    .fetch_optional(&mut **tx)
    .await
    .map_err(error::storage)?;
    let active = sqlx::query_scalar::<_, i64>(super::create::COUNT_ACTIVE_DEVICES_SQL)
        .bind(account)
        .fetch_one(&mut **tx)
        .await
        .map_err(error::storage)?;
    let (maximum, revision) = row.unwrap_or((None, 0));
    Ok(AccountDevicePolicy {
        maximum,
        global_maximum: global,
        effective_maximum: maximum.map_or(global, i64::from),
        active_devices: active,
        revision,
    })
}

pub(crate) async fn read(
    pool: &PgPool,
    account: Uuid,
    quota: DeviceQuotaResolver,
) -> AppResult<AccountDevicePolicy> {
    let mut tx = pool.begin().await.map_err(error::storage)?;
    let global = quota(&mut tx).await?;
    let exists = sqlx::query_scalar::<_, Uuid>("SELECT id FROM accounts WHERE id=$1 FOR KEY SHARE")
        .bind(account)
        .fetch_optional(&mut *tx)
        .await
        .map_err(error::storage)?;
    if exists.is_none() {
        return Err(AppError::NotFound("账号不存在".to_owned()));
    }
    let result = snapshot(&mut tx, account, global).await?;
    tx.commit().await.map_err(error::storage)?;
    Ok(result)
}

pub(crate) async fn update(
    pool: &PgPool,
    actor: Uuid,
    account: Uuid,
    maximum: Option<i32>,
    expected: i64,
    quota: DeviceQuotaResolver,
) -> AppResult<AccountDevicePolicy> {
    let mut tx = pool.begin().await.map_err(error::storage)?;
    let global = quota(&mut tx).await?;
    let exists = sqlx::query_scalar::<_, Uuid>("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(account)
        .fetch_optional(&mut *tx)
        .await
        .map_err(error::storage)?;
    if exists.is_none() {
        return Err(AppError::NotFound("账号不存在".to_owned()));
    }
    let current = snapshot(&mut tx, account, global).await?;
    if current.revision != expected {
        return Err(AppError::Conflict(
            "设备限额已变化，请刷新后重试".to_owned(),
        ));
    }
    if current.maximum == maximum {
        return Err(AppError::Conflict("设备限额没有变化".to_owned()));
    }
    sqlx::query(
        "INSERT INTO account_device_policies(account_id,maximum,revision,updated_by) VALUES($1,$2,1,$3) \
         ON CONFLICT(account_id) DO UPDATE SET maximum=EXCLUDED.maximum, \
         revision=account_device_policies.revision+1, updated_by=EXCLUDED.updated_by, updated_at=now()",
    ).bind(account).bind(maximum).bind(actor).execute(&mut *tx).await.map_err(error::storage)?;
    let result = snapshot(&mut tx, account, global).await?;
    let request = cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events(id,actor_account_id,action,resource_kind,resource_id,outcome,request_id,details) \
         VALUES($1,$2,'device_policy.updated','account_device_policy',$3,'success',$4,$5)",
    ).bind(Uuid::now_v7()).bind(actor).bind(account.to_string()).bind(request)
     .bind(serde_json::json!({"maximum":result.maximum,"effective_maximum":result.effective_maximum,"revision":result.revision}))
     .execute(&mut *tx).await.map_err(error::storage)?;
    tx.commit().await.map_err(error::storage)?;
    cloud_domain::mark_semantic_audit_recorded();
    Ok(result)
}
