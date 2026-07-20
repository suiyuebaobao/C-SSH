//! 按账号所有权读取单个模型元数据。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::ModelProfile;

use super::{ModelRow, model_from_row, storage};

pub(crate) async fn get(
    pool: &PgPool,
    account_id: Uuid,
    model_id: Uuid,
) -> AppResult<ModelProfile> {
    let row = sqlx::query_as::<_, ModelRow>(
        "SELECT id, name, provider, base_url, model_name, context_length, \
         capability_tags, default_parameters, enabled, is_default, sort_order, \
         vault_envelope_id, created_at, updated_at \
         FROM model_profiles WHERE account_id = $1 AND id = $2",
    )
    .bind(account_id)
    .bind(model_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法读取模型元数据"))?
    .ok_or_else(|| AppError::NotFound("模型不存在".to_owned()))?;
    Ok(model_from_row(row))
}
