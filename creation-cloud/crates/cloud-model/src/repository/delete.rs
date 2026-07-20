//! 按账号所有权删除单个模型元数据。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use super::storage;

pub(crate) async fn delete(pool: &PgPool, account_id: Uuid, model_id: Uuid) -> AppResult<()> {
    let deleted = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM model_profiles WHERE account_id = $1 AND id = $2 RETURNING id",
    )
    .bind(account_id)
    .bind(model_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法删除模型元数据"))?;
    if deleted.is_none() {
        return Err(AppError::NotFound("模型不存在".to_owned()));
    }
    Ok(())
}
