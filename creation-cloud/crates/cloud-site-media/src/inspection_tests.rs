//! 验证 published 站点媒体五类观察、受控路径只读和巡检表边界。

use sha2::{Digest, Sha256};
use tokio::fs;
use uuid::Uuid;

use crate::{
    Service, SiteMediaInspectionStatus,
    inspection_model::PublishedMediaCandidate,
    repository::inspection::{PUBLISHED_MEDIA_SQL, UPSERT_INSPECTION_SQL},
    use_case::inspection::inspect_candidate,
};

#[test]
fn exposes_same_connection_media_inspection_entrypoint() {
    let _ = Service::inspect_published_media_with_connection;
}

#[tokio::test]
async fn classifies_all_five_states_without_mutating_media() {
    let root = tempfile::tempdir().expect("临时站点媒体根应可创建");
    let healthy_id = Uuid::now_v7();
    let healthy_key = storage_key(healthy_id);
    let healthy_path = root.path().join(&healthy_key);
    fs::create_dir_all(healthy_path.parent().expect("对象路径应有父目录"))
        .await
        .expect("对象目录应可创建");
    let bytes = b"published-media";
    fs::write(&healthy_path, bytes)
        .await
        .expect("站点媒体测试文件应可写入");
    let expected_hash = digest(bytes);

    let healthy = inspect_candidate(
        root.path(),
        candidate(healthy_id, &healthy_key, bytes.len() as i64, &expected_hash),
    )
    .await;
    assert_eq!(healthy.status, SiteMediaInspectionStatus::Healthy);
    assert_eq!(
        healthy.observed_sha256.as_deref(),
        Some(expected_hash.as_str())
    );
    assert_eq!(
        fs::read(&healthy_path).await.expect("巡检后媒体仍应存在"),
        bytes
    );

    let missing_id = Uuid::now_v7();
    let missing = inspect_candidate(
        root.path(),
        candidate(
            missing_id,
            &storage_key(missing_id),
            bytes.len() as i64,
            &expected_hash,
        ),
    )
    .await;
    assert_eq!(missing.status, SiteMediaInspectionStatus::Missing);

    let size_mismatch = inspect_candidate(
        root.path(),
        candidate(healthy_id, &healthy_key, 1, &expected_hash),
    )
    .await;
    assert_eq!(
        size_mismatch.status,
        SiteMediaInspectionStatus::SizeMismatch
    );

    let hash_mismatch = inspect_candidate(
        root.path(),
        candidate(
            healthy_id,
            &healthy_key,
            bytes.len() as i64,
            &"0".repeat(64),
        ),
    )
    .await;
    assert_eq!(
        hash_mismatch.status,
        SiteMediaInspectionStatus::HashMismatch
    );

    let io_error = inspect_candidate(
        root.path(),
        candidate(healthy_id, "../uncontrolled.png", 1, &"0".repeat(64)),
    )
    .await;
    assert_eq!(io_error.status, SiteMediaInspectionStatus::IoError);
}

#[test]
fn repository_reads_only_published_identity_and_writes_only_inspection_table() {
    assert!(PUBLISHED_MEDIA_SQL.contains("state = 'published'"));
    assert!(!PUBLISHED_MEDIA_SQL.contains("state = 'draft'"));
    assert!(!UPSERT_INSPECTION_SQL.contains("UPDATE site_media SET"));
    assert!(!UPSERT_INSPECTION_SQL.contains("DELETE FROM"));
}

#[test]
fn migration_constrains_both_inspection_tables_to_five_states() {
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

fn candidate(
    media_id: Uuid,
    storage_key: &str,
    expected_byte_size: i64,
    expected_sha256: &str,
) -> PublishedMediaCandidate {
    PublishedMediaCandidate {
        media_id,
        storage_key: storage_key.to_owned(),
        expected_byte_size,
        expected_sha256: expected_sha256.to_owned(),
    }
}

fn storage_key(id: Uuid) -> String {
    let opaque = id.to_string();
    format!("objects/{}/{}.png", &opaque[..2], opaque)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
