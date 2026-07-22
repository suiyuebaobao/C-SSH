//! 在管理员二次授权、状态机和期望版本校验后原子推进反馈状态。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminFeedbackDetail, FeedbackStatus, Service, UpdateFeedbackStatusInput, authorization,
    repository, validation,
};

use super::audit;

impl Service {
    pub async fn update_feedback_status(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateFeedbackStatusInput,
    ) -> AppResult<AdminFeedbackDetail> {
        let actor_id = authorization::admin(actor)?;
        let id = validation::id(id)?;
        let input = validation::status(input)?;
        let request_id = audit::request_id();
        let audit_context = repository::semantic_audit::AuditContext {
            actor_id,
            feedback_id: id,
            request_id: &request_id,
            reason: &input.reason,
        };
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::error::transaction)?;
        let current = match repository::get::management_for_update(&mut transaction, id).await {
            Ok(Some(current)) => current,
            Ok(None) => {
                if let Err(error) = repository::semantic_audit::status_failure(
                    &mut transaction,
                    &repository::semantic_audit::StatusFailure {
                        audit: audit_context,
                        from_status: None,
                        to_status: input.status,
                        failure_code: "not_found",
                    },
                )
                .await
                {
                    return audit::rollback(transaction, error).await;
                }
                return audit::commit_failure(
                    transaction,
                    AppError::NotFound("反馈不存在".to_owned()),
                )
                .await;
            }
            Err(error) => return audit::rollback(transaction, error).await,
        };
        let current_status = match FeedbackStatus::try_from(current.status.as_str()) {
            Ok(status) => status,
            Err(error) => return audit::rollback(transaction, error).await,
        };
        if current.version != input.expected_version {
            if let Err(error) = repository::semantic_audit::status_failure(
                &mut transaction,
                &repository::semantic_audit::StatusFailure {
                    audit: audit_context,
                    from_status: Some(current_status),
                    to_status: input.status,
                    failure_code: "version_conflict",
                },
            )
            .await
            {
                return audit::rollback(transaction, error).await;
            }
            return audit::commit_failure(
                transaction,
                AppError::Conflict("反馈已被其他管理员更新".to_owned()),
            )
            .await;
        }
        if !valid_transition(current_status, input.status) {
            if let Err(error) = repository::semantic_audit::status_failure(
                &mut transaction,
                &repository::semantic_audit::StatusFailure {
                    audit: audit_context,
                    from_status: Some(current_status),
                    to_status: input.status,
                    failure_code: "invalid_transition",
                },
            )
            .await
            {
                return audit::rollback(transaction, error).await;
            }
            return audit::commit_failure(
                transaction,
                AppError::Conflict("反馈状态迁移不合法".to_owned()),
            )
            .await;
        }
        let updated = match repository::update::status(
            &mut transaction,
            &repository::update::StatusMutation {
                audit: audit_context,
                expected_version: input.expected_version,
                current: current_status,
                target: input.status,
            },
        )
        .await
        {
            Ok(Some(updated)) => updated,
            Ok(None) => {
                let error = AppError::Conflict("反馈已被其他管理员更新".to_owned());
                return audit::rollback(transaction, error).await;
            }
            Err(error) => return audit::rollback(transaction, error).await,
        };
        let detail = match AdminFeedbackDetail::try_from(updated) {
            Ok(detail) => detail,
            Err(error) => return audit::rollback(transaction, error).await,
        };
        audit::commit_success(transaction, detail).await
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
