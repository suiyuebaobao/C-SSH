//! 只投影当前已发布首页二维码的必要公开元数据。

use cloud_domain::AppResult;

use crate::{PublicHomeQr, Service, repository};

impl Service {
    pub async fn current_home_qr(&self) -> AppResult<PublicHomeQr> {
        PublicHomeQr::try_from(repository::public::current::execute(self.pool()).await?)
    }
}
