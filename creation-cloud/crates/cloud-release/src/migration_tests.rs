//! 锁定版本发布前每个资产都必须拥有可用下载来源的数据库门禁。

const RELEASE_PUBLISH_INTEGRITY: &str =
    include_str!("../../../migrations/0009_release_publish_integrity.sql");
const RELEASE_SOURCE_DELETE_RESTRICT: &str =
    include_str!("../../../migrations/0010_release_source_delete_restrict.sql");
const RELEASE_INSTALLED_IDENTITY: &str =
    include_str!("../../../migrations/0052_release_installed_identity.sql");
const WINDOWS_SIGNATURE_SCOPE: &str =
    include_str!("../../../migrations/0053_windows_updater_signature_scope.sql");

#[test]
fn publish_guard_requires_an_enabled_source_for_every_asset() {
    for contract in [
        "OLD.status = 'validating' AND NEW.status = 'published'",
        "release requires at least one asset",
        "every release asset requires an enabled source",
        "source.asset_id = asset.id AND source.enabled",
        "CREATE OR REPLACE FUNCTION guard_release_mutation()",
        "local source asset identity is immutable",
        "WHERE asset_id = OLD.id AND source_kind = 'local'",
        "FOR UPDATE OF releases",
        "CREATE OR REPLACE FUNCTION guard_release_source_mutation()",
    ] {
        assert!(
            RELEASE_PUBLISH_INTEGRITY.contains(contract),
            "版本发布完整性迁移缺少契约：{contract}"
        );
    }
}

#[test]
fn updater_signature_scope_blocks_non_windows_publication_metadata() {
    for contract in [
        "release_assets_updater_signature_scope_check",
        "asset.platform = 'windows'",
        "asset.package_kind IN ('exe', 'msi', 'zip')",
        "asset.updater_signature IS NOT NULL",
        "non-windows assets cannot carry updater signatures",
    ] {
        assert!(
            WINDOWS_SIGNATURE_SCOPE.contains(contract),
            "0053 发布签名范围迁移缺少契约：{contract}"
        );
    }
}

#[test]
fn asset_and_release_delete_cannot_cascade_over_sources() {
    for contract in [
        "DROP CONSTRAINT release_sources_asset_id_fkey",
        "FOREIGN KEY (asset_id) REFERENCES release_assets(id) ON DELETE RESTRICT",
        "NEW.release_id IS DISTINCT FROM OLD.release_id",
        "CREATE TRIGGER release_assets_parent_identity_guard",
        "NEW.asset_id IS DISTINCT FROM OLD.asset_id",
        "CREATE TRIGGER release_sources_parent_identity_guard",
    ] {
        assert!(
            RELEASE_SOURCE_DELETE_RESTRICT.contains(contract),
            "来源删除约束迁移缺少契约：{contract}"
        );
    }
    assert!(!RELEASE_SOURCE_DELETE_RESTRICT.contains("ON DELETE CASCADE"));
}

#[test]
fn installed_identity_is_nullable_for_backfill_then_permanently_immutable() {
    for contract in [
        "ADD COLUMN installed_sha256 TEXT",
        "asset.package_kind IN ('exe', 'msi', 'zip')",
        "windows update assets require updater signatures",
        "installed_sha256 ~ '^[0-9a-f]{64}$'",
        "installed identity evidence must be recorded separately",
        "installed identity evidence is immutable",
        "records_windows_signature",
        "asset identity with installed evidence is immutable",
        "SET installed_sha256 = sha256",
        "platform = 'android'",
        "package_kind = 'apk'",
    ] {
        assert!(
            RELEASE_INSTALLED_IDENTITY.contains(contract),
            "安装身份迁移缺少契约：{contract}"
        );
    }
    assert!(!RELEASE_INSTALLED_IDENTITY.contains("installed_sha256 TEXT NOT NULL"));
}
