//! 仅允许删除未进入校验流程的草稿版本。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{ReleaseStatus, Service, authorization, repository, validation};

impl Service {
    pub async fn delete_release(&self, actor: &AdminActor, id: Uuid) -> AppResult<()> {
        authorization::require(actor)?;
        let id = validation::valid_id(id, "版本标识")?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let current = repository::release::lock::execute(&mut transaction, id).await?;
        if current.status != ReleaseStatus::Draft {
            return Err(AppError::Conflict("只有草稿版本可以删除".into()));
        }
        repository::release::delete::execute(&mut transaction, id).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)
    }
}
