//! 执行版本元数据更新和单向发布状态迁移。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    Release, ReleaseAsset, ReleaseStatus, Service, UpdateReleaseInput, authorization, repository,
    validation,
};

impl Service {
    pub async fn update_release(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateReleaseInput,
    ) -> AppResult<Release> {
        authorization::require(actor)?;
        let id = validation::valid_id(id, "版本标识")?;
        let current = repository::release::get::execute(&self.pool, id).await?;
        let input = normalize(input)?;
        ensure_update_allowed(&current, &input)?;

        if current.status == ReleaseStatus::Validating
            && input.status == Some(ReleaseStatus::Published)
        {
            let assets = repository::asset::list::execute(&self.pool, id).await?;
            ensure_formal_asset_shape(&current.version, &assets)?;
        }
        if input.status == Some(ReleaseStatus::Published)
            && repository::asset::signature::has_invalid_metadata(&self.pool, id).await?
        {
            return Err(AppError::Conflict(
                "Windows 正式资产必须有 updater signature，其他资产不得携带该字段".into(),
            ));
        }

        repository::release::update::execute(&self.pool, id, &input).await
    }
}

fn ensure_formal_asset_shape(version: &str, assets: &[ReleaseAsset]) -> AppResult<()> {
    let expected = cloud_domain::formal_release_asset_identities(version)
        .ok_or_else(|| AppError::Internal("数据库中的版本号无效".into()))?;
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
            "发布版本的正式资产形态与版本合同不一致".into(),
        ));
    }
    Ok(())
}

pub(crate) fn normalize(input: UpdateReleaseInput) -> AppResult<UpdateReleaseInput> {
    Ok(UpdateReleaseInput {
        title_zh: validation::optional_text(input.title_zh.as_deref(), "中文标题", 200)?,
        title_en: validation::optional_text(input.title_en.as_deref(), "英文标题", 200)?,
        notes_zh: validation::optional_text(input.notes_zh.as_deref(), "中文发布说明", 20_000)?,
        notes_en: validation::optional_text(input.notes_en.as_deref(), "英文发布说明", 20_000)?,
        status: input.status,
    })
}

fn ensure_update_allowed(current: &Release, input: &UpdateReleaseInput) -> AppResult<()> {
    let changes_metadata = input.title_zh.is_some()
        || input.title_en.is_some()
        || input.notes_zh.is_some()
        || input.notes_en.is_some();
    if !changes_metadata && input.status.is_none() {
        return Err(AppError::Validation("版本更新内容不能为空".into()));
    }
    if changes_metadata && !current.status.allows_asset_mutation() {
        return Err(AppError::Conflict(
            "已发布版本的说明与标题不可原地覆盖".into(),
        ));
    }
    let target = input.status.unwrap_or(current.status);
    if !valid_transition(current.status, target) {
        return Err(AppError::Conflict("发布状态迁移不合法".into()));
    }
    Ok(())
}

pub(crate) const fn valid_transition(from: ReleaseStatus, to: ReleaseStatus) -> bool {
    matches!(
        (from, to),
        (
            ReleaseStatus::Draft,
            ReleaseStatus::Draft | ReleaseStatus::Validating
        ) | (
            ReleaseStatus::Validating,
            ReleaseStatus::Validating | ReleaseStatus::Published
        ) | (
            ReleaseStatus::Published,
            ReleaseStatus::Published | ReleaseStatus::Revoked | ReleaseStatus::Hidden
        ) | (ReleaseStatus::Revoked, ReleaseStatus::Revoked)
            | (ReleaseStatus::Hidden, ReleaseStatus::Hidden)
    )
}

#[cfg(test)]
mod release_shape_tests {
    use chrono::Utc;

    use super::*;

    fn asset(platform: &str, architecture: &str, package_kind: &str) -> ReleaseAsset {
        ReleaseAsset {
            id: Uuid::now_v7(),
            release_id: Uuid::now_v7(),
            platform: platform.into(),
            architecture: architecture.into(),
            package_kind: package_kind.into(),
            file_name: "asset.bin".into(),
            byte_size: 1,
            sha256: "a".repeat(64),
            installed_sha256: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn publishing_uses_the_versioned_three_or_four_asset_contract() {
        let legacy = vec![
            asset("windows", "x86_64", "exe"),
            asset("windows", "x86_64", "msi"),
            asset("windows", "x86_64", "zip"),
            asset("android", "aarch64", "apk"),
        ];
        assert!(ensure_formal_asset_shape("0.8.7", &legacy).is_ok());
        assert!(ensure_formal_asset_shape("0.8.8", &legacy).is_err());

        let current = vec![legacy[0].clone(), legacy[2].clone(), legacy[3].clone()];
        assert!(ensure_formal_asset_shape("0.8.8", &current).is_ok());
        assert!(ensure_formal_asset_shape("0.8.7", &current).is_err());
    }
}
