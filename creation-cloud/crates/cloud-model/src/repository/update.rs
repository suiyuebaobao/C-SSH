//! 更新模型元数据，并在同一事务内维持默认模型唯一性。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ModelProfile, types::UpdateModel};

use super::{ModelRow, model_from_row, storage, write_error};

pub(crate) async fn update(
    pool: &PgPool,
    account_id: Uuid,
    model_id: Uuid,
    model: UpdateModel,
) -> AppResult<ModelProfile> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始模型更新事务"))?;
    if model.is_default == Some(true) {
        sqlx::query(
            "UPDATE model_profiles SET is_default = FALSE \
             WHERE account_id = $1 AND id <> $2",
        )
        .bind(account_id)
        .bind(model_id)
        .execute(&mut *transaction)
        .await
        .map_err(storage("无法更新默认模型"))?;
    }

    let base_url_set = model.base_url.is_some();
    let base_url = model.base_url.flatten();
    let vault_set = model.vault_envelope_id.is_some();
    let vault_envelope_id = model.vault_envelope_id.flatten();
    let row = sqlx::query_as::<_, ModelRow>(
        "UPDATE model_profiles SET \
         name = COALESCE($3, name), provider = COALESCE($4, provider), \
         base_url = CASE WHEN $5 THEN $6 ELSE base_url END, \
         model_name = COALESCE($7, model_name), \
         context_length = COALESCE($8, context_length), \
         capability_tags = COALESCE($9, capability_tags), \
         default_parameters = COALESCE($10, default_parameters), \
         enabled = COALESCE($11, enabled), is_default = COALESCE($12, is_default), \
         sort_order = COALESCE($13, sort_order), \
         vault_envelope_id = CASE WHEN $14 THEN $15 ELSE vault_envelope_id END, \
         updated_at = now() WHERE account_id = $1 AND id = $2 \
         RETURNING id, name, provider, base_url, model_name, context_length, \
         capability_tags, default_parameters, enabled, is_default, sort_order, \
         vault_envelope_id, created_at, updated_at",
    )
    .bind(account_id)
    .bind(model_id)
    .bind(model.name)
    .bind(model.provider)
    .bind(base_url_set)
    .bind(base_url)
    .bind(model.model_name)
    .bind(model.context_length)
    .bind(model.capability_tags)
    .bind(model.default_parameters)
    .bind(model.enabled)
    .bind(model.is_default)
    .bind(model.sort_order)
    .bind(vault_set)
    .bind(vault_envelope_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::NotFound("模型不存在".to_owned()))?;
    transaction
        .commit()
        .await
        .map_err(storage("无法提交模型更新事务"))?;
    Ok(model_from_row(row))
}
