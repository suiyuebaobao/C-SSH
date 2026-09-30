//! 覆盖受控不透明路径、隔离写入、原子落位、身份复核和可恢复删除。

use sha2::Digest;
use tokio::fs;

use crate::storage::{controlled_path, quarantine_delete, read_verified, readiness_probe, stage};

#[tokio::test]
async fn staged_file_commits_and_reads_only_with_matching_identity() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    let bytes = b"verified-png-placeholder";
    let sha256 = hex::encode(sha2::Sha256::digest(bytes));
    let mut staged = stage(root.path(), bytes).await.expect("隔离写入应成功");
    let key = staged.storage_key.clone();
    staged.commit().await.expect("原子落位应成功");

    let loaded = read_verified(
        root.path(),
        &key,
        i64::try_from(bytes.len()).expect("测试长度应可表示"),
        &sha256,
    )
    .await
    .expect("文件身份匹配时应可读取");
    assert_eq!(loaded, bytes);
    assert!(read_verified(root.path(), &key, 1, &sha256).await.is_err());
}

#[tokio::test]
async fn failed_commit_never_removes_an_existing_destination() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    let mut staged = stage(root.path(), b"new").await.expect("隔离写入应成功");
    let target = controlled_path(root.path(), &staged.storage_key).expect("生成路径应受控");
    fs::write(&target, b"existing")
        .await
        .expect("冲突目标应可创建");

    assert!(staged.commit().await.is_err());
    staged.cleanup().await;
    assert_eq!(
        fs::read(target).await.expect("旧目标不应被清理"),
        b"existing"
    );
}

#[tokio::test]
async fn readiness_uses_real_layout_and_leaves_no_probe_file() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    readiness_probe(root.path())
        .await
        .expect("真实站点媒体布局就绪探针应通过");
    assert!(
        fs::read_dir(root.path().join("quarantine"))
            .await
            .expect("应读取隔离目录")
            .next_entry()
            .await
            .expect("应读取隔离目录首项")
            .is_none()
    );
    let mut object_prefixes = fs::read_dir(root.path().join("objects"))
        .await
        .expect("应读取对象目录");
    while let Some(prefix) = object_prefixes
        .next_entry()
        .await
        .expect("应读取对象分片目录")
    {
        assert!(prefix.file_type().await.expect("应读取分片类型").is_dir());
        assert!(
            fs::read_dir(prefix.path())
                .await
                .expect("应读取对象分片")
                .next_entry()
                .await
                .expect("应读取对象分片首项")
                .is_none()
        );
    }
}

#[tokio::test]
async fn delete_quarantine_can_restore_or_finish() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    let mut staged = stage(root.path(), b"delete-me")
        .await
        .expect("隔离写入应成功");
    let key = staged.storage_key.clone();
    let target = controlled_path(root.path(), &key).expect("生成路径应受控");
    staged.commit().await.expect("原子落位应成功");

    let mut deleted = quarantine_delete(root.path(), &key)
        .await
        .expect("删除隔离应成功");
    assert!(!target.exists());
    deleted.restore().await.expect("删除回滚应成功");
    assert!(target.exists());

    let deleted = quarantine_delete(root.path(), &key)
        .await
        .expect("第二次删除隔离应成功");
    deleted.finish().await.expect("删除清理应成功");
    assert!(!target.exists());
}

#[tokio::test]
async fn dropping_armed_guards_cleans_or_restores_only_current_files() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    let staged = stage(root.path(), b"cancel-before-commit")
        .await
        .expect("隔离写入应成功");
    let uncommitted_target =
        controlled_path(root.path(), &staged.storage_key).expect("生成路径应受控");
    drop(staged);
    assert!(!uncommitted_target.exists());

    let mut staged = stage(root.path(), b"cancel-after-commit")
        .await
        .expect("隔离写入应成功");
    let key = staged.storage_key.clone();
    let committed_target = controlled_path(root.path(), &key).expect("生成路径应受控");
    staged.commit().await.expect("原子落位应成功");
    drop(staged);
    assert!(!committed_target.exists());

    let mut staged = stage(root.path(), b"cancel-delete")
        .await
        .expect("隔离写入应成功");
    let key = staged.storage_key.clone();
    let restored_target = controlled_path(root.path(), &key).expect("生成路径应受控");
    staged.commit().await.expect("原子落位应成功");
    staged.disarm();
    let deleted = quarantine_delete(root.path(), &key)
        .await
        .expect("删除隔离应成功");
    drop(deleted);
    assert!(restored_target.exists());
}

#[tokio::test]
async fn finalization_handoff_preserves_files_until_database_identity_is_known() {
    let root = tempfile::tempdir().expect("临时站点媒体目录应可创建");
    let mut staged = stage(root.path(), b"preserve-create")
        .await
        .expect("隔离写入应成功");
    let key = staged.storage_key.clone();
    let target = controlled_path(root.path(), &key).expect("生成路径应受控");
    staged.commit().await.expect("原子落位应成功");
    staged.preserve_on_drop();
    drop(staged);
    assert!(target.exists(), "未核对数据库身份前不得删除已落位文件");

    let mut deleted = quarantine_delete(root.path(), &key)
        .await
        .expect("删除隔离应成功");
    deleted.preserve_on_drop();
    drop(deleted);
    assert!(!target.exists(), "未核对数据库身份前不得猜测恢复原路径");
    assert!(
        fs::read_dir(root.path().join("quarantine"))
            .await
            .expect("应读取隔离目录")
            .next_entry()
            .await
            .expect("应读取隔离目录首项")
            .is_some(),
        "未核对数据库身份前应保留删除隔离文件"
    );
}

#[test]
fn controlled_path_rejects_traversal_and_malformed_keys() {
    let root = std::path::Path::new("site-media-test");
    for key in [
        "../secret.png",
        "objects/AA/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png",
        "objects/aa/bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb.png",
        "objects/aa/not-a-uuid.png",
        "objects/aa/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.svg",
    ] {
        assert!(controlled_path(root, key).is_err(), "非法键不应通过: {key}");
    }
}
