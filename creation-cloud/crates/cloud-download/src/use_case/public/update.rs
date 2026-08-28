//! 以已发布策略筛选目标版本和本站来源，同时保持现有 latest 响应结构兼容。

use cloud_domain::{AppError, AppResult, SemanticVersion, normalize_semantic_version};

use crate::{
    LatestUpdate, PublicAsset, Service, SourceKind, UpdateAsset, UpdateCheckQuery,
    UpdateCheckResponse, UpdateIdentityStatus, UpdateSource, repository,
};

const DOWNLOAD_PREFIX: &str = "/api/v1/downloads";
const WINDOWS_UPDATER_TRUST_EPOCH: &str = "0.8.8-0";

impl Service {
    pub async fn check_update(&self, query: UpdateCheckQuery) -> AppResult<UpdateCheckResponse> {
        let query = ValidatedQuery::try_from(query)?;
        let policy = self.public_update_policy().await?;
        let policy_revision = (policy.revision > 0)
            .then(|| u64::try_from(policy.revision))
            .transpose()
            .map_err(|_| AppError::Storage("版本策略修订超出客户端范围".into()))?;
        if !policy.enabled {
            return Ok(no_update(&query, policy_revision, policy.sha256_enabled));
        }
        let target_text = policy
            .target_version
            .as_deref()
            .ok_or_else(|| AppError::Storage("已发布版本策略缺少目标版本".into()))?;
        let (target_text, target_version) = normalize_semantic_version(target_text)
            .ok_or_else(|| AppError::Storage("已发布版本策略目标版本无效".into()))?;
        if target_version <= query.current_version {
            return Ok(no_update(&query, policy_revision, policy.sha256_enabled));
        }
        if requires_manual_windows_bridge(&query, &target_version)? {
            return Ok(no_update(&query, policy_revision, policy.sha256_enabled));
        }
        let required = policy
            .forced_versions
            .iter()
            .any(|version| version == &query.current_text);
        let identity_status = if required && policy.sha256_enabled {
            let expected = repository::policy::installed_sha256(
                &self.pool,
                &query.current_text,
                &query.platform,
                &query.architecture,
                &query.package_kind,
            )
            .await?;
            evaluate_identity(
                true,
                true,
                query.current_install_sha256.as_deref(),
                expected.as_deref(),
            )?
        } else {
            UpdateIdentityStatus::NotApplicable
        };
        let release_id = policy
            .target_release_id
            .ok_or_else(|| AppError::Storage("已发布版本策略缺少目标发布标识".into()))?;
        let release = self
            .public_manifest()
            .await?
            .into_iter()
            .find(|release| release.id == release_id)
            .ok_or_else(|| AppError::Storage("版本策略目标不在公开发布清单中".into()))?;
        if release.version != target_text || release.channel != "stable" {
            return Err(AppError::Storage("版本策略目标发布身份不一致".into()));
        }
        let assets = release
            .assets
            .into_iter()
            .filter(|asset| {
                asset.platform == query.platform
                    && asset.architecture == query.architecture
                    && asset.package_kind == query.package_kind
            })
            .map(project_asset)
            .collect::<AppResult<Vec<_>>>()?;
        if assets.is_empty() {
            return Err(AppError::Storage("版本策略目标缺少当前平台正式资产".into()));
        }
        let (title, notes) = match query.locale {
            Locale::ZhCn => (release.title_zh, release.notes_zh),
            Locale::En => (release.title_en, release.notes_en),
        };
        Ok(UpdateCheckResponse {
            update_available: true,
            current_version: query.current_text,
            latest: Some(LatestUpdate {
                version: target_text,
                channel: release.channel,
                title,
                notes,
                published_at: release.published_at,
                assets,
            }),
            required,
            policy_revision,
            sha256_enabled: policy.sha256_enabled,
            identity_status,
        })
    }
}

fn requires_manual_windows_bridge(
    query: &ValidatedQuery,
    target_version: &SemanticVersion,
) -> AppResult<bool> {
    let (_, trust_epoch) = normalize_semantic_version(WINDOWS_UPDATER_TRUST_EPOCH)
        .ok_or_else(|| AppError::Storage("Windows updater 信任根边界无效".into()))?;
    Ok(query.platform == "windows"
        && target_version >= &trust_epoch
        && (query.current_version < trust_epoch || query.package_kind == "msi"))
}

