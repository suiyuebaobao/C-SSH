//! 在站点媒体根目录内执行隔离写入、原子落位与受控路径读取。
//! 所有磁盘操作都拒绝符号链接和 Windows reparse point，浏览器不能提供路径。

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{AsyncWriteExt, BufWriter},
};
use uuid::Uuid;

pub(crate) struct StagedFile {
    pub storage_key: String,
    quarantine_path: PathBuf,
    final_path: PathBuf,
    committed: bool,
    armed: bool,
}

pub(crate) struct DeletedFile {
    original_path: PathBuf,
    quarantine_path: PathBuf,
    armed: bool,
    restore_on_drop: bool,
}

struct StorageLayout {
    quarantine: PathBuf,
    objects: PathBuf,
}

pub(crate) async fn stage(root: &Path, png: &[u8]) -> AppResult<StagedFile> {
    let layout = prepare_layout(root).await?;
    let opaque = Uuid::new_v4().to_string();
    let prefix = &opaque[..2];
    let storage_key = format!("objects/{prefix}/{opaque}.png");
    let object_dir = ensure_child_directory(&layout.objects, prefix).await?;
    let final_path = object_dir.join(format!("{opaque}.png"));
    let quarantine_path = layout.quarantine.join(format!("{}.upload", Uuid::new_v4()));
    let file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&quarantine_path)
        .await
        .map_err(|_| AppError::Storage("无法创建站点媒体隔离文件".into()))?;
    let staged = StagedFile {
        storage_key,
        quarantine_path,
        final_path,
        committed: false,
        armed: true,
    };
    let mut writer = BufWriter::new(file);
    if writer.write_all(png).await.is_err() || writer.flush().await.is_err() {
        drop(writer);
        return Err(AppError::Storage("写入站点媒体隔离文件失败".into()));
    }
    let file = writer.into_inner();
    if file.sync_all().await.is_err() {
        drop(file);
        return Err(AppError::Storage("同步站点媒体隔离文件失败".into()));
    }
    Ok(staged)
}

impl StagedFile {
    pub async fn commit(&mut self) -> AppResult<()> {
        ensure_target_absent(&self.final_path).await?;
        std::fs::rename(&self.quarantine_path, &self.final_path)
            .map_err(|_| AppError::Storage("站点媒体原子落位失败".into()))?;
        // 同步 rename 与清理权转移之间没有取消点，后续失败也只删除本轮文件。
        self.committed = true;
        ensure_regular_file(&self.final_path, "已落位站点媒体文件无效").await
    }

    pub async fn cleanup(&mut self) {
        let _ = fs::remove_file(&self.quarantine_path).await;
        if self.committed {
            let _ = fs::remove_file(&self.final_path).await;
        }
        self.armed = false;
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }

    pub fn preserve_on_drop(&mut self) {
        self.armed = false;
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // future 被取消时仍同步清理由本轮生成的文件，绝不触碰冲突目标。
        let _ = std::fs::remove_file(&self.quarantine_path);
        if self.committed {
            let _ = std::fs::remove_file(&self.final_path);
        }
    }
}

pub(crate) async fn quarantine_delete(root: &Path, storage_key: &str) -> AppResult<DeletedFile> {
    let layout = prepare_layout(root).await?;
    let original_path = resolve_existing_object(&layout.objects, storage_key).await?;
    let quarantine_path = layout.quarantine.join(format!("{}.delete", Uuid::new_v4()));
    ensure_target_absent(&quarantine_path).await?;
    std::fs::rename(&original_path, &quarantine_path)
        .map_err(|_| AppError::Storage("站点媒体文件无法进入删除隔离区".into()))?;
    let mut deleted = DeletedFile {
        original_path,
        quarantine_path,
        armed: true,
        restore_on_drop: true,
    };
    if let Err(error) = ensure_regular_file(&deleted.quarantine_path, "删除隔离文件无效").await
    {
        deleted.restore().await?;
        return Err(error);
    }
    Ok(deleted)
}

