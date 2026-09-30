//! 验证本站发布资产五类观察、文件只读边界和外链零查询路径。

use sha2::{Digest, Sha256};
use tokio::fs;
use uuid::Uuid;

use crate::{
    AssetInspectionStatus, Service,
    file_verification::{FileVerifier, InspectionVerifier},
    model::PublishedLocalAsset,
    repository::inspection::{PUBLISHED_LOCAL_ASSETS_SQL, UPSERT_INSPECTION_SQL},
    use_case::inspection::inspect_candidate,
};

#[test]
fn exposes_same_connection_asset_inspection_entrypoint() {
    let _ = Service::inspect_published_local_assets_with_connection;
}

#[tokio::test]
async fn classifies_all_five_states_without_mutating_files() {
    let root = tempfile::tempdir().expect("临时下载根应可创建");
    fs::create_dir_all(root.path().join("objects/directory"))
        .await
        .expect("测试目录应可创建");
    let healthy_bytes = b"healthy-asset";
    fs::write(root.path().join("objects/healthy"), healthy_bytes)
        .await
        .expect("健康文件应可写入");
    fs::write(root.path().join("objects/wrong-size"), b"short")
        .await
        .expect("大小异常文件应可写入");
    fs::write(root.path().join("objects/wrong-hash"), b"changed-asset")
        .await
        .expect("摘要异常文件应可写入");
    let verifier = InspectionVerifier::default();
    let expected_hash = digest(healthy_bytes);

    let healthy = inspect_candidate(
        root.path(),
        &verifier,
        candidate(
            "objects/healthy",
            healthy_bytes.len() as i64,
            &expected_hash,
        ),
    )
    .await;
    assert_eq!(healthy.status, AssetInspectionStatus::Healthy);
    assert_eq!(
        healthy.observed_sha256.as_deref(),
        Some(expected_hash.as_str())
    );
    assert_eq!(
        fs::read(root.path().join("objects/healthy"))
            .await
            .expect("巡检后文件仍应可读"),
        healthy_bytes
    );

    let missing = inspect_candidate(
        root.path(),
        &verifier,
        candidate("objects/missing", 1, &"0".repeat(64)),
    )
    .await;
    assert_eq!(missing.status, AssetInspectionStatus::Missing);

    let size_mismatch = inspect_candidate(
        root.path(),
        &verifier,
        candidate("objects/wrong-size", 99, &"0".repeat(64)),
    )
    .await;
    assert_eq!(size_mismatch.status, AssetInspectionStatus::SizeMismatch);

    let hash_mismatch = inspect_candidate(
        root.path(),
        &verifier,
        candidate(
            "objects/wrong-hash",
            b"changed-asset".len() as i64,
            &expected_hash,
        ),
    )
    .await;
    assert_eq!(hash_mismatch.status, AssetInspectionStatus::HashMismatch);

    let io_error = inspect_candidate(
        root.path(),
        &verifier,
        candidate("objects/directory", 0, &"0".repeat(64)),
    )
    .await;
    assert_eq!(io_error.status, AssetInspectionStatus::IoError);
}

#[tokio::test]
async fn inspection_bypasses_online_download_cache() {
    let root = tempfile::tempdir().expect("临时下载根应可创建");
    fs::create_dir_all(root.path().join("objects"))
        .await
        .expect("对象目录应可创建");
    let path = root.path().join("objects/cached");
    let original = b"trusted";
    let changed = b"changed";
    let expected_hash = digest(original);
    fs::write(&path, original).await.expect("原始文件应可写入");

    let online = FileVerifier::default();
    let mut online_file = fs::File::open(&path).await.expect("在线文件应可打开");
    online
        .verify(
            &path,
            &mut online_file,
            original.len() as u64,
            &expected_hash,
        )
        .await
        .expect("在线校验应写入正向缓存");
    drop(online_file);
    fs::write(&path, changed).await.expect("同长度变更应可写入");

    let inspection = inspect_candidate(
        root.path(),
        &InspectionVerifier::default(),
        candidate("objects/cached", changed.len() as i64, &expected_hash),
    )
    .await;
    assert_eq!(inspection.status, AssetInspectionStatus::HashMismatch);
    let changed_hash = digest(changed);
    assert_eq!(
        inspection.observed_sha256.as_deref(),
        Some(changed_hash.as_str())
    );
}

#[test]
fn candidate_query_has_no_external_url_or_network_path() {
    assert!(PUBLISHED_LOCAL_ASSETS_SQL.contains("release.status = 'published'"));
    assert!(PUBLISHED_LOCAL_ASSETS_SQL.contains("source.enabled"));
    assert!(PUBLISHED_LOCAL_ASSETS_SQL.contains("source.source_kind = 'local'"));
    assert!(!PUBLISHED_LOCAL_ASSETS_SQL.contains("external_url"));
    assert!(!PUBLISHED_LOCAL_ASSETS_SQL.contains("http"));
    assert!(!UPSERT_INSPECTION_SQL.contains("UPDATE releases SET"));
    assert!(!UPSERT_INSPECTION_SQL.contains("UPDATE release_sources SET"));
    assert!(!UPSERT_INSPECTION_SQL.contains("DELETE FROM"));
}

#[test]
fn inspection_migration_allows_only_fixed_observation_states() {
    let migration = include_str!("../../../migrations/0022_published_asset_inspection.sql");
    for status in [
        "healthy",
        "missing",
        "size_mismatch",
        "hash_mismatch",
        "io_error",
    ] {
        assert!(migration.contains(status));
    }
    assert!(migration.contains("release_source_inspections"));
    assert!(migration.contains("site_media_inspections"));
}

fn candidate(path: &str, expected_byte_size: i64, expected_sha256: &str) -> PublishedLocalAsset {
    PublishedLocalAsset {
        source_id: Uuid::now_v7(),
        local_path: path.to_owned(),
        expected_byte_size,
        expected_sha256: expected_sha256.to_owned(),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
