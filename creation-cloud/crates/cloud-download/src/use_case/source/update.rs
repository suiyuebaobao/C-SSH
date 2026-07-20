//! 只允许调整来源排序和启停状态。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{ReleaseSource, Service, UpdateSourceInput, authorization, repository, validation};

impl Service {
    pub async fn update_source(
        &self,
        actor: &AdminActor,
        source_id: Uuid,
        input: UpdateSourceInput,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        let source_id = validation::valid_id(source_id, "来源标识")?;
        let input = normalize(input)?;
        let source = repository::source::get::execute(&self.pool, source_id).await?;
        let asset = repository::asset::get(&self.pool, source.asset_id).await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        repository::release_lock::execute(&mut transaction, asset.release_id).await?;
        let locked_asset =
            repository::asset_lock::execute(&mut transaction, source.asset_id).await?;
        if locked_asset.release_id != asset.release_id {
            return Err(AppError::Conflict("资产所属版本已经变化".into()));
        }
        let locked_source = repository::source::lock::execute(&mut transaction, source_id).await?;
        if locked_source.asset_id != source.asset_id {
            return Err(AppError::Conflict("来源所属资产已经变化".into()));
        }
        let updated =
            repository::source::update::execute(&mut transaction, source_id, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(updated)
    }
}

pub(crate) fn normalize(input: UpdateSourceInput) -> AppResult<UpdateSourceInput> {
    if input.sort_order.is_none() && input.enabled.is_none() {
        return Err(AppError::Validation("来源更新内容不能为空".into()));
    }
    Ok(UpdateSourceInput {
        sort_order: input.sort_order.map(validation::sort_order).transpose()?,
        enabled: input.enabled,
    })
}
