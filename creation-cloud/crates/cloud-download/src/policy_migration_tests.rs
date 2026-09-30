//! 锁定更新签名、追加式策略发布和单一公开指针的数据库合同。

const MIGRATION: &str = include_str!("../../../migrations/0049_update_policy_and_signatures.sql");
const INSTALLED_IDENTITY_MIGRATION: &str =
    include_str!("../../../migrations/0052_release_installed_identity.sql");
const SIGNATURE_SCOPE_MIGRATION: &str =
    include_str!("../../../migrations/0053_windows_updater_signature_scope.sql");
const NO_UPDATE_MIGRATION: &str = include_str!("../../../migrations/0058_no_update_versions.sql");

#[test]
fn update_policy_publications_are_append_only_and_signatures_gate_installers() {
    for contract in [
        "ADD COLUMN updater_signature TEXT",
        "asset.package_kind IN ('exe', 'msi')",
        "CREATE TABLE update_policy_draft",
        "CREATE TABLE update_policy_publications",
        "CREATE TABLE update_policy_publication_state",
        "published update policies are immutable",
        "current_revision BIGINT REFERENCES update_policy_publications(revision)",
        "update policy publication revision must advance",
    ] {
        assert!(
            MIGRATION.contains(contract),
            "版本策略迁移缺少契约：{contract}"
        );
    }
}

#[test]
fn updater_signature_scope_migration_is_exact_and_permanent() {
    for contract in [
        "LOCK TABLE release_assets IN ACCESS EXCLUSIVE MODE",
        "cleans_non_windows_signature",
        "SET updater_signature = NULL",
        "WHERE updater_signature IS NOT NULL",
        "release_assets_updater_signature_scope_check",
        "non-windows assets cannot carry updater signatures",
    ] {
        assert!(
            SIGNATURE_SCOPE_MIGRATION.contains(contract),
            "0053 签名范围迁移缺少契约：{contract}"
        );
    }
    assert!(!SIGNATURE_SCOPE_MIGRATION.contains("DELETE FROM"));
    assert!(!SIGNATURE_SCOPE_MIGRATION.contains("UPDATE releases"));
    assert!(!SIGNATURE_SCOPE_MIGRATION.contains("UPDATE release_sources"));
    assert!(!SIGNATURE_SCOPE_MIGRATION.contains("DISABLE TRIGGER"));
}

#[test]
fn portable_signature_and_installed_identity_are_forward_only_extensions() {
    for contract in [
        "asset.package_kind IN ('exe', 'msi', 'zip')",
        "ADD COLUMN installed_sha256 TEXT",
        "installed identity evidence is immutable",
    ] {
        assert!(
            INSTALLED_IDENTITY_MIGRATION.contains(contract),
            "0052 迁移缺少契约：{contract}"
        );
    }
}

#[test]
fn no_update_versions_extend_draft_and_publications_with_pairwise_guards() {
    for contract in [
        "ALTER TABLE update_policy_draft",
        "ALTER TABLE update_policy_publications",
        "ADD COLUMN no_update_versions TEXT[] NOT NULL DEFAULT '{}'",
        "cardinality(no_update_versions) <= 128",
        "forced_versions && no_update_versions",
        "disabled_versions && no_update_versions",
    ] {
        assert!(
            NO_UPDATE_MIGRATION.contains(contract),
            "0058 无需更新迁移缺少契约：{contract}"
        );
    }
    assert!(!NO_UPDATE_MIGRATION.contains("UPDATE update_policy_publications"));
    assert!(!NO_UPDATE_MIGRATION.contains("DELETE FROM"));
}
