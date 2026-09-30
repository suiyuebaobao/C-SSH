//! 保存下载域连接池和只读发布根目录，并提供统一构造入口。

use std::{path::PathBuf, sync::Arc};

use cloud_store::PgPool;
use url::Url;

use crate::{file_verification::FileVerifier, limiter::DownloadLimiter};

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
    pub(crate) download_root: Arc<PathBuf>,
    pub(crate) public_base_url: Url,
    pub(crate) file_verifier: FileVerifier,
    pub(crate) limiter: DownloadLimiter,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, download_root: impl Into<PathBuf>) -> Self {
        Self::with_public_base_url(
            pool,
            download_root,
            Url::parse("https://c-ssh.com/").expect("固定生产地址必须合法"),
        )
    }

    /// 服务启动时传入已由 cloud-config 校验的公开根地址，不从请求头推断。
    #[must_use]
    pub fn with_public_base_url(
        pool: PgPool,
        download_root: impl Into<PathBuf>,
        public_base_url: Url,
    ) -> Self {
        Self {
            pool,
            download_root: Arc::new(download_root.into()),
            public_base_url,
            file_verifier: FileVerifier::default(),
            limiter: DownloadLimiter::default(),
        }
    }
}
