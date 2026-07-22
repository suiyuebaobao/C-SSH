//! 通过当前会话派生账号并返回有界分页的本人下载历史。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{DownloadHistoryItem, Service, repository};

impl Service {
    pub async fn account_history(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<DownloadHistoryItem>> {
        let page = page.normalized();
        let (items, total) =
            repository::account_history::list(&self.pool, session.account_id, page).await?;
        Ok(Page {
            items,
            page: page.page,
            size: page.size,
            total,
        })
    }
}
