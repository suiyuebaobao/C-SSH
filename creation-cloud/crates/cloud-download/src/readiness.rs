//! 复用真实上传布局验证下载隔离目录与对象目录可写、可原子落位并可清理。

use cloud_domain::AppResult;

use crate::{Service, upload_file};

impl Service {
    pub async fn ready(&self) -> AppResult<()> {
        upload_file::readiness_probe(self.download_root.as_path()).await
    }
}
