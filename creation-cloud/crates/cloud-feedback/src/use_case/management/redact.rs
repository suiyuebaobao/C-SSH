//! 将可能含敏感内容的反馈替换为固定墓碑并关闭，操作不可恢复或物理删除。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminFeedbackDetail, RedactFeedbackInput, Service, authorization, repository, validation,
};

use super::audit;

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
                if let Err(error) = repository::semantic_audit::redaction_failure(
                    &mut transaction,
                    &repository::semantic_audit::RedactionFailure {
                        audit: audit_context,
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
        if current.redacted_at.is_some() {
            if let Err(error) = repository::semantic_audit::redaction_failure(
                &mut transaction,
                &repository::semantic_audit::RedactionFailure {
                    audit: audit_context,
                    failure_code: "already_redacted",
                },
            )
            .await
            {
                return audit::rollback(transaction, error).await;
            }
            return audit::commit_failure(
                transaction,
                AppError::Conflict("反馈已经完成安全脱敏".to_owned()),
            )
            .await;
        }
        if current.version != input.expected_version {
            if let Err(error) = repository::semantic_audit::redaction_failure(
                &mut transaction,
                &repository::semantic_audit::RedactionFailure {
                    audit: audit_context,
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
        let updated = match repository::update::redact(
            &mut transaction,
            &repository::update::RedactionMutation {
                audit: audit_context,
                expected_version: input.expected_version,
            },
        )
        .await
        {
            Ok(Some(updated)) => updated,
            Ok(None) => {
                let error = AppError::Conflict("反馈已被其他管理员更新或脱敏".to_owned());
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
