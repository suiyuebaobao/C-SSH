//! 向管理员返回一条站点媒体记录，不暴露额外文件系统操作。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{Service, SiteMedia, repository, validation};

impl Service {
    pub async fn get(&self, _actor: &AdminActor, media_id: Uuid) -> AppResult<SiteMedia> {
        repository::get::execute(self.pool(), validation::valid_id(media_id)?).await
    }
}
