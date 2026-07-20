//! 以当前账号作为强制所有权条件读取单条反馈。

use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{FeedbackSubmission, Service, authorization, repository, validation};

impl Service {
    pub async fn get_own_feedback(
        &self,
        session: &AuthenticatedSession,
        id: Uuid,
    ) -> AppResult<FeedbackSubmission> {
        let account_id = authorization::user(session)?;
        let id = validation::id(id)?;
        let row = repository::get::owned(&self.pool, account_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound("反馈不存在".to_owned()))?;
        FeedbackSubmission::try_from(row)
    }
}
