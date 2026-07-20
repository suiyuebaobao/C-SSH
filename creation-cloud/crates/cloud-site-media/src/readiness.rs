//! 复用真实站点媒体隔离与对象布局验证写入、原子落位及清理能力。

use cloud_domain::AppResult;

use crate::{Service, storage};

impl Service {
    pub async fn ready(&self) -> AppResult<()> {
        storage::readiness_probe(self.site_media_root()).await
    }
}