impl DeletedFile {
    pub fn preserve_on_drop(&mut self) {
        self.armed = false;
    }

    pub async fn restore(&mut self) -> AppResult<()> {
        ensure_target_absent(&self.original_path).await?;
        std::fs::rename(&self.quarantine_path, &self.original_path)
            .map_err(|_| AppError::Storage("站点媒体删除回滚失败".into()))?;
        ensure_regular_file(&self.original_path, "回滚后的站点媒体文件无效").await?;
        self.armed = false;
        Ok(())
    }

    pub async fn finish(mut self) -> AppResult<()> {
        // 数据库提交后不再允许恢复原路径；取消时 Drop 只清除本轮隔离文件。
        self.restore_on_drop = false;
        fs::remove_file(&self.quarantine_path)
            .await
            .map_err(|_| AppError::Storage("站点媒体删除隔离文件清理失败".into()))?;
        self.armed = false;
        Ok(())
    }
}

impl Drop for DeletedFile {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.restore_on_drop
            && std::fs::symlink_metadata(&self.original_path)
                .is_err_and(|error| error.kind() == ErrorKind::NotFound)
        {
            let _ = std::fs::rename(&self.quarantine_path, &self.original_path);
        } else if !self.restore_on_drop {
            let _ = std::fs::remove_file(&self.quarantine_path);
        }
    }
}

pub(crate) async fn read_verified(
    root: &Path,
    storage_key: &str,
    expected_size: i64,
    expected_sha256: &str,
) -> AppResult<Vec<u8>> {
    let layout = prepare_layout(root).await?;
    let path = resolve_existing_object(&layout.objects, storage_key).await?;
    let bytes = fs::read(path)
        .await
        .map_err(|_| AppError::NotFound("公开站点媒体文件不存在".into()))?;
    let size_matches = i64::try_from(bytes.len()).ok() == Some(expected_size);
    let hash_matches = hex::encode(Sha256::digest(&bytes)) == expected_sha256;
    if !size_matches || !hash_matches {
        return Err(AppError::Storage("公开站点媒体文件身份校验失败".into()));
    }
    Ok(bytes)
}

pub(crate) async fn readiness_probe(root: &Path) -> AppResult<()> {
    let mut staged = stage(root, b"ready").await?;
    staged.commit().await?;
    fs::remove_file(&staged.final_path)
        .await
        .map_err(|_| AppError::Storage("无法清理站点媒体就绪探针".into()))?;
    staged.disarm();
    Ok(())
}

#[cfg(test)]
pub(crate) fn controlled_path(root: &Path, storage_key: &str) -> AppResult<PathBuf> {
    validate_storage_key(storage_key)?;
    Ok(root.join(storage_key))
}

fn validate_storage_key(storage_key: &str) -> AppResult<(&str, &str)> {
    if storage_key.contains('\\') || Path::new(storage_key).is_absolute() {
        return Err(AppError::Internal("站点媒体存储键不受控".into()));
    }
    let mut parts = storage_key.split('/');
    let (Some(objects), Some(prefix), Some(file), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(AppError::Internal("站点媒体存储键不受控".into()));
    };
    let uuid_text = file
        .strip_suffix(".png")
        .ok_or_else(|| AppError::Internal("站点媒体存储键扩展名无效".into()))?;
    let valid_prefix = prefix.len() == 2
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    let valid_uuid = Uuid::parse_str(uuid_text).is_ok() && uuid_text.starts_with(prefix);
    if objects != "objects" || !valid_prefix || !valid_uuid {
        return Err(AppError::Internal("站点媒体存储键不受控".into()));
    }
    Ok((prefix, file))
}

