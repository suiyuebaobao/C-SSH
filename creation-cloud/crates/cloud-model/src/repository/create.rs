//! 创建模型元数据，并在同一事务内维持默认模型唯一性。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ModelProfile, types::CreateModel};

use super::{ModelRow, model_from_row, storage, write_error};

pub(crate) async fn create(
    pool: &PgPool,
    account_id: Uuid,
    model: CreateModel,
) -> AppResult<ModelProfile> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始模型创建事务"))?;
    if model.is_default {
        sqlx::query("UPDATE model_profiles SET is_default = FALSE WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage("无法更新默认模型"))?;
    }
    let row = sqlx::query_as::<_, ModelRow>(
        "INSERT INTO model_profiles \
         (id, account_id, name, provider, base_url, model_name, context_length, \
          capability_tags, default_parameters, enabled, is_default, sort_order, vault_envelope_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
         ON CONFLICT (account_id, name) DO NOTHING \
         RETURNING id, name, provider, base_url, model_name, context_length, \
          capability_tags, default_parameters, enabled, is_default, sort_order, \
          vault_envelope_id, created_at, updated_at",
    )
    .bind(model.id)
    .bind(account_id)
    .bind(model.name)
    .bind(model.provider)
    .bind(model.base_url)
    .bind(model.model_name)
    .bind(model.context_length)
    .bind(model.capability_tags)
    .bind(model.default_parameters)
    .bind(model.enabled)
    .bind(model.is_default)
    .bind(model.sort_order)
    .bind(model.vault_envelope_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::Conflict("同一账号下的模型名称已存在".to_owned()))?;
    transaction
        .commit()
        .await
        .map_err(storage("无法提交模型创建事务"))?;
    Ok(model_from_row(row))
}
