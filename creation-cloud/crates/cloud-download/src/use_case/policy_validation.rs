//! 验证版本策略列表、目标资产与安装身份。

use crate::{
    SaveUpdatePolicyDraftInput,
    model::{ForcedIdentityRow, PolicyAssetRow},
};
use cloud_domain::{
    AppError, AppResult, formal_release_asset_identities, normalize_semantic_version,
};
use std::collections::HashSet;

pub(super) fn validate_draft(
    mut input: SaveUpdatePolicyDraftInput,
) -> AppResult<SaveUpdatePolicyDraftInput> {
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
    if let Some(values) = &input.disabled_versions {
        input.disabled_versions = Some(normalize_policy_versions(values, "关闭")?);
    }
    if let Some(values) = &input.no_update_versions {
        input.no_update_versions = Some(normalize_policy_versions(values, "无需更新")?);
    }
    Ok(input)
}

pub(super) fn validate_forced_versions(
    values: &[String],
    target: Option<&str>,
) -> AppResult<Vec<String>> {
    let target = target
        .map(|value| {
            normalize_semantic_version(value)
                .ok_or_else(|| AppError::Conflict("目标正式版本不是有效语义版本".into()))
        })
        .transpose()?;
    let normalized = normalize_policy_versions(values, "强制更新")?;
    for value in &normalized {
        let (_, version) = normalize_semantic_version(value)
            .ok_or_else(|| AppError::Validation("强制更新列表包含无效语义版本".into()))?;
        if target
            .as_ref()
            .is_some_and(|(_, target)| version >= *target)
        {
            return Err(AppError::Validation("强制更新版本必须低于目标版本".into()));
        }
    }
    Ok(normalized)
}

fn normalize_policy_versions(values: &[String], label: &str) -> AppResult<Vec<String>> {
    if values.len() > 128 {
        return Err(AppError::Validation(format!("{label}版本数量超过上限")));
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let (text, _) = normalize_semantic_version(value)
            .ok_or_else(|| AppError::Validation(format!("{label}列表包含无效语义版本")))?;
        if seen.insert(text.clone()) {
            normalized.push(text);
        }
    }
    normalized.sort();
    Ok(normalized)
}

pub(super) fn policy_target_is_eligible(row: &crate::model::PolicyTargetRow) -> bool {
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

pub(super) fn validate_formal_assets(version: &str, assets: &[PolicyAssetRow]) -> AppResult<()> {
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

pub(super) fn preferred_policy_assets(rows: Vec<PolicyAssetRow>) -> Vec<PolicyAssetRow> {
    let mut seen = HashSet::new();
    rows.into_iter().filter(|row| seen.insert(row.id)).collect()
}

pub(super) fn validate_forced_identities(
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

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(super) fn validated_policy_versions(
    input: &SaveUpdatePolicyDraftInput,
    previous_disabled: &[String],
    previous_no_update: &[String],
) -> AppResult<(Vec<String>, Vec<String>)> {
    let disabled = input
        .disabled_versions
        .as_deref()
        .unwrap_or(previous_disabled)
        .to_vec();
    let no_update = input
        .no_update_versions
        .as_deref()
        .unwrap_or(previous_no_update)
        .to_vec();
    validate_policy_version_lists(&input.forced_versions, &disabled, &no_update)?;
    Ok((disabled, no_update))
}

pub(super) fn validate_policy_version_lists(
    forced: &[String],
    disabled: &[String],
    no_update: &[String],
) -> AppResult<()> {
    if intersects(forced, disabled) {
        return Err(AppError::Validation(
            "同一版本不能同时强制更新和关闭".into(),
        ));
    }
    if intersects(forced, no_update) {
        return Err(AppError::Validation(
            "同一版本不能同时强制更新和无需更新".into(),
        ));
    }
    if intersects(disabled, no_update) {
        return Err(AppError::Validation(
            "同一版本不能同时关闭和无需更新".into(),
        ));
    }
    Ok(())
}

fn intersects(left: &[String], right: &[String]) -> bool {
    left.iter().any(|version| right.contains(version))
}
