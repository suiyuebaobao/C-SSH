//! 编排版本策略三步草稿、正式资产门禁、追加发布与公开读取。

use std::collections::HashSet;

use cloud_domain::{
    AdminActor, AppError, AppResult, formal_release_asset_identities, mark_semantic_audit_recorded,
    normalize_semantic_version,
};
use tokio::fs;
use uuid::Uuid;

use crate::{
    AdminUpdatePolicySnapshot, PublishUpdatePolicyInput, PublishedUpdatePolicy,
    SaveUpdatePolicyDraftInput, Service, UpdatePolicyDraft, UpdatePolicyTargetRelease, local_file,
    model::{ForcedIdentityRow, PolicyAssetRow, PublishedUpdatePolicyRow, UpdatePolicyDraftRow},
    repository,
};

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
        let row = repository::policy::save_draft(
            &mut transaction,
            actor_id,
            input.expected_revision,
            input.enabled,
            &input.forced_versions,
            input.target_release_id,
            input.sha256_enabled,
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
        if draft.enabled {
            let target_id = draft
                .target_release_id
                .ok_or_else(|| AppError::Conflict("启用策略必须选择目标正式版本".into()))?;
            let target_version =
                repository::policy::target_version(&mut transaction, target_id).await?;
            validate_forced_versions(&draft.forced_versions, Some(&target_version))?;
            let assets = preferred_policy_assets(
                repository::policy::policy_assets(&mut transaction, target_id).await?,
            );
            validate_formal_assets(&target_version, &assets)?;
            for asset in &assets {
                self.verify_policy_asset(asset).await?;
            }
            if draft.sha256_enabled {
                let identities =
                    repository::policy::forced_identities(&mut transaction, &draft.forced_versions)
                        .await?;
                validate_forced_identities(&draft.forced_versions, &identities)?;
            }
        }
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
                target_release_id: row.target_release_id,
                sha256_enabled: row.sha256_enabled,
            },
        )
        .await?;
        transaction.commit().await.map_err(transaction_error)?;
        mark_semantic_audit_recorded();
        Ok(row.into())
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

fn validate_draft(mut input: SaveUpdatePolicyDraftInput) -> AppResult<SaveUpdatePolicyDraftInput> {
    if input.expected_revision < 0 {
        return Err(AppError::Validation("版本策略草稿修订无效".into()));
    }
    if input.enabled {
        if input.target_release_id.is_none_or(|id| id.is_nil()) {
            return Err(AppError::Validation("启用策略必须选择目标正式版本".into()));
        }
        input.forced_versions = validate_forced_versions(&input.forced_versions, None)?;
    } else {
        input.forced_versions.clear();
        input.target_release_id = None;
    }
    Ok(input)
}

fn validate_forced_versions(values: &[String], target: Option<&str>) -> AppResult<Vec<String>> {
    if values.len() > 128 {
        return Err(AppError::Validation("强制更新版本数量超过上限".into()));
    }
    let target = target
        .map(|value| {
            normalize_semantic_version(value)
                .ok_or_else(|| AppError::Conflict("目标正式版本不是有效语义版本".into()))
        })
        .transpose()?;
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let (text, version) = normalize_semantic_version(value)
            .ok_or_else(|| AppError::Validation("强制更新列表包含无效语义版本".into()))?;
        if target
            .as_ref()
            .is_some_and(|(_, target)| version >= *target)
        {
            return Err(AppError::Validation("强制更新版本必须低于目标版本".into()));
        }
        if seen.insert(text.clone()) {
            normalized.push(text);
        }
    }
    normalized.sort();
    Ok(normalized)
}

fn policy_target_is_eligible(row: &crate::model::PolicyTargetRow) -> bool {
    let Some(expected) = formal_release_asset_identities(&row.version) else {
        return false;
    };
    let Ok(expected_count) = i64::try_from(expected.len()) else {
        return false;
    };
    let Ok(signature_count) = i64::try_from(
        expected
            .iter()
            .filter(|(platform, _, _)| *platform == "windows")
            .count(),
    ) else {
        return false;
    };
    row.asset_count == expected_count
        && row.formal_asset_count == expected_count
        && row.required_signature_count == signature_count
        && row.local_source_count == expected_count
}

