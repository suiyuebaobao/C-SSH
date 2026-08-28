//! 原子核对版本化正式资产并调用发布域一次性 helper 写入安装身份。

use chrono::{DateTime, SecondsFormat, Utc};
use cloud_domain::{
    AdminActor, AppError, AppResult, FormalReleaseAssetIdentity, formal_release_asset_identities,
    mark_semantic_audit_recorded, normalize_semantic_version,
};

use crate::{
    InstalledIdentityEntryInput, RecordInstalledIdentitiesInput, RecordInstalledIdentitiesResult,
    Service, authorization, repository,
};

const MAX_ASSET_BYTES: i64 = 4 * 1024 * 1024 * 1024;
impl Service {
    pub async fn record_release_installed_identities(
        &self,
        actor: &AdminActor,
        input: RecordInstalledIdentitiesInput,
    ) -> AppResult<RecordInstalledIdentitiesResult> {
        let actor_id = authorization::require(actor)?;
        let input = validate(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release =
            repository::installed_identity::lock_release(&mut transaction, input.release_id)
                .await?;
        validate_release(&release, &input)?;
        let assets =
            repository::installed_identity::lock_assets(&mut transaction, input.release_id).await?;
        validate_assets(&assets, input.release_id, &input.entries)?;
        let idempotent = assets.iter().zip(&input.entries).all(|(asset, entry)| {
            asset.installed_sha256.as_deref() == Some(entry.installed_sha256.as_str())
        });
        for (asset, entry) in assets.iter().zip(&input.entries) {
            cloud_release::record_installed_sha256_in_transaction(
                actor,
                &mut transaction,
                asset.id,
                &entry.installed_sha256,
            )
            .await?;
        }
        repository::installed_identity::audit(
            &mut transaction,
            actor_id,
            input.release_id,
            serde_json::json!({
                "version": &input.version,
                "release_updated_at": &input.expected_release_updated_at,
                "product_input_sha256": &input.product_input_sha256,
                "entry_count": input.entries.len(),
                "entries": &input.entries,
                "idempotent": idempotent,
            }),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        mark_semantic_audit_recorded();
        Ok(RecordInstalledIdentitiesResult {
            release_id: release.id,
            version: release.version,
            release_updated_at: input.expected_release_updated_at,
            product_input_sha256: input.product_input_sha256,
            entry_count: input.entries.len(),
            idempotent,
        })
    }
}

fn validate(input: RecordInstalledIdentitiesInput) -> AppResult<RecordInstalledIdentitiesInput> {
    if input.release_id.is_nil() {
        return Err(AppError::Validation("版本标识无效".into()));
    }
    let Some((normalized_version, _)) = normalize_semantic_version(&input.version) else {
        return Err(AppError::Validation("版本号格式无效".into()));
    };
    if normalized_version != input.version {
        return Err(AppError::Validation("版本号必须使用规范 SemVer".into()));
    }
    let parsed_updated_at = DateTime::parse_from_rfc3339(&input.expected_release_updated_at)
        .map_err(|_| AppError::Validation("expected_release_updated_at 格式无效".into()))?
        .with_timezone(&Utc);
    if canonical_updated_at(&parsed_updated_at) != input.expected_release_updated_at {
        return Err(AppError::Validation(
            "expected_release_updated_at 必须是规范 UTC RFC3339".into(),
        ));
    }
    strict_sha256(&input.product_input_sha256, "product-input SHA256")?;
    let expected = formal_release_asset_identities(&input.version)
        .ok_or_else(|| AppError::Validation("版本号格式无效".into()))?;
    if input.entries.len() != expected.len() {
        return Err(AppError::Validation(
            "安装身份数量与版本正式资产合同不一致".into(),
        ));
    }
    for (entry, identity) in input.entries.iter().zip(expected) {
        validate_entry(&input.version, *identity, entry)?;
    }
    Ok(input)
}

fn validate_entry(
    version: &str,
    expected: FormalReleaseAssetIdentity,
    entry: &InstalledIdentityEntryInput,
) -> AppResult<()> {
    if (
        entry.platform.as_str(),
        entry.architecture.as_str(),
        entry.package_kind.as_str(),
    ) != expected
    {
        return Err(AppError::Validation(
            "安装身份顺序或形态与版本合同不一致".into(),
        ));
    }
    if entry.file_name != expected_file_name(version, expected.2) {
        return Err(AppError::Validation("安装身份文件名与版本不一致".into()));
    }
    if !(1..=MAX_ASSET_BYTES).contains(&entry.byte_size) {
        return Err(AppError::Validation("安装身份资产大小无效".into()));
    }
    strict_sha256(&entry.download_sha256, "下载 SHA256")?;
    strict_sha256(&entry.installed_sha256, "安装 SHA256")?;
    if entry.package_kind == "apk" && entry.installed_sha256 != entry.download_sha256 {
        return Err(AppError::Validation(
            "Android 安装身份必须等于 APK 下载摘要".into(),
        ));
    }
    Ok(())
}

fn validate_release(
    release: &repository::installed_identity::LockedIdentityRelease,
    input: &RecordInstalledIdentitiesInput,
) -> AppResult<()> {
    if release.id != input.release_id
        || release.version != input.version
        || release.channel != "stable"
        || !matches!(release.status.as_str(), "validating" | "published")
    {
        return Err(AppError::Conflict(
            "安装身份只允许匹配的 stable validating/published 版本".into(),
        ));
    }
    if canonical_updated_at(&release.updated_at) != input.expected_release_updated_at {
        return Err(AppError::Conflict("版本记录已变化，请刷新后重试".into()));
    }
    Ok(())
}

fn validate_assets(
    assets: &[repository::installed_identity::LockedIdentityAsset],
    release_id: uuid::Uuid,
    entries: &[InstalledIdentityEntryInput],
) -> AppResult<()> {
    if assets.len() != entries.len() {
        return Err(AppError::Conflict(
            "目标版本资产数量与安装身份清单不一致".into(),
        ));
    }
    for (asset, entry) in assets.iter().zip(entries) {
        if asset.release_id != release_id
            || asset.platform != entry.platform
            || asset.architecture != entry.architecture
            || asset.package_kind != entry.package_kind
            || asset.file_name != entry.file_name
            || asset.byte_size != entry.byte_size
            || asset.sha256 != entry.download_sha256
        {
            return Err(AppError::Conflict(
                "目标版本资产身份与发布清单不一致".into(),
            ));
        }
    }
    Ok(())
}

fn strict_sha256(value: &str, field: &str) -> AppResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(AppError::Validation(format!(
            "{field} 必须是 64 位小写十六进制字符串"
        )));
    }
    Ok(())
}

