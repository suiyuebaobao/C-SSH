//! 在同一数据库事务边界内隔离草稿文件并删除记录，失败时恢复原路径。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, SiteMediaState, finalization, repository, storage, validation};

impl Service {
    pub async fn delete(&self, _actor: &AdminActor, media_id: Uuid) -> AppResult<()> {
        let media_id = validation::valid_id(media_id)?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| AppError::Storage("无法开始站点媒体删除事务".into()))?;
        let media = repository::get::lock(&mut transaction, media_id).await?;
        ensure_deletable(media.state)?;
        let mut deleted =
            storage::quarantine_delete(self.site_media_root(), &media.storage_key).await?;

        if let Err(error) = repository::delete::execute(&mut transaction, media_id).await {
            let restore_result = deleted.restore().await;
            let _ = transaction.rollback().await;
            restore_result?;
            return Err(error);
        }
        finalization::delete(self.pool().clone(), transaction, deleted, media).await
    }
}

pub(super) fn ensure_deletable(state: SiteMediaState) -> AppResult<()> {
    if state != SiteMediaState::Draft {
        return Err(AppError::Conflict("只有草稿站点媒体可以删除".into()));
    }
    Ok(())
}
