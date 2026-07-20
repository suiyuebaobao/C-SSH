//! 再次验证管理员 actor 后执行显式详情读取，正文不进入列表路径。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{AdminFeedbackDetail, Service, authorization, repository, validation};

impl Service {
    pub async fn get_feedback_for_management(
        &self,
        actor: &AdminActor,
        id: Uuid,
    ) -> AppResult<AdminFeedbackDetail> {
        authorization::admin(actor)?;
        let id = validation::id(id)?;
        let row = repository::get::management(&self.pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("反馈不存在".to_owned()))?;
        AdminFeedbackDetail::try_from(row)
    }
}
