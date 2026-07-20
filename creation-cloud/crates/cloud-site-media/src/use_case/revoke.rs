//! 将已发布首页二维码单向撤销，禁止草稿和历史撤销记录反向迁移。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, SiteMedia, SiteMediaState, repository, validation};

impl Service {
    pub async fn revoke(&self, _actor: &AdminActor, media_id: Uuid) -> AppResult<SiteMedia> {
        let media_id = validation::valid_id(media_id)?;
        let media = repository::get::execute(self.pool(), media_id).await?;
        ensure_revocable(media.state)?;
        repository::revoke::execute(self.pool(), media_id).await
    }
}

pub(super) fn ensure_revocable(state: SiteMediaState) -> AppResult<()> {
    if state != SiteMediaState::Published {
        return Err(AppError::Conflict("只有已发布站点媒体可以撤销".into()));
    }
    Ok(())
}