fn project_asset(mut asset: PublicAsset) -> AppResult<UpdateAsset> {
    asset
        .sources
        .retain(|source| source.source_kind == SourceKind::Local);
    if asset.sources.is_empty() {
        return Err(AppError::Storage("版本策略目标资产缺少本站下载来源".into()));
    }
    if matches!(asset.package_kind.as_str(), "exe" | "msi" | "zip") {
        let signature = asset
            .updater_signature
            .as_deref()
            .ok_or_else(|| AppError::Storage("Windows 安装资产缺少 updater signature".into()))?;
        crate::signature::validate(signature)?;
    } else if let Some(signature) = asset.updater_signature.as_deref() {
        crate::signature::validate(signature)?;
    }
    let sources = asset
        .sources
        .into_iter()
        .map(|source| UpdateSource {
            source_kind: source.source_kind,
            provider_name: source.provider_name,
            download_url: format!(
                "{DOWNLOAD_PREFIX}/assets/{}/sources/{}",
                asset.id, source.id
            ),
        })
        .collect();
    Ok(UpdateAsset {
        id: asset.id,
        architecture: asset.architecture,
        package_kind: asset.package_kind,
        file_name: asset.file_name,
        byte_size: asset.byte_size,
        sha256: asset.sha256,
        updater_signature: asset.updater_signature,
        sources,
    })
}

#[derive(Clone, Copy, Debug)]
enum Locale {
    ZhCn,
    En,
}

#[derive(Debug)]
struct ValidatedQuery {
    platform: String,
    architecture: String,
    package_kind: String,
    current_text: String,
    current_version: SemanticVersion,
    current_install_sha256: Option<String>,
    locale: Locale,
}

impl TryFrom<UpdateCheckQuery> for ValidatedQuery {
    type Error = AppError;

    fn try_from(query: UpdateCheckQuery) -> AppResult<Self> {
        if query.channel != "stable" {
            return Err(AppError::Validation("客户端更新渠道只允许 stable".into()));
        }
        let locale = match query.locale.as_str() {
            "zh-CN" => Locale::ZhCn,
            "en" => Locale::En,
            _ => return Err(AppError::Validation("locale 必须是 zh-CN 或 en".into())),
        };
        let valid_target = matches!(
            (
                query.platform.as_str(),
                query.architecture.as_str(),
                query.package_kind.as_str(),
            ),
            ("windows", "x86_64", "exe" | "msi" | "zip") | ("android", "aarch64", "apk")
        );
        if !valid_target {
            return Err(AppError::Validation("平台或架构不属于正式更新目标".into()));
        }
        let (current_text, current_version) = normalize_semantic_version(&query.current_version)
            .ok_or_else(|| AppError::Validation("current_version 必须是有效语义版本".into()))?;
        Ok(Self {
            platform: query.platform,
            architecture: query.architecture,
            package_kind: query.package_kind,
            current_text,
            current_version,
            current_install_sha256: query.current_install_sha256,
            locale,
        })
    }
}

fn no_update(
    query: &ValidatedQuery,
    policy_revision: Option<u64>,
    sha256_enabled: bool,
) -> UpdateCheckResponse {
    UpdateCheckResponse {
        update_available: false,
        current_version: query.current_text.clone(),
        latest: None,
        required: false,
        policy_revision,
        sha256_enabled,
        identity_status: UpdateIdentityStatus::NotApplicable,
    }
}

