//! 分离完整手动下载集合与保留历史兼容的自动更新、安装身份集合。

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

const DESKTOP_FORMAL_ASSETS: [FormalReleaseAssetIdentity; 6] = [
    ("windows", "x86_64", "exe"),
    ("windows", "x86_64", "zip"),
    ("android", "aarch64", "apk"),
    ("macos", "aarch64", "tar_gz"),
    ("linux", "x86_64", "appimage"),
    ("linux", "aarch64", "appimage"),
];

const MANUAL_ONLY_ASSETS: [FormalReleaseAssetIdentity; 3] = [
    ("windows", "x86_64", "zip"),
    ("macos", "aarch64", "dmg"),
    ("android", "aarch64", "apk"),
];

const MANUAL_LINUX_ASSETS: [FormalReleaseAssetIdentity; 4] = [
    MANUAL_ONLY_ASSETS[0],
    MANUAL_ONLY_ASSETS[1],
    MANUAL_ONLY_ASSETS[2],
    ("linux", "x86_64", "appimage"),
];

pub const MANUAL_RELEASE_ASSETS: [FormalReleaseAssetIdentity; 10] = [
    ("windows", "x86_64", "exe"),
    ("windows", "x86_64", "zip"),
    ("windows", "aarch64", "exe"),
    ("windows", "aarch64", "zip"),
    ("macos", "aarch64", "dmg"),
    ("android", "aarch64", "apk"),
    ("linux", "x86_64", "deb"),
    ("linux", "x86_64", "appimage"),
    ("linux", "aarch64", "deb"),
    ("linux", "aarch64", "appimage"),
];

/// 自动更新与安装身份按版本冻结，历史发布合同不随新增平台变化。
/// `0.8.8`（含预发布版本）起不再包含 MSI，旧版本仍保持四形态。
#[must_use]
pub fn formal_release_asset_identities(
    value: &str,
) -> Option<&'static [FormalReleaseAssetIdentity]> {
    let (_, version) = normalize_semantic_version(value)?;
    let (_, boundary) = normalize_semantic_version("0.8.8-0")?;
    let (_, desktop_boundary) = normalize_semantic_version("0.9.7-0")?;
    Some(if version >= desktop_boundary {
        &DESKTOP_FORMAL_ASSETS
    } else if version >= boundary {
        &CURRENT_FORMAL_ASSETS
    } else {
        &LEGACY_FORMAL_ASSETS
    })
}

fn manual_only_release(value: &str) -> bool {
    let Some((_, version)) = normalize_semantic_version(value) else {
        return false;
    };
    let (_, boundary) = normalize_semantic_version("0.9.9-0").expect("fixed release boundary");
    version >= boundary
}

/// 只限制新版本创建的资产；历史发布记录及下载不被新矩阵删除或改写。
#[must_use]
pub fn release_asset_is_allowed(value: &str, identity: (&str, &str, &str)) -> bool {
    let Some(canonical) = formal_release_asset_identities(value) else {
        return false;
    };
    if manual_only_release(value) {
        return MANUAL_LINUX_ASSETS.contains(&identity);
    }
    if canonical == LEGACY_FORMAL_ASSETS {
        return canonical.contains(&identity);
    }
    MANUAL_RELEASE_ASSETS.contains(&identity) || canonical.contains(&identity)
}

/// 各版本采用自己的闭合集合；缺项、多项和重复项均不能发布。
#[must_use]
pub fn release_asset_set_is_complete(value: &str, assets: &[(&str, &str, &str)]) -> bool {
    let Some(canonical) = formal_release_asset_identities(value) else {
        return false;
    };
    let matches = |expected: &[FormalReleaseAssetIdentity]| {
        assets.len() == expected.len() && expected.iter().all(|identity| assets.contains(identity))
    };
    // 手动三件/四件套与updater六件套分开，补发Linux不代表具备自动更新资格。
    if manual_only_release(value) {
        return matches(&MANUAL_ONLY_ASSETS) || matches(&MANUAL_LINUX_ASSETS);
    }
    if canonical == DESKTOP_FORMAL_ASSETS {
        return matches(canonical)
            || (assets.len() == MANUAL_RELEASE_ASSETS.len() + 1
                && MANUAL_RELEASE_ASSETS
                    .iter()
                    .all(|identity| assets.contains(identity))
                && assets.contains(&("macos", "aarch64", "tar_gz")));
    }
    matches(canonical) || (canonical == CURRENT_FORMAL_ASSETS && matches(&MANUAL_RELEASE_ASSETS))
}

/// 签名信任根的适用资产；是否属于某个版本的完整集合由上面的版本合同判定。
#[must_use]
pub fn is_desktop_updater_asset(platform: &str, kind: &str) -> bool {
    matches!(
        (platform, kind),
        ("windows", "exe" | "msi" | "zip") | ("macos", "tar_gz") | ("linux", "appimage")
    )
}

