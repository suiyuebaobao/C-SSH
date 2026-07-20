//! 保存站点媒体领域所需的 PostgreSQL 连接池和受控文件根目录。

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use cloud_store::PgPool;

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
    pub(crate) site_media_root: Arc<PathBuf>,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, site_media_root: impl Into<PathBuf>) -> Self {
        Self {
            pool,
            site_media_root: Arc::new(site_media_root.into()),
        }
    }

    pub(crate) const fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn site_media_root(&self) -> &Path {
        self.site_media_root.as_path()
    }
}
