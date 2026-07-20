//! 校验本人反馈并携带当前请求标识进入事务限流创建路径。

use cloud_domain::{AppResult, AuthenticatedSession, current_request_id};
use uuid::Uuid;

use crate::{
    CreateFeedbackInput, FeedbackSubmission, Service, authorization, repository, validation,
};

impl Service {
    pub async fn create_feedback(
        &self,
        session: &AuthenticatedSession,
        input: CreateFeedbackInput,
    ) -> AppResult<FeedbackSubmission> {
        let account_id = authorization::user(session)?;
        let input = validation::create(input)?;
        let request_id = current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
        repository::create::execute(&self.pool, Uuid::now_v7(), account_id, &request_id, &input)
            .await
    }
}
