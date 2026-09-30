//! 验证策略版本与正式资产的边界。

#[cfg(test)]
mod tests {
    use super::super::policy_validation::*;
    use crate::{
        SaveUpdatePolicyDraftInput,
        model::{ForcedIdentityRow, PolicyAssetRow},
    };
    use uuid::Uuid;

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
    fn no_update_versions_are_semver_normalized_deduplicated_and_bounded() {
        let input = validate_draft(SaveUpdatePolicyDraftInput {
            expected_revision: 0,
            enabled: false,
            forced_versions: Vec::new(),
            disabled_versions: None,
            no_update_versions: Some(vec!["v0.8.9".into(), "0.8.9".into(), "0.8.8".into()]),
            target_release_id: None,
            sha256_enabled: true,
        })
        .unwrap();
        assert_eq!(
            input.no_update_versions.unwrap(),
            vec!["0.8.8".to_owned(), "0.8.9".to_owned()]
        );

        let mut too_many = Vec::new();
        for index in 0..129 {
            too_many.push(format!("0.8.{index}"));
        }
        let input = SaveUpdatePolicyDraftInput {
            expected_revision: 0,
            enabled: false,
            forced_versions: Vec::new(),
            disabled_versions: None,
            no_update_versions: Some(too_many),
            target_release_id: None,
            sha256_enabled: true,
        };
        assert!(validate_draft(input).is_err());
    }

    #[test]
    fn policy_version_lists_are_pairwise_disjoint() {
        assert!(validate_policy_version_lists(&["0.8.7".into()], &["0.8.7".into()], &[]).is_err());
        assert!(validate_policy_version_lists(&["0.8.7".into()], &[], &["0.8.7".into()]).is_err());
        assert!(validate_policy_version_lists(&[], &["0.8.7".into()], &["0.8.7".into()]).is_err());
        assert!(
            validate_policy_version_lists(&["0.8.6".into()], &["0.8.7".into()], &["0.8.8".into()])
                .is_ok()
        );
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
