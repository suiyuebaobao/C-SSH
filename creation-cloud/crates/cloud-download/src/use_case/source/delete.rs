//! 只允许删除未发布版本的来源，已发布来源通过停用保留历史。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, SourceKind, authorization, repository, stored_file_delete, validation};

impl Service {
    pub async fn delete_source(&self, actor: &AdminActor, source_id: Uuid) -> AppResult<()> {
        authorization::require(actor)?;
        let source_id = validation::valid_id(source_id, "来源标识")?;
        let source = repository::source::get::execute(&self.pool, source_id).await?;
        let asset = repository::asset::get(&self.pool, source.asset_id).await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release_status =
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
        if !matches!(release_status.as_str(), "draft" | "validating") {
            return Err(AppError::Conflict(
                "已发布来源必须停用并保留，不能删除".into(),
            ));
        }

        let mut isolated = if locked_source.source_kind == SourceKind::Local {
            let local_path = locked_source
                .local_path
                .as_deref()
                .ok_or_else(|| AppError::Internal("本站来源缺少对象键".into()))?;
            Some(
                stored_file_delete::QuarantinedObject::isolate(
                    self.download_root.as_path(),
                    local_path,
                )
                .await?,
            )
        } else {
            None
        };

        if let Err(error) = repository::source::delete::execute(&mut transaction, source_id).await {
            let rollback_failed = transaction.rollback().await.is_err();
            let restore_failed = isolated
                .as_mut()
                .is_some_and(|object| object.restore().is_err());
            if rollback_failed || restore_failed {
                return Err(AppError::Storage(
                    "删除来源失败且无法完整恢复本站文件".into(),
                ));
            }
            return Err(error);
        }
        if let Err(error) = transaction.commit().await {
            let restore_failed = isolated
                .as_mut()
                .is_some_and(|object| object.restore().is_err());
            if restore_failed {
                return Err(AppError::Storage(
                    "提交来源删除失败且无法恢复本站文件".into(),
                ));
            }
            return Err(repository::map_transaction_error(error));
        }
        if let Some(object) = isolated {
            object.finish()?;
        }
        Ok(())
    }
}
