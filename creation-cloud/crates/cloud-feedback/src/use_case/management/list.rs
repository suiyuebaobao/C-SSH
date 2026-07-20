//! 再次验证管理员 actor 后，仅查询不含标题、正文或邮箱的分页摘要。

use cloud_domain::{AdminActor, AppResult, Page};

use crate::{
    AdminFeedbackListQuery, AdminFeedbackSummary, Service, authorization, repository, validation,
};

impl Service {
    pub async fn list_feedback_for_management(
        &self,
        actor: &AdminActor,
        query: AdminFeedbackListQuery,
    ) -> AppResult<Page<AdminFeedbackSummary>> {
        authorization::admin(actor)?;
        let query = validation::management_query(query);
        let (rows, total) =
            repository::list::management(&self.pool, query.page, query.status).await?;
        let items = rows
            .into_iter()
            .map(AdminFeedbackSummary::try_from)
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Page {
            items,
            page: query.page.page,
            size: query.page.size,
            total,
        })
    }
}
