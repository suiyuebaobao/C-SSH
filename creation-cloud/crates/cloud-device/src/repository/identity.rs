//! 当前认证实例的身份补登记事务；同值幂等，已绑定身份不覆盖，审计不复制原始标识。
use super::error;
use crate::{model::DeviceRow, Device};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use uuid::Uuid;

pub(crate) async fn bind(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device: Uuid,
    fingerprint: &str,
) -> AppResult<Device> {
    let mut tx = pool.begin().await.map_err(error::storage)?;
    let credential = sqlx::query_scalar::<_, i64>(
        "SELECT credential_version FROM accounts WHERE id=$1 AND status='active' FOR UPDATE",
    )
    .bind(session.account_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(error::storage)?
    .ok_or_else(|| AppError::Unauthorized("账号不可用".to_owned()))?;
    let valid = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=$1 AND account_id=$2 AND device_id=$3 \
         AND credential_version=$4 AND revoked_at IS NULL AND expires_at>now() AND absolute_expires_at>now())",
    ).bind(session.session_id).bind(session.account_id).bind(device).bind(credential)
     .fetch_one(&mut *tx).await.map_err(error::storage)?;
    if !valid {
        return Err(AppError::Unauthorized("设备会话无效或已过期".to_owned()));
    }
    let row = sqlx::query_as::<_, DeviceRow>(
        "SELECT id,account_id,name,platform,public_id,last_seen_at,revoked_at,created_at,updated_at,physical_device_fingerprint \
         FROM devices WHERE account_id=$1 AND id=$2 AND revoked_at IS NULL FOR UPDATE",
    ).bind(session.account_id).bind(device).fetch_optional(&mut *tx).await.map_err(error::storage)?
     .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))?;
    if let Some(existing) = row.9.as_deref() {
        if existing != fingerprint {
            return Err(AppError::Conflict(
                "当前安装实例的设备身份已变化".to_owned(),
            ));
        }
        tx.commit().await.map_err(error::storage)?;
        return Ok(Device::from_row(row));
    }
    let row = sqlx::query_as::<_, DeviceRow>(
        "UPDATE devices SET physical_device_fingerprint=$3,updated_at=now() WHERE account_id=$1 AND id=$2 \
         RETURNING id,account_id,name,platform,public_id,last_seen_at,revoked_at,created_at,updated_at,physical_device_fingerprint",
    ).bind(session.account_id).bind(device).bind(fingerprint).fetch_one(&mut *tx).await.map_err(error::storage)?;
    let request = cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events(id,actor_account_id,action,resource_kind,resource_id,outcome,request_id,details) \
         VALUES($1,$2,'device.identity_bound','device',$3,'success',$4,'{\"identity_version\":1}'::jsonb)",
    ).bind(Uuid::now_v7()).bind(session.account_id).bind(device.to_string()).bind(request)
     .execute(&mut *tx).await.map_err(error::storage)?;
    tx.commit().await.map_err(error::storage)?;
    cloud_domain::mark_semantic_audit_recorded();
    Ok(Device::from_row(row))
}