fn validate_formal_assets(version: &str, assets: &[PolicyAssetRow]) -> AppResult<()> {
    let expected = formal_release_asset_identities(version)
        .ok_or_else(|| AppError::Conflict("策略目标版本不是有效语义版本".into()))?;
    if assets.len() != expected.len()
        || expected.iter().any(|identity| {
            !assets.iter().any(|asset| {
                (
                    asset.platform.as_str(),
                    asset.architecture.as_str(),
                    asset.package_kind.as_str(),
                ) == *identity
            })
        })
    {
        return Err(AppError::Conflict(
            "策略目标正式资产形态与版本合同不一致".into(),
        ));
    }
    for asset in assets {
        if asset.id.is_nil()
            || asset.source_id.is_nil()
            || asset.file_name.trim().is_empty()
            || asset.local_path.trim().is_empty()
        {
            return Err(AppError::Conflict("策略目标资产或本站来源身份无效".into()));
        }
        let windows_update_asset = asset.platform == "windows"
            && matches!(asset.package_kind.as_str(), "exe" | "msi" | "zip");
        match (windows_update_asset, asset.updater_signature.as_deref()) {
            (true, Some(signature)) => {
                crate::signature::validate(signature)?;
            }
            (true, None) => {
                return Err(AppError::Conflict(
                    "Windows 正式策略资产缺少 updater signature".into(),
                ));
            }
            (false, Some(_)) => {
                return Err(AppError::Conflict(
                    "Android 或其它非 Windows 策略资产不得携带 updater signature".into(),
                ));
            }
            (false, None) => {}
        }
    }
    Ok(())
}

fn preferred_policy_assets(rows: Vec<PolicyAssetRow>) -> Vec<PolicyAssetRow> {
    let mut seen = HashSet::new();
    rows.into_iter().filter(|row| seen.insert(row.id)).collect()
}

fn validate_forced_identities(
    forced_versions: &[String],
    rows: &[ForcedIdentityRow],
) -> AppResult<()> {
    let expected_count = forced_versions.iter().try_fold(0_usize, |count, version| {
        let identities = formal_release_asset_identities(version)
            .ok_or_else(|| AppError::Conflict("强制版本不是有效语义版本".into()))?;
        count
            .checked_add(identities.len())
            .ok_or_else(|| AppError::Conflict("强制版本安装身份数量溢出".into()))
    })?;
    if rows.len() != expected_count {
        return Err(AppError::Conflict(
            "开启 SHA-256 的强制版本必须具备对应版本的全部安装身份".into(),
        ));
    }
    for version in forced_versions {
        let expected = formal_release_asset_identities(version)
            .ok_or_else(|| AppError::Conflict("强制版本不是有效语义版本".into()))?;
        for expected in expected.iter().copied() {
            let mut matching = rows.iter().filter(|row| {
                row.version == *version
                    && (
                        row.platform.as_str(),
                        row.architecture.as_str(),
                        row.package_kind.as_str(),
                    ) == expected
            });
            let Some(row) = matching.next() else {
                return Err(AppError::Conflict(
                    "开启 SHA-256 的强制版本缺少安装身份".into(),
                ));
            };
            if matching.next().is_some() {
                return Err(AppError::Conflict("强制版本安装身份不唯一".into()));
            }
            let installed = row.installed_sha256.as_deref().ok_or_else(|| {
                AppError::Conflict("开启 SHA-256 的强制版本安装身份尚未回填".into())
            })?;
            if !is_sha256(installed) || !is_sha256(&row.asset_sha256) {
                return Err(AppError::Conflict("强制版本安装身份摘要无效".into()));
            }
            if expected == ("android", "aarch64", "apk") && installed != row.asset_sha256 {
                return Err(AppError::Conflict(
                    "Android APK 安装身份必须等于最终资产摘要".into(),
                ));
            }
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
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
            target_release_id: value.target_release_id,
            target_version: value.target_version,
            sha256_enabled: value.sha256_enabled,
            published_at: Some(value.published_at),
            published_by: Some(value.published_by),
        }
    }
}

