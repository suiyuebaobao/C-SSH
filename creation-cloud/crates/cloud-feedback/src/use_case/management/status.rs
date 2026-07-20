//! 在管理员二次授权、状态机和期望版本校验后原子推进反馈状态。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminFeedbackDetail, FeedbackStatus, Service, UpdateFeedbackStatusInput, authorization,
    repository, validation,
};

impl Service {
    pub async fn update_feedback_status(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateFeedbackStatusInput,
    ) -> AppResult<AdminFeedbackDetail> {
        authorization::admin(actor)?;
        let id = validation::id(id)?;
        let input = validation::status(input)?;
        let current = repository::get::management(&self.pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("反馈不存在".to_owned()))?;
        let current_status = FeedbackStatus::try_from(current.status.as_str())?;
        if current.version != input.expected_version {
            return Err(AppError::Conflict("反馈已被其他管理员更新".to_owned()));
        }
        if !valid_transition(current_status, input.status) {
            return Err(AppError::Conflict("反馈状态迁移不合法".to_owned()));
        }
        let updated =
            repository::update::status(&self.pool, id, input.expected_version, input.status)
                .await?
                .ok_or_else(|| AppError::Conflict("反馈已被其他管理员更新".to_owned()))?;
        AdminFeedbackDetail::try_from(updated)
    }
}

pub(crate) const fn valid_transition(from: FeedbackStatus, to: FeedbackStatus) -> bool {
    matches!(
        (from, to),
        (
            FeedbackStatus::New,
            FeedbackStatus::Triaged | FeedbackStatus::Closed
        ) | (
            FeedbackStatus::Triaged,
            FeedbackStatus::InProgress | FeedbackStatus::Closed
        ) | (
            FeedbackStatus::InProgress,
            FeedbackStatus::Resolved | FeedbackStatus::Closed
        ) | (
            FeedbackStatus::Resolved,
            FeedbackStatus::InProgress | FeedbackStatus::Closed
        )
    )
}
