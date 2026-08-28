//! 定义按版本演进且保留历史兼容的正式发布资产形态。

use crate::normalize_semantic_version;

pub type FormalReleaseAssetIdentity = (&'static str, &'static str, &'static str);

const LEGACY_FORMAL_ASSETS: [FormalReleaseAssetIdentity; 4] = [
    ("windows", "x86_64", "exe"),
    ("windows", "x86_64", "msi"),
    ("windows", "x86_64", "zip"),
    ("android", "aarch64", "apk"),
];

const CURRENT_FORMAL_ASSETS: [FormalReleaseAssetIdentity; 3] = [
    ("windows", "x86_64", "exe"),
    ("windows", "x86_64", "zip"),
    ("android", "aarch64", "apk"),
];

/// `0.8.8`（含预发布版本）起不再把 MSI 作为正式资产；旧版本仍保持四形态。
#[must_use]
pub fn formal_release_asset_identities(
    value: &str,
) -> Option<&'static [FormalReleaseAssetIdentity]> {
    let (_, version) = normalize_semantic_version(value)?;
    let (_, boundary) = normalize_semantic_version("0.8.8-0")?;
    Some(if version >= boundary {
        &CURRENT_FORMAL_ASSETS
    } else {
        &LEGACY_FORMAL_ASSETS
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_shape_changes_at_0_8_8_without_removing_legacy_msi() {
        let legacy = formal_release_asset_identities("0.8.7").expect("旧版本必须合法");
        assert_eq!(legacy.len(), 4);
        assert!(legacy.contains(&("windows", "x86_64", "msi")));

        for version in ["0.8.8-beta.1", "0.8.8", "1.0.0"] {
            let current = formal_release_asset_identities(version).expect("新版本必须合法");
            assert_eq!(current.len(), 3);
            assert!(!current.contains(&("windows", "x86_64", "msi")));
        }
        assert!(formal_release_asset_identities("not-semver").is_none());
    }
}