fn transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("版本策略事务失败".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(platform: &str, architecture: &str, kind: &str) -> PolicyAssetRow {
        PolicyAssetRow {
            id: Uuid::now_v7(),
            platform: platform.into(),
            architecture: architecture.into(),
            package_kind: kind.into(),
            file_name: "asset.bin".into(),
            byte_size: 1,
            sha256: "a".repeat(64),
            updater_signature: (platform == "windows").then(|| "a".repeat(64)),
            source_id: Uuid::now_v7(),
            local_path: "objects/example".into(),
        }
    }

    fn forced_identity(
        version: &str,
        platform: &str,
        architecture: &str,
        kind: &str,
    ) -> ForcedIdentityRow {
        let asset_sha256 = if platform == "android" {
            "d".repeat(64)
        } else {
            match kind {
                "exe" => "a",
                "msi" => "b",
                _ => "c",
            }
            .repeat(64)
        };
        ForcedIdentityRow {
            version: version.into(),
            platform: platform.into(),
            architecture: architecture.into(),
            package_kind: kind.into(),
            installed_sha256: Some(asset_sha256.clone()),
            asset_sha256,
        }
    }

    #[test]
    fn updater_signature_scope_is_enforced_before_policy_publication() {
        let legacy = vec![
            asset("windows", "x86_64", "exe"),
            asset("windows", "x86_64", "msi"),
            asset("windows", "x86_64", "zip"),
            asset("android", "aarch64", "apk"),
        ];
        assert!(validate_formal_assets("0.8.7", &legacy).is_ok());
        assert!(validate_formal_assets("0.8.8", &legacy).is_err());

        let mut assets = vec![legacy[0].clone(), legacy[2].clone(), legacy[3].clone()];
        assert!(validate_formal_assets("0.8.8", &assets).is_ok());
        assets[2].updater_signature = Some("a".repeat(64));
        assert!(validate_formal_assets("0.8.8", &assets).is_err());
        assets[2].updater_signature = None;
        assets[0].updater_signature = None;
        assert!(validate_formal_assets("0.8.8", &assets).is_err());
        assets[0].updater_signature = Some("a".repeat(64));
        assets[1].updater_signature = None;
        assert!(validate_formal_assets("0.8.8", &assets).is_err());
    }

    #[test]
    fn forced_versions_are_semver_deduplicated_and_below_target() {
        let values = vec!["v0.7.7".into(), "0.7.7".into(), "0.7.9".into()];
        assert_eq!(
            validate_forced_versions(&values, Some("0.8.0"))
                .unwrap()
                .len(),
            2
        );
        assert!(validate_forced_versions(&["0.8.0".into()], Some("0.8.0")).is_err());
        assert!(validate_forced_versions(&[], Some("legacy")).is_err());
    }

    #[test]
    fn sha_policy_requires_each_forced_versions_exact_identity_shape() {
        let versions = vec!["0.8.7".to_owned(), "0.8.8".to_owned()];
        let mut rows = vec![
            forced_identity("0.8.7", "windows", "x86_64", "exe"),
            forced_identity("0.8.7", "windows", "x86_64", "msi"),
            forced_identity("0.8.7", "windows", "x86_64", "zip"),
            forced_identity("0.8.7", "android", "aarch64", "apk"),
            forced_identity("0.8.8", "windows", "x86_64", "exe"),
            forced_identity("0.8.8", "windows", "x86_64", "zip"),
            forced_identity("0.8.8", "android", "aarch64", "apk"),
        ];
        assert!(validate_forced_identities(&versions, &rows).is_ok());
        rows[6].installed_sha256 = Some("e".repeat(64));
        assert!(validate_forced_identities(&versions, &rows).is_err());
        let android_asset_sha256 = rows[6].asset_sha256.clone();
        rows[6].installed_sha256 = Some(android_asset_sha256);
        rows[0].installed_sha256 = None;
        assert!(validate_forced_identities(&versions, &rows).is_err());
        rows[0].installed_sha256 = Some("a".repeat(64));
        rows.pop();
        assert!(validate_forced_identities(&versions, &rows).is_err());
    }

    #[test]
    fn policy_target_readiness_uses_the_release_versions_expected_counts() {
        let mut row = crate::model::PolicyTargetRow {
            id: Uuid::now_v7(),
            version: "0.8.8".into(),
            published_at: chrono::Utc::now(),
            asset_count: 3,
            formal_asset_count: 3,
            required_signature_count: 2,
            local_source_count: 3,
        };
        assert!(policy_target_is_eligible(&row));
        row.required_signature_count = 3;
        assert!(!policy_target_is_eligible(&row));
        row.version = "0.8.7".into();
        row.asset_count = 4;
        row.formal_asset_count = 4;
        row.required_signature_count = 3;
        row.local_source_count = 4;
        assert!(policy_target_is_eligible(&row));
    }
}
