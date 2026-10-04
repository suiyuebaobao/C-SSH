//! 只在媒体专属目录内生成不可覆盖文件，并按记录身份读取及清理。

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

pub(crate) struct StagedAsset {
    pub temp: NamedTempFile,
    pub bytes: u64,
    pub sha256: String,
}

pub(crate) async fn prepare(base: &Path) -> AppResult<PathBuf> {
    fs::create_dir_all(base).await.map_err(io_error)?;
    let base_meta = fs::symlink_metadata(base).await.map_err(io_error)?;
    if !base_meta.is_dir() || base_meta.file_type().is_symlink() {
        return Err(AppError::Storage("媒体根目录身份无效".into()));
    }
    let base = fs::canonicalize(base).await.map_err(io_error)?;
    let root = base.join("tutorials");
    match fs::create_dir(&root).await {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => return Err(io_error(e)),
    }
    let meta = fs::symlink_metadata(&root).await.map_err(io_error)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(AppError::Storage("教学媒体目录不能是链接".into()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
            .await
            .map_err(io_error)?;
    }
    if fs::canonicalize(&root).await.map_err(io_error)?.parent() != Some(base.as_path()) {
        return Err(AppError::Storage("教学媒体目录越界".into()));
    }
    Ok(root)
}

pub(crate) fn temporary(root: &Path) -> AppResult<NamedTempFile> {
    tempfile::Builder::new()
        .prefix(".tutorial-upload-")
        .tempfile_in(root)
        .map_err(io_error)
}

pub(crate) async fn stage_bytes(root: &Path, bytes: &[u8]) -> AppResult<StagedAsset> {
    let temp = temporary(root)?;
    let mut file = fs::File::from_std(temp.reopen().map_err(io_error)?);
    file.write_all(bytes).await.map_err(io_error)?;
    file.sync_all().await.map_err(io_error)?;
    Ok(StagedAsset {
        temp,
        bytes: bytes.len() as u64,
        sha256: format!("{:x}", Sha256::digest(bytes)),
    })
}

pub(crate) fn path(root: &Path, id: Uuid, poster: bool) -> PathBuf {
    root.join(format!("{id}.{}", if poster { "png" } else { "mp4" }))
}

pub(crate) fn persist(asset: StagedAsset, root: &Path, id: Uuid, poster: bool) -> AppResult<()> {
    asset
        .temp
        .persist_noclobber(path(root, id, poster))
        .map_err(|_| AppError::Storage("无法发布教学媒体文件；已有文件不会被覆盖".into()))?;
    Ok(())
}

pub(crate) async fn open(root: &Path, id: Uuid, poster: bool, bytes: u64) -> AppResult<fs::File> {
    let path = path(root, id, poster);
    let before = fs::symlink_metadata(&path).await.map_err(io_error)?;
    ensure_regular(&before, bytes)?;
    let file = fs::File::open(&path).await.map_err(io_error)?;
    let opened = file.metadata().await.map_err(io_error)?;
    let after = fs::symlink_metadata(&path).await.map_err(io_error)?;
    ensure_regular(&opened, bytes)?;
    ensure_regular(&after, bytes)?;
    if !same_file(&before, &opened) || !same_file(&opened, &after) {
        return Err(AppError::Storage("教学媒体文件身份发生变化".into()));
    }
    Ok(file)
}

pub(crate) async fn verify(
    root: &Path,
    id: Uuid,
    poster: bool,
    bytes: u64,
    sha: &str,
) -> AppResult<fs::File> {
    let mut file = open(root, id, poster, bytes).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let size = file.read(&mut buffer).await.map_err(io_error)?;
        if size == 0 {
            break;
        }
        hasher.update(&buffer[..size]);
    }
    if format!("{:x}", hasher.finalize()) != sha {
        return Err(AppError::Storage("教学媒体完整性校验失败".into()));
    }
    Ok(file)
}

pub(crate) async fn remove(
    root: &Path,
    id: Uuid,
    poster: bool,
    bytes: u64,
    sha: &str,
) -> AppResult<()> {
    let target = path(root, id, poster);
    match fs::symlink_metadata(&target).await {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io_error(e)),
        Ok(_) => {}
    }
    let file = verify(root, id, poster, bytes, sha).await?;
    let opened = file.metadata().await.map_err(io_error)?;
    let current = fs::symlink_metadata(&target).await.map_err(io_error)?;
    if !same_file(&opened, &current) {
        return Err(AppError::Storage("清理前媒体身份已变化".into()));
    }
    fs::remove_file(target).await.map_err(io_error)
}

fn ensure_regular(meta: &std::fs::Metadata, bytes: u64) -> AppResult<()> {
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() != bytes {
        return Err(AppError::Storage("教学媒体文件身份无效".into()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() != 1 {
            return Err(AppError::Storage("教学媒体不能使用硬链接".into()));
        }
    }
    Ok(())
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        left.dev() == right.dev() && left.ino() == right.ino()
    }
    #[cfg(not(unix))]
    {
        left.len() == right.len() && left.modified().ok() == right.modified().ok()
    }
}

pub(crate) fn io_error(_: std::io::Error) -> AppError {
    AppError::Storage("教学媒体文件操作失败".into())
}

pub(crate) async fn sync_directory(root: &Path) -> AppResult<()> {
    // 先确保文件名变更落盘，再提交数据库公开指针或清理终态。
    fs::File::open(root)
        .await
        .map_err(io_error)?
        .sync_all()
        .await
        .map_err(io_error)
}
