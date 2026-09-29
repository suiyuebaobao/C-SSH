//! 编排版本策略三步草稿、正式资产门禁、追加发布与公开读取。

use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use tokio::fs;
use uuid::Uuid;

use crate::{
    AdminUpdatePolicySnapshot, PublishUpdatePolicyInput, PublishedUpdatePolicy,
    SaveUpdatePolicyDraftInput, Service, UpdatePolicyDraft, UpdatePolicyTargetRelease, local_file,
    model::{PolicyAssetRow, PublishedUpdatePolicyRow, UpdatePolicyDraftRow},
    repository,
};

use super::policy_validation::*;

impl Service {
    pub async fn public_update_policy(&self) -> AppResult<PublishedUpdatePolicy> {
        repository::policy::current(&self.pool)
            .await
            .map(|value| value.map_or_else(PublishedUpdatePolicy::disabled, Into::into))
    }

    pub async fn admin_update_policy(
        &self,
        actor: &AdminActor,
    ) -> AppResult<AdminUpdatePolicySnapshot> {
        require_actor(actor)?;
        let (draft, published, targets) = tokio::try_join!(
            repository::policy::draft(&self.pool),
            repository::policy::current(&self.pool),
            repository::policy::targets(&self.pool)
        )?;
        Ok(AdminUpdatePolicySnapshot {
            draft: draft.into(),
            published: published.map_or_else(PublishedUpdatePolicy::disabled, Into::into),
            target_releases: targets
                .into_iter()
                .map(|row| {
                    let eligible = policy_target_is_eligible(&row);
                    UpdatePolicyTargetRelease {
                        id: row.id,
                        version: row.version,
                        published_at: row.published_at,
                        eligible,
                        readiness: if eligible {
                            "ready".into()
                        } else {
                            "requires_exact_versioned_signed_local_assets".into()
                        },
                    }
                })
                .collect(),
        })
    }

    pub async fn save_update_policy_draft(
        &self,
        actor: &AdminActor,
        input: SaveUpdatePolicyDraftInput,
    ) -> AppResult<UpdatePolicyDraft> {
        let actor_id = require_actor(actor)?;
        let input = validate_draft(input)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let current = repository::policy::lock_draft(&mut transaction).await?;
        if current.revision != input.expected_revision {
            return Err(AppError::Conflict(
                "版本策略草稿已变化，请刷新后重试".into(),
            ));
        }
        let (disabled_versions, no_update_versions) = validated_policy_versions(
            &input,
            &current.disabled_versions,
            &current.no_update_versions,
        )?;
        let row = repository::policy::save_draft(
            &mut transaction,
            actor_id,
            input.expected_revision,
            &input,
            &disabled_versions,
            &no_update_versions,
        )
        .await?;
        repository::policy::audit(
            &mut transaction,
            actor_id,
            repository::policy::PolicyAudit {
                action: "update_policy.draft_saved",
                revision: row.revision,
                enabled: row.enabled,
                forced_count: row.forced_versions.len(),
                disabled_count: row.disabled_versions.len(),
                no_update_count: row.no_update_versions.len(),
                target_release_id: row.target_release_id,
                sha256_enabled: row.sha256_enabled,
            },
        )
        .await?;
        transaction.commit().await.map_err(transaction_error)?;
        mark_semantic_audit_recorded();
        Ok(row.into())
    }

    pub async fn publish_update_policy(
        &self,
        actor: &AdminActor,
        input: PublishUpdatePolicyInput,
    ) -> AppResult<PublishedUpdatePolicy> {
        let actor_id = require_actor(actor)?;
        if input.confirmation != "publish_update_policy" {
            return Err(AppError::Validation("版本策略发布确认无效".into()));
        }
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let draft = repository::policy::lock_draft(&mut transaction).await?;
        if draft.revision != input.expected_draft_revision {
            return Err(AppError::Conflict(
                "版本策略草稿已变化，请刷新后重试".into(),
            ));
        }
        let current_revision = repository::policy::lock_publication_revision(&mut transaction)
            .await?
            .unwrap_or(0);
        self.validate_policy_publication(&mut transaction, &draft)
            .await?;
        let revision = current_revision
            .checked_add(1)
            .ok_or_else(|| AppError::Conflict("版本策略修订已达上限".into()))?;
        let row = repository::policy::publish(&mut transaction, actor_id, revision, &draft).await?;
        repository::policy::audit(
            &mut transaction,
            actor_id,
            repository::policy::PolicyAudit {
                action: "update_policy.published",
                revision: row.revision,
                enabled: row.enabled,
                forced_count: row.forced_versions.len(),
                disabled_count: row.disabled_versions.len(),
                no_update_count: row.no_update_versions.len(),
                target_release_id: row.target_release_id,
                sha256_enabled: row.sha256_enabled,
            },
        )
        .await?;
        transaction.commit().await.map_err(transaction_error)?;
        mark_semantic_audit_recorded();
        Ok(row.into())
    }

