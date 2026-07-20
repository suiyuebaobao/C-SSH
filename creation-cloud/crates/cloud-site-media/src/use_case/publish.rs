//! 串行化首页二维码槽位发布，并在单一事务内撤销旧版本后发布目标草稿。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, SiteMedia, SiteMediaSlot, SiteMediaState, repository, validation};

impl Service {
    pub async fn publish(&self, _actor: &AdminActor, media_id: Uuid) -> AppResult<SiteMedia> {
        let media_id = validation::valid_id(media_id)?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| AppError::Storage("无法开始站点媒体发布事务".into()))?;
        repository::publish::lock_slot(&mut transaction, SiteMediaSlot::HomeQr).await?;
        let media = repository::get::lock(&mut transaction, media_id).await?;
        ensure_publishable(media.state)?;
        repository::publish::revoke_current(&mut transaction, media.slot, media_id).await?;
        let media = repository::publish::execute(&mut transaction, media_id).await?;
        transaction
            .commit()
            .await
            .map_err(|_| AppError::Storage("提交站点媒体发布事务失败".into()))?;
        Ok(media)
    }
}

pub(super) fn ensure_publishable(state: SiteMediaState) -> AppResult<()> {
    if state != SiteMediaState::Draft {
        return Err(AppError::Conflict("只有草稿站点媒体可以发布".into()));
    }
    Ok(())
}
