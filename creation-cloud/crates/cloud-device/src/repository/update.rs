//! 重命名当前账号尚未撤销的单个设备。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{model::DeviceRow, Device};

use super::error;

pub(crate) const RENAME_SQL: &str = "UPDATE devices SET name = $3, updated_at = now() \
     WHERE account_id = $1 AND id = ANY($2) AND revoked_at IS NULL";

pub(crate) async fn rename(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
    name: &str,
) -> AppResult<Option<Device>> {
    let mut tx = pool.begin().await.map_err(error::storage)?;
    let owner = sqlx::query_scalar::<_, Uuid>(super::delete::LOCK_ACCOUNT_SQL)
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(error::storage)?;
    if owner.is_none() {
        return Err(AppError::Unauthorized("账号不可用".to_owned()));
    }
    let members = super::group::active_ids(&mut tx, account_id, device_id).await?;
    if members.is_empty() {
        return Ok(None);
    }
    sqlx::query(RENAME_SQL)
        .bind(account_id)
        .bind(&members)
        .bind(name)
        .execute(&mut *tx)
        .await
        .map_err(error::storage)?;
    let row = sqlx::query_as::<_, DeviceRow>(super::get::FIND_SQL)
        .bind(account_id)
        .bind(device_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(error::storage)?;
    tx.commit().await.map_err(error::storage)?;
    Ok(Some(Device::from_row(row)))
}