async fn prepare_layout(root: &Path) -> AppResult<StorageLayout> {
    let root = ensure_root(root).await?;
    Ok(StorageLayout {
        quarantine: ensure_child_directory(&root, "quarantine").await?,
        objects: ensure_child_directory(&root, "objects").await?,
    })
}

async fn ensure_root(path: &Path) -> AppResult<PathBuf> {
    if let Err(error) = fs::create_dir_all(path).await
        && error.kind() != ErrorKind::AlreadyExists
    {
        return Err(AppError::Storage("无法创建站点媒体根目录".into()));
    }
    let metadata = fs::symlink_metadata(path)
        .await
        .map_err(|_| AppError::Storage("站点媒体根目录不可用".into()))?;
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(AppError::Forbidden("站点媒体根目录不能是符号链接".into()));
    }
    fs::canonicalize(path)
        .await
        .map_err(|_| AppError::Storage("站点媒体根目录不可用".into()))
}

async fn ensure_child_directory(root: &Path, name: &str) -> AppResult<PathBuf> {
    let path = root.join(name);
    match fs::create_dir(&path).await {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(_) => return Err(AppError::Storage("无法创建站点媒体存储目录".into())),
    }
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|_| AppError::Storage("站点媒体存储目录不可用".into()))?;
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(AppError::Forbidden("站点媒体存储目录不能是符号链接".into()));
    }
    restrict_directory(&path).await?;
    let canonical = fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::Storage("站点媒体存储目录不可用".into()))?;
    if canonical.parent() != Some(root) {
        return Err(AppError::Forbidden("站点媒体存储目录越出受控根目录".into()));
    }
    Ok(canonical)
}

async fn resolve_existing_object(objects: &Path, storage_key: &str) -> AppResult<PathBuf> {
    let (prefix, file) = validate_storage_key(storage_key)?;
    let object_dir = existing_child_directory(objects, prefix).await?;
    let path = object_dir.join(file);
    ensure_regular_file(&path, "公开站点媒体文件不存在").await?;
    let canonical = fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::NotFound("公开站点媒体文件不存在".into()))?;
    if canonical.parent() != Some(object_dir.as_path()) {
        return Err(AppError::Forbidden("站点媒体文件越出受控对象目录".into()));
    }
    Ok(canonical)
}

async fn existing_child_directory(root: &Path, name: &str) -> AppResult<PathBuf> {
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|_| AppError::NotFound("站点媒体对象目录不存在".into()))?;
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(AppError::Forbidden("站点媒体对象目录不能是符号链接".into()));
    }
    let canonical = fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::NotFound("站点媒体对象目录不存在".into()))?;
    if canonical.parent() != Some(root) {
        return Err(AppError::Forbidden("站点媒体对象目录越出受控根目录".into()));
    }
    Ok(canonical)
}

async fn ensure_target_absent(path: &Path) -> AppResult<()> {
    match fs::symlink_metadata(path).await {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(AppError::Conflict("站点媒体目标已存在".into())),
        Err(_) => Err(AppError::Storage("无法确认站点媒体目标是否存在".into())),
    }
}

async fn ensure_regular_file(path: &Path, message: &str) -> AppResult<()> {
    let metadata = fs::symlink_metadata(path)
        .await
        .map_err(|_| AppError::NotFound(message.into()))?;
    if is_link_like(&metadata) || !metadata.is_file() || !has_single_link(&metadata) {
        return Err(AppError::Forbidden(message.into()));
    }
    Ok(())
}

#[cfg(unix)]
async fn restrict_directory(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .await
        .map_err(|_| AppError::Storage("无法收紧站点媒体目录权限".into()))
}

#[cfg(not(unix))]
async fn restrict_directory(_path: &Path) -> AppResult<()> {
    Ok(())
}

fn is_link_like(metadata: &std::fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    false
}

#[cfg(unix)]
fn has_single_link(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() == 1
}

#[cfg(not(unix))]
fn has_single_link(_metadata: &std::fs::Metadata) -> bool {
    true
}
