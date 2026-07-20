//! 仅删除仍属于可编辑版本的资产。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, authorization, repository, validation};

impl Service {
    pub async fn delete_asset(&self, actor: &AdminActor, id: Uuid) -> AppResult<()> {
        authorization::require(actor)?;
        let id = validation::valid_id(id, "资产标识")?;
        let locator = repository::asset::get::execute(&self.pool, id).await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release =
            repository::release::lock::execute(&mut transaction, locator.release_id).await?;
        let asset = repository::asset::lock::execute(&mut transaction, id).await?;
        if asset.release_id != locator.release_id {
            return Err(AppError::Conflict("资产所属版本已经变化".into()));
        }
        if !release.status.allows_asset_mutation() {
            return Err(AppError::Conflict("已发布版本的资产必须保留".into()));
        }
        repository::asset::delete::execute(&mut transaction, id).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)
    }
}
