//! 覆盖上传大小、SHA256 与失败清理边界。

use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::upload_file::{
    CleanupFile, MAX_ASSET_BYTES, UploadIdentity, UploadLayout, readiness_probe,
};

#[test]
fn accepts_exact_asset_identity() {
    let bytes = b"creation-cloud-upload";
    let hash = format!("{:x}", Sha256::digest(bytes));
    let mut identity =
        UploadIdentity::new(bytes.len() as u64, &hash).expect("合法资产身份应能开始上传");
    identity.observe(&bytes[..8]).expect("首块应被接受");
    identity.observe(&bytes[8..]).expect("末块应被接受");
    identity.finish().expect("大小与哈希一致时应通过");
}

#[test]
fn rejects_oversized_or_mismatched_upload() {
    assert!(UploadIdentity::new(MAX_ASSET_BYTES + 1, &"a".repeat(64)).is_err());

    let mut identity =
        UploadIdentity::new(3, &"a".repeat(64)).expect("格式合法的身份应进入流式校验");
    assert!(identity.observe(b"four").is_err());
}

#[test]
fn bounds_provider_name() {
    assert!(crate::validation::required_text(&"a".repeat(101), "来源名称", 100).is_err());
}

#[test]
fn cleanup_guard_removes_task_owned_file() {
    let path = std::env::temp_dir().join(format!("cloud-download-{}.part", Uuid::now_v7()));
    std::fs::write(&path, b"temporary").expect("测试临时文件应可写入");
    drop(CleanupFile::new(path.clone()));
    assert!(!path.exists());
}

#[tokio::test]
async fn promotes_to_opaque_path_and_cleans_on_drop() {
    let root = std::env::temp_dir().join(format!("cloud-download-root-{}", Uuid::now_v7()));
    let layout = UploadLayout::prepare(&root)
        .await
        .expect("隔离下载根应可初始化");
    let object_id = Uuid::now_v7();
    let temp_path = layout.temp_path(object_id);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .await
        .expect("隔离临时文件应可创建");
    file.write_all(b"asset")
        .await
        .expect("隔离临时文件应可写入");
    drop(file);

    let mut cleanup = CleanupFile::new(temp_path.clone());
    let (final_path, relative) = layout
        .promote(&temp_path, object_id, &mut cleanup)
        .await
        .expect("临时文件应原子提交");
    assert_eq!(relative, format!("objects/{object_id}"));
    let canonical_root = tokio::fs::canonicalize(&root)
        .await
        .expect("隔离下载根应可规范化");
    assert!(final_path.starts_with(canonical_root));
    drop(cleanup);
    assert!(!final_path.exists());
    tokio::fs::remove_dir_all(&root)
        .await
        .expect("隔离下载根应可清理");
}

#[tokio::test]
async fn refuses_to_overwrite_existing_object() {
    let root = std::env::temp_dir().join(format!("cloud-download-root-{}", Uuid::now_v7()));
    let layout = UploadLayout::prepare(&root)
        .await
        .expect("隔离下载根应可初始化");
    let object_id = Uuid::now_v7();
    let final_path = root.join("objects").join(object_id.to_string());
    tokio::fs::write(&final_path, b"existing")
        .await
        .expect("冲突目标应可准备");
    let temp_path = layout.temp_path(object_id);
    tokio::fs::write(&temp_path, b"new")
        .await
        .expect("隔离临时文件应可准备");
    let mut cleanup = CleanupFile::new(temp_path.clone());
    assert!(
        layout
            .promote(&temp_path, object_id, &mut cleanup)
            .await
            .is_err()
    );
    assert_eq!(
        tokio::fs::read(&final_path)
            .await
            .expect("冲突目标应保持可读"),
        b"existing"
    );
    drop(cleanup);
    tokio::fs::remove_dir_all(&root)
        .await
        .expect("隔离下载根应可清理");
}

#[tokio::test]
async fn readiness_uses_real_quarantine_and_object_paths_without_residue() {
    let root = std::env::temp_dir().join(format!("cloud-download-ready-{}", Uuid::now_v7()));
    readiness_probe(&root)
        .await
        .expect("真实下载布局就绪探针应通过");
    for child in ["quarantine", "objects"] {
        assert!(
            tokio::fs::read_dir(root.join(child))
                .await
                .expect("应读取就绪探针目录")
                .next_entry()
                .await
                .expect("应读取目录首项")
                .is_none()
        );
    }
    tokio::fs::remove_dir_all(&root)
        .await
        .expect("应清理下载就绪探针目录");
}
