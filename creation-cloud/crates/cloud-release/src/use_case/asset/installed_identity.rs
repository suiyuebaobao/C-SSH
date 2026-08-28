//! 为认证发布编排提供一次性安装身份取证入口，普通资产表单不暴露该字段。

use cloud_domain::{AdminActor, AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::{ReleaseAsset, authorization, repository, validation};

pub(crate) async fn record_in_transaction(
    actor: &AdminActor,
    transaction: &mut Transaction<'_, Postgres>,
    asset_id: Uuid,
    installed_sha256: &str,
) -> AppResult<ReleaseAsset> {
    authorization::require(actor)?;
    let asset_id = validation::valid_id(asset_id, "资产标识")?;
    let installed_sha256 = validation::sha256(installed_sha256)?;
    let current = repository::asset::lock::execute(transaction, asset_id).await?;
    if !is_formal_identity(&current) {
        return Err(AppError::Validation(
            "安装身份只允许受支持的正式资产形态".into(),
        ));
    }
    if let Some(existing) = current.installed_sha256.as_deref() {
        return if existing == installed_sha256 {
            Ok(current)
        } else {
            Err(AppError::Conflict("安装身份摘要不可变".into()))
        };
    }
    repository::asset::installed_identity::execute(transaction, asset_id, &installed_sha256).await
}

fn is_formal_identity(asset: &ReleaseAsset) -> bool {
    matches!(
        (
            asset.platform.as_str(),
            asset.architecture.as_str(),
            asset.package_kind.as_str(),
        ),
        ("windows", "x86_64", "exe" | "msi" | "zip") | ("android", "aarch64", "apk")
    )
}

#[cfg(test)]
mod tests {
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
    fn legacy_and_current_formal_installation_identities_accept_evidence() {
        for (platform, architecture, package_kind) in [
            ("windows", "x86_64", "exe"),
            ("windows", "x86_64", "msi"),
            ("windows", "x86_64", "zip"),
            ("android", "aarch64", "apk"),
        ] {
            assert!(is_formal_identity(&asset(
                platform,
                architecture,
                package_kind
            )));
        }
        assert!(!is_formal_identity(&asset("linux", "x86_64", "appimage")));
        assert!(!is_formal_identity(&asset("android", "x86_64", "apk")));
    }
}
