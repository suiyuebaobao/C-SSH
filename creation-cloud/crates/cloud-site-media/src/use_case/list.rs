//! 向管理员返回有界的首页二维码历史，不跨站点媒体领域读取其它数据。

use cloud_domain::{AdminActor, AppResult};

use crate::{Service, SiteMedia, repository, validation};

impl Service {
    pub async fn list(&self, _actor: &AdminActor, limit: Option<u32>) -> AppResult<Vec<SiteMedia>> {
        repository::list::execute(self.pool(), validation::list_limit(limit)).await
    }
}
