//! 保存下载域连接池和只读发布根目录，并提供统一构造入口。

use std::{path::PathBuf, sync::Arc};

use cloud_store::PgPool;

use crate::{file_verification::FileVerifier, limiter::DownloadLimiter};

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
    pub(crate) download_root: Arc<PathBuf>,
    pub(crate) file_verifier: FileVerifier,
    pub(crate) limiter: DownloadLimiter,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, download_root: impl Into<PathBuf>) -> Self {
        Self {
            pool,
            download_root: Arc::new(download_root.into()),
            file_verifier: FileVerifier::default(),
            limiter: DownloadLimiter::default(),
        }
    }
}