fn canonical_updated_at(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn expected_file_name(version: &str, package_kind: &str) -> String {
    let current_contract =
        formal_release_asset_identities(version).is_some_and(|identities| identities.len() == 3);
    let windows_prefix = if current_contract {
        "C-SSH"
    } else {
        "Creation-SSH"
    };
    match package_kind {
        "exe" => format!("{windows_prefix}_{version}_x64-setup.exe"),
        "msi" => format!("{windows_prefix}_{version}_x64_en-US.msi"),
        "zip" => format!("{windows_prefix}_{version}_portable-Windows-x64.zip"),
        "apk" => format!("C-SSH_{version}_android-arm64.apk"),
        _ => unreachable!("正式身份只包含版本合同声明的包形态"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn valid_input(version: &str) -> RecordInstalledIdentitiesInput {
        let identities = formal_release_asset_identities(version).expect("fixture 版本必须合法");
        RecordInstalledIdentitiesInput {
            release_id: Uuid::now_v7(),
            version: version.into(),
            expected_release_updated_at: "2026-08-17T12:34:56.123456Z".into(),
            product_input_sha256: "a".repeat(64),
            entries: identities
                .iter()
                .enumerate()
                .map(|(index, (platform, architecture, package_kind))| {
                    let download_sha256 = char::from(b'b' + u8::try_from(index).unwrap_or(0))
                        .to_string()
                        .repeat(64);
                    InstalledIdentityEntryInput {
                        platform: (*platform).into(),
                        architecture: (*architecture).into(),
                        package_kind: (*package_kind).into(),
                        file_name: expected_file_name(version, package_kind),
                        byte_size: i64::try_from(index).unwrap_or(0) + 1,
                        installed_sha256: if *package_kind == "apk" {
                            download_sha256.clone()
                        } else {
                            "f".repeat(64)
                        },
                        download_sha256,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn accepts_only_the_exact_versioned_asset_identity() {
        let legacy = valid_input("0.8.7");
        assert_eq!(legacy.entries.len(), 4);
        assert!(
            legacy
                .entries
                .iter()
                .any(|entry| entry.package_kind == "msi")
        );
        assert!(validate(legacy).is_ok());

        let current = valid_input("0.8.8");
        assert_eq!(current.entries.len(), 3);
        assert!(
            current
                .entries
                .iter()
                .all(|entry| entry.package_kind != "msi")
        );
        assert_eq!(current.entries[0].file_name, "C-SSH_0.8.8_x64-setup.exe");
        assert_eq!(
            current.entries[1].file_name,
            "C-SSH_0.8.8_portable-Windows-x64.zip"
        );
        assert!(validate(current).is_ok());

        let mut input = valid_input("0.8.8");
        input.entries.swap(0, 1);
        assert!(validate(input).is_err());
        let mut input = valid_input("0.8.8");
        input.entries[2].installed_sha256 = "0".repeat(64);
        assert!(validate(input).is_err());
        let mut input = valid_input("0.8.8");
        input.product_input_sha256 = "A".repeat(64);
        assert!(validate(input).is_err());
    }

    #[test]
    fn updated_at_requires_the_exact_utc_rfc3339_representation() {
        let mut input = valid_input("0.8.8");
        input.expected_release_updated_at = "2026-08-17T20:34:56.123456+08:00".into();
        assert!(validate(input).is_err());
    }

    #[test]
    fn stale_release_updated_at_is_a_conflict_after_the_release_lock() {
        let input = valid_input("0.8.8");
        let release = repository::installed_identity::LockedIdentityRelease {
            id: input.release_id,
            version: input.version.clone(),
            channel: "stable".into(),
            status: "validating".into(),
            updated_at: DateTime::parse_from_rfc3339("2026-08-17T12:34:57.123456Z")
                .expect("fixture 时间必须合法")
                .with_timezone(&Utc),
        };
        assert!(matches!(
            validate_release(&release, &input),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn windows_installed_identity_may_differ_from_download_identity() {
        let input = valid_input("0.8.8");
        for entry in input
            .entries
            .iter()
            .filter(|entry| entry.platform == "windows")
        {
            assert_ne!(entry.installed_sha256, entry.download_sha256);
        }
        assert!(validate(input).is_ok());
    }
}