    pub(super) async fn validate_policy_publication(
        &self,
        transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
        draft: &UpdatePolicyDraftRow,
    ) -> AppResult<()> {
        validate_policy_version_lists(
            &draft.forced_versions,
            &draft.disabled_versions,
            &draft.no_update_versions,
        )?;
        if draft.enabled {
            let target_id = draft
                .target_release_id
                .ok_or_else(|| AppError::Conflict("启用策略必须选择目标正式版本".into()))?;
            let target_version = repository::policy::target_version(transaction, target_id).await?;
            let (target_text, _) = cloud_domain::normalize_semantic_version(&target_version)
                .ok_or_else(|| AppError::Conflict("目标正式版本不是有效语义版本".into()))?;
            if draft.disabled_versions.contains(&target_text) {
                return Err(AppError::Validation("更新目标不能是关闭版本".into()));
            }
            validate_forced_versions(&draft.forced_versions, Some(&target_version))?;
            let assets = preferred_policy_assets(
                repository::policy::policy_assets(transaction, target_id).await?,
            );
            validate_formal_assets(&target_version, &assets)?;
            for asset in &assets {
                self.verify_policy_asset(asset).await?;
            }
            if draft.sha256_enabled {
                let identities =
                    repository::policy::forced_identities(transaction, &draft.forced_versions)
                        .await?;
                validate_forced_identities(&draft.forced_versions, &identities)?;
            }
        }
        Ok(())
    }

    async fn verify_policy_asset(&self, asset: &PolicyAssetRow) -> AppResult<()> {
        let path = local_file::resolve(self.download_root.as_path(), &asset.local_path).await?;
        let mut file = fs::File::open(&path)
            .await
            .map_err(|_| AppError::Conflict("策略目标包含不可读的本站资产".into()))?;
        let actual = file
            .metadata()
            .await
            .map_err(|_| AppError::Conflict("策略目标资产元数据不可读".into()))?
            .len();
        if u64::try_from(asset.byte_size).ok() != Some(actual) {
            return Err(AppError::Conflict("策略目标资产大小不一致".into()));
        }
        self.file_verifier
            .verify(&path, &mut file, actual, &asset.sha256)
            .await?;
        Ok(())
    }
}

pub(super) fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let id = actor.account_id();
    if id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".into()))
    } else {
        Ok(id)
    }
}

impl From<UpdatePolicyDraftRow> for UpdatePolicyDraft {
    fn from(value: UpdatePolicyDraftRow) -> Self {
        Self {
            revision: value.revision,
            enabled: value.enabled,
            forced_versions: value.forced_versions,
            disabled_versions: value.disabled_versions,
            no_update_versions: value.no_update_versions,
            target_release_id: value.target_release_id,
            sha256_enabled: value.sha256_enabled,
            updated_at: value.updated_at,
        }
    }
}

impl From<PublishedUpdatePolicyRow> for PublishedUpdatePolicy {
    fn from(value: PublishedUpdatePolicyRow) -> Self {
        Self {
            revision: value.revision,
            enabled: value.enabled,
            forced_versions: value.forced_versions,
            disabled_versions: value.disabled_versions,
            no_update_versions: value.no_update_versions,
            target_release_id: value.target_release_id,
            target_version: value.target_version,
            sha256_enabled: value.sha256_enabled,
            published_at: Some(value.published_at),
            published_by: Some(value.published_by),
        }
    }
}

pub(super) fn transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("版本策略事务失败".into())
}