fn evaluate_identity(
    required: bool,
    sha256_enabled: bool,
    current: Option<&str>,
    expected: Option<&str>,
) -> AppResult<UpdateIdentityStatus> {
    if !required || !sha256_enabled {
        return Ok(UpdateIdentityStatus::NotApplicable);
    }
    let expected = expected
        .filter(|value| is_lower_sha256(value))
        .ok_or_else(|| AppError::Storage("强制版本安装身份证据不完整".into()))?;
    let Some(current) = current else {
        return Ok(UpdateIdentityStatus::Missing);
    };
    let current = current.trim().to_ascii_lowercase();
    if !is_lower_sha256(&current) {
        return Err(AppError::Validation(
            "current_install_sha256 必须是 64 位十六进制摘要".into(),
        ));
    }
    Ok(if current == expected {
        UpdateIdentityStatus::Verified
    } else {
        UpdateIdentityStatus::Mismatch
    })
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::PublicSource;

    fn query(platform: &str, architecture: &str, package_kind: &str) -> UpdateCheckQuery {
        UpdateCheckQuery {
            platform: platform.into(),
            architecture: architecture.into(),
            package_kind: package_kind.into(),
            current_version: "0.7.7".into(),
            current_install_sha256: None,
            channel: "stable".into(),
            locale: "zh-CN".into(),
        }
    }

    fn asset(kind: &str, signature: Option<String>) -> PublicAsset {
        let id = Uuid::now_v7();
        PublicAsset {
            id,
            platform: "windows".into(),
            architecture: "x86_64".into(),
            package_kind: kind.into(),
            file_name: "asset.bin".into(),
            byte_size: 1,
            sha256: "a".repeat(64),
            updater_signature: signature,
            sources: vec![PublicSource {
                id: Uuid::now_v7(),
                source_kind: SourceKind::Local,
                provider_name: "本站".into(),
                sort_order: 0,
                download_path: "not-exposed".into(),
            }],
        }
    }

    #[test]
    fn accepts_only_formal_platform_architecture_pairs() {
        assert!(ValidatedQuery::try_from(query("windows", "x86_64", "exe")).is_ok());
        assert!(ValidatedQuery::try_from(query("windows", "x86_64", "msi")).is_ok());
        assert!(ValidatedQuery::try_from(query("windows", "x86_64", "zip")).is_ok());
        assert!(ValidatedQuery::try_from(query("android", "aarch64", "apk")).is_ok());
        assert!(ValidatedQuery::try_from(query("linux", "x86_64", "exe")).is_err());
        assert!(ValidatedQuery::try_from(query("android", "aarch64", "zip")).is_err());
    }

    #[test]
    fn signing_epoch_suppresses_only_legacy_windows_and_msi_queries() {
        fn check(
            platform: &str,
            package_kind: &str,
            current_version: &str,
            target_version: &str,
        ) -> bool {
            let mut input = query(
                platform,
                if platform == "windows" {
                    "x86_64"
                } else {
                    "aarch64"
                },
                package_kind,
            );
            input.current_version = current_version.into();
            let query = ValidatedQuery::try_from(input).unwrap();
            let (_, target_version) = normalize_semantic_version(target_version).unwrap();
            requires_manual_windows_bridge(&query, &target_version).unwrap()
        }

        assert!(check("windows", "exe", "0.8.7", "0.8.8"));
        assert!(check("windows", "zip", "0.8.6", "0.8.9"));
        assert!(check("windows", "msi", "0.8.8", "0.8.9"));
        assert!(!check("windows", "msi", "0.8.6", "0.8.7"));
        assert!(!check("windows", "exe", "0.8.8", "0.8.9"));
        assert!(!check("android", "apk", "0.8.7", "0.8.8"));
    }

    #[test]
    fn manual_bridge_uses_the_exact_legacy_no_update_shape() {
        let mut input = query("windows", "x86_64", "exe");
        input.current_version = "0.8.7".into();
        let query = ValidatedQuery::try_from(input).unwrap();
        let value = serde_json::to_value(no_update(&query, Some(35), true)).unwrap();
        let object = value.as_object().unwrap();

        assert_eq!(object.len(), 7);
        for key in [
            "update_available",
            "current_version",
            "latest",
            "required",
            "policy_revision",
            "sha256_enabled",
            "identity_status",
        ] {
            assert!(object.contains_key(key), "旧客户端响应缺少字段 {key}");
        }
        assert_eq!(value["update_available"], false);
        assert_eq!(value["current_version"], "0.8.7");
        assert!(value["latest"].is_null());
        assert_eq!(value["required"], false);
        assert_eq!(value["policy_revision"], 35);
        assert_eq!(value["sha256_enabled"], true);
        assert_eq!(value["identity_status"], "not_applicable");
    }

    #[test]
    fn updater_signatures_are_required_for_all_windows_update_shapes() {
        assert!(project_asset(asset("exe", None)).is_err());
        assert!(project_asset(asset("msi", Some("b".repeat(64)))).is_ok());
        assert!(project_asset(asset("zip", None)).is_err());
        assert!(project_asset(asset("zip", Some("b".repeat(64)))).is_ok());
    }

    #[test]
    fn projected_sources_are_always_cloud_download_endpoints() {
        let projected = project_asset(asset("zip", Some("b".repeat(64))))
            .expect("ZIP 必须携带 updater signature");
        assert!(
            projected.sources[0]
                .download_url
                .starts_with("/api/v1/downloads/assets/")
        );
    }

    #[test]
    fn identity_is_consumed_only_for_forced_sha_policy() {
        let expected = "a".repeat(64);
        assert_eq!(
            evaluate_identity(true, true, Some(&expected), Some(&expected)).unwrap(),
            UpdateIdentityStatus::Verified
        );
        assert_eq!(
            evaluate_identity(true, true, None, Some(&expected)).unwrap(),
            UpdateIdentityStatus::Missing
        );
        assert_eq!(
            evaluate_identity(true, true, Some(&"b".repeat(64)), Some(&expected)).unwrap(),
            UpdateIdentityStatus::Mismatch
        );
        assert_eq!(
            evaluate_identity(true, false, Some("not-a-sha"), None).unwrap(),
            UpdateIdentityStatus::NotApplicable
        );
        assert_eq!(
            evaluate_identity(false, true, Some("not-a-sha"), None).unwrap(),
            UpdateIdentityStatus::NotApplicable
        );
        assert!(evaluate_identity(true, true, Some("not-a-sha"), Some(&expected)).is_err());
        assert!(evaluate_identity(true, true, Some(&expected), None).is_err());
    }
}
