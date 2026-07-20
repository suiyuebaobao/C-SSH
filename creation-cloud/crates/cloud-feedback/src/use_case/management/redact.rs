//! 将可能含敏感内容的反馈替换为固定墓碑并关闭，操作不可恢复或物理删除。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminFeedbackDetail, RedactFeedbackInput, Service, authorization, repository, validation,
};

impl Service {
    pub async fn redact_feedback(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: RedactFeedbackInput,
    ) -> AppResult<AdminFeedbackDetail> {
        let actor_id = authorization::admin(actor)?;
        let id = validation::id(id)?;
        let input = validation::redaction(input)?;
        let current = repository::get::management(&self.pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("反馈不存在".to_owned()))?;
        if current.redacted_at.is_some() {
            return Err(AppError::Conflict("反馈已经完成安全脱敏".to_owned()));
        }
        if current.version != input.expected_version {
            return Err(AppError::Conflict("反馈已被其他管理员更新".to_owned()));
        }
        let updated = repository::update::redact(
            &self.pool,
            actor_id,
            id,
            input.expected_version,
            &input.reason,
        )
        .await?
        .ok_or_else(|| AppError::Conflict("反馈已被其他管理员更新或脱敏".to_owned()))?;
        AdminFeedbackDetail::try_from(updated)
    }
}
