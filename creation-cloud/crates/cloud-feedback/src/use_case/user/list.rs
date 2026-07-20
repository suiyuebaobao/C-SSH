//! 仅在当前账号范围内分页返回本人反馈，不提供跨账号筛选参数。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{FeedbackSubmission, Service, authorization, repository, validation};

impl Service {
    pub async fn list_own_feedback(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<FeedbackSubmission>> {
        let account_id = authorization::user(session)?;
        let page = validation::bounded_page(page);
        let (rows, total) = repository::list::owned(&self.pool, account_id, page).await?;
        let items = rows
            .into_iter()
            .map(FeedbackSubmission::try_from)
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Page {
            items,
            page: page.page,
            size: page.size,
            total,
        })
    }
}
