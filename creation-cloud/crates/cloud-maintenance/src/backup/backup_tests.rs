//! 验证备份清单只读检查的健康、缺失、陈旧与格式边界。

use super::*;

async fn write_manifest(root: &Path, completed_at: DateTime<Utc>, declared_size: u64) {
    fs::write(root.join("database.dump"), b"database")
        .await
        .expect("数据库备份测试文件应可写入");
    fs::write(root.join("release.tar"), b"release")
        .await
        .expect("发布元数据测试文件应可写入");
    let manifest = BackupManifest {
        schema_version: 2,
        completed_at,
        database: BackupFileDeclaration {
            file_name: "database.dump".to_owned(),
            byte_size: declared_size,
            sha256: "aa".repeat(32),
        },
        release_bundle: BackupFileDeclaration {
            file_name: "release.tar".to_owned(),
            byte_size: 7,
            sha256: "bb".repeat(32),
        },
    };
    fs::write(
        root.join(MANIFEST_NAME),
        serde_json::to_vec(&manifest).expect("测试清单应可编码"),
    )
    .await
    .expect("测试清单应可写入");
}

#[tokio::test]
async fn valid_manifest_is_healthy_without_mutating_files() {
    let root = tempfile::tempdir().expect("临时备份目录应可创建");
    let now = Utc::now();
    write_manifest(root.path(), now, 8).await;
    let before = fs::metadata(root.path().join("database.dump"))
        .await
        .expect("测试文件应存在");
    let result = check_latest_backup(root.path(), Duration::from_secs(26 * 3600), now).await;
    let after = fs::metadata(root.path().join("database.dump"))
        .await
        .expect("测试文件应继续存在");
    assert_eq!(result.observation, ObservationCode::Healthy);
    assert_eq!(result.checked_file_count, 2);
    assert!(same_file_identity(&before, &after));
}

#[tokio::test]
async fn missing_root_is_observed_without_creation() {
    let parent = tempfile::tempdir().expect("临时父目录应可创建");
    let missing = parent.path().join("missing");
    let result = check_latest_backup(&missing, Duration::from_secs(60), Utc::now()).await;
    assert_eq!(result.observation, ObservationCode::Missing);
    assert!(!missing.exists());
}

#[tokio::test]
async fn stale_and_size_mismatch_are_distinct_observations() {
    let stale = tempfile::tempdir().expect("临时备份目录应可创建");
    let now = Utc::now();
    write_manifest(stale.path(), now - chrono::Duration::hours(27), 8).await;
    let stale_result = check_latest_backup(stale.path(), Duration::from_secs(26 * 3600), now).await;
    assert_eq!(stale_result.observation, ObservationCode::Stale);

    let invalid = tempfile::tempdir().expect("临时备份目录应可创建");
    write_manifest(invalid.path(), now, 9).await;
    let invalid_result =
        check_latest_backup(invalid.path(), Duration::from_secs(26 * 3600), now).await;
    assert_eq!(invalid_result.observation, ObservationCode::Invalid);
}

#[tokio::test]
async fn manifest_requires_two_distinct_payload_files() {
    let root = tempfile::tempdir().expect("临时备份目录应可创建");
    let now = Utc::now();
    write_manifest(root.path(), now, 8).await;
    let mut manifest: BackupManifest = serde_json::from_slice(
        &fs::read(root.path().join(MANIFEST_NAME))
            .await
            .expect("测试清单应可读取"),
    )
    .expect("测试清单应可解析");
    manifest.release_bundle = manifest.database.clone();
    fs::write(
        root.path().join(MANIFEST_NAME),
        serde_json::to_vec(&manifest).expect("测试清单应可编码"),
    )
    .await
    .expect("测试清单应可覆盖");

    let checked = check_latest_backup(root.path(), Duration::from_secs(60), now).await;
    assert_eq!(checked.observation, ObservationCode::Invalid);
}

#[test]
fn backup_file_names_are_single_bounded_ascii_components() {
    for valid in ["database.dump", "release-bundle-1.tar", "A_1"] {
        assert!(valid_direct_file_name(valid));
    }
    for invalid in [
        "",
        ".",
        "..",
        "../backup",
        "dir/file",
        "dir\\file",
        "C:backup",
    ] {
        assert!(!valid_direct_file_name(invalid));
    }
}

#[tokio::test]
async fn old_schema_and_invalid_digest_are_rejected() {
    let root = tempfile::tempdir().expect("临时备份目录应可创建");
    let now = Utc::now();
    write_manifest(root.path(), now, 8).await;
    let manifest_path = root.path().join(MANIFEST_NAME);
    let mut manifest: BackupManifest =
        serde_json::from_slice(&fs::read(&manifest_path).await.expect("测试清单应可读取"))
            .expect("测试清单应可解析");
    manifest.schema_version = 1;
    fs::write(
        &manifest_path,
        serde_json::to_vec(&manifest).expect("测试清单应可编码"),
    )
    .await
    .expect("测试清单应可覆盖");
    assert_eq!(
        check_latest_backup(root.path(), Duration::from_secs(60), now)
            .await
            .observation,
        ObservationCode::Invalid
    );

    manifest.schema_version = 2;
    manifest.database.sha256 = "A".repeat(64);
    fs::write(
        &manifest_path,
        serde_json::to_vec(&manifest).expect("测试清单应可编码"),
    )
    .await
    .expect("测试清单应可覆盖");
    assert_eq!(
        check_latest_backup(root.path(), Duration::from_secs(60), now)
            .await
            .observation,
        ObservationCode::Invalid
    );
}

#[tokio::test]
async fn future_timestamp_and_unknown_fields_are_rejected() {
    let root = tempfile::tempdir().expect("临时备份目录应可创建");
    let now = Utc::now();
    write_manifest(root.path(), now + chrono::Duration::seconds(1), 8).await;
    assert_eq!(
        check_latest_backup(root.path(), Duration::from_secs(60), now)
            .await
            .observation,
        ObservationCode::Invalid
    );

    write_manifest(root.path(), now, 8).await;
    let manifest_path = root.path().join(MANIFEST_NAME);
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).await.expect("测试清单应可读取"))
            .expect("测试清单应可解析");
    value
        .as_object_mut()
        .expect("清单应为对象")
        .insert("unexpected".to_owned(), serde_json::Value::Bool(true));
    fs::write(
        &manifest_path,
        serde_json::to_vec(&value).expect("测试清单应可编码"),
    )
    .await
    .expect("测试清单应可覆盖");
    assert_eq!(
        check_latest_backup(root.path(), Duration::from_secs(60), now)
            .await
            .observation,
        ObservationCode::Invalid
    );
}
