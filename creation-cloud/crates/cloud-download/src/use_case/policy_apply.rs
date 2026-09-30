//! 在一个事务内保存并发布版本策略，保护两个修订并记录语义审计。

use super::{
    policy::{require_actor, transaction_error},
    policy_validation::{validate_draft, validated_policy_versions},
};
use crate::{ApplyUpdatePolicyInput, PublishedUpdatePolicy, Service, repository};
use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};

impl Service {
    pub async fn apply_update_policy(
        &self,
        actor: &AdminActor,
        input: ApplyUpdatePolicyInput,
    ) -> AppResult<PublishedUpdatePolicy> {
        let actor_id = require_actor(actor)?;
        if input.confirmation != "apply_update_policy" || input.expected_published_revision < 0 {
            return Err(AppError::Validation("版本策略生效确认无效".into()));
        }
        let input_draft = validate_draft(input.draft)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let current = repository::policy::lock_draft(&mut transaction).await?;
        if current.revision != input_draft.expected_revision {
            return Err(AppError::Conflict(
                "版本策略草稿已变化，请刷新后重试".into(),
            ));
        }
        let revision = repository::policy::lock_publication_revision(&mut transaction)
            .await?
            .unwrap_or(0);
        if revision != input.expected_published_revision {
            return Err(AppError::Conflict("已生效策略已变化，请刷新后重试".into()));
        }
        let (disabled, no_update) = validated_policy_versions(
            &input_draft,
            &current.disabled_versions,
            &current.no_update_versions,
        )?;
        let draft = repository::policy::save_draft(
            &mut transaction,
            actor_id,
            current.revision,
            &input_draft,
            &disabled,
            &no_update,
        )
        .await?;
        self.validate_policy_publication(&mut transaction, &draft)
            .await?;
        let revision = revision
            .checked_add(1)
            .ok_or_else(|| AppError::Conflict("版本策略修订已达上限".into()))?;
        let published =
            repository::policy::publish(&mut transaction, actor_id, revision, &draft).await?;
        repository::policy::audit(
            &mut transaction,
            actor_id,
            repository::policy::PolicyAudit {
                action: "update_policy.applied",
                revision,
                enabled: published.enabled,
                forced_count: published.forced_versions.len(),
                disabled_count: published.disabled_versions.len(),
                no_update_count: published.no_update_versions.len(),
                target_release_id: published.target_release_id,
                sha256_enabled: published.sha256_enabled,
            },
        )
        .await?;
        transaction.commit().await.map_err(transaction_error)?;
        mark_semantic_audit_recorded();
        Ok(published.into())
    }
}
