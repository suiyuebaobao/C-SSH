//! 在受控下载根内接收 multipart 文件并核对不可变资产身份。
//! 浏览器只能提供文件字节和来源名称，落盘路径全部由服务端生成。

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use axum::extract::{Multipart, multipart::Field};
use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

pub(crate) const MAX_ASSET_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_PROVIDER_FIELD_BYTES: usize = 512;

pub(crate) struct ReceivedUpload {
    pub provider_name: String,
}

pub(crate) struct UploadLayout {
    quarantine: PathBuf,
    objects: PathBuf,
}

impl UploadLayout {
    pub async fn prepare(download_root: &Path) -> AppResult<Self> {
        let root = ensure_root(download_root).await?;
        let quarantine = ensure_child_directory(&root, "quarantine").await?;
        let objects = ensure_child_directory(&root, "objects").await?;
        Ok(Self {
            quarantine,
            objects,
        })
    }

    #[must_use]
    pub fn temp_path(&self, upload_id: Uuid) -> PathBuf {
        self.quarantine.join(format!("{upload_id}.part"))
    }

    #[must_use]
    pub fn deletion_path(&self, deletion_id: Uuid) -> PathBuf {
        self.quarantine.join(format!("{deletion_id}.delete"))
    }

    #[must_use]
    pub fn object_path(&self, object_id: Uuid) -> PathBuf {
        self.objects.join(object_id.to_string())
    }

    pub async fn promote(
        &self,
        temp_path: &Path,
        object_id: Uuid,
        cleanup: &mut CleanupFile,
    ) -> AppResult<(PathBuf, String)> {
        let final_path = self.object_path(object_id);
        match fs::symlink_metadata(&final_path).await {
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Ok(_) => return Err(AppError::Conflict("上传文件目标已经存在".into())),
            Err(_) => return Err(AppError::Internal("无法检查上传文件目标".into())),
        }
        std::fs::rename(temp_path, &final_path)
            .map_err(|_| AppError::Internal("原子提交上传文件失败".into()))?;
        // 同步 rename 与清理权转移之间没有取消点，避免正式路径出现无主文件。
        cleanup.replace(final_path.clone());
        let metadata = fs::symlink_metadata(&final_path)
            .await
            .map_err(|_| AppError::Internal("无法复核已提交上传文件".into()))?;
        if is_link_like(&metadata) || !metadata.is_file() || !has_single_link(&metadata) {
            return Err(AppError::Internal("已提交上传文件类型无效".into()));
        }
        Ok((final_path, format!("objects/{object_id}")))
    }
}

pub(crate) struct CleanupFile {
    path: Option<PathBuf>,
}

impl CleanupFile {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    pub fn replace(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    pub fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for CleanupFile {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub(crate) async fn receive(
    multipart: &mut Multipart,
    temp_path: &Path,
    expected_size: u64,
    expected_sha256: &str,
) -> AppResult<ReceivedUpload> {
    let mut file_seen = false;
    let mut provider_name = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("上传表单格式无效".into()))?
    {
        let name = field
            .name()
            .map(str::to_owned)
            .ok_or_else(|| AppError::Validation("上传表单字段缺少名称".into()))?;
        match name.as_str() {
            "file" if !file_seen => {
                stream_file(&mut field, temp_path, expected_size, expected_sha256).await?;
                file_seen = true;
            }
            "file" => return Err(AppError::Validation("只能上传一个文件".into())),
            "provider_name" if provider_name.is_none() => {
                provider_name = Some(read_provider_name(&mut field).await?);
            }
            "provider_name" => {
                return Err(AppError::Validation("来源名称不能重复".into()));
            }
            _ => return Err(AppError::Validation("上传表单包含未知字段".into())),
        }
    }
    if !file_seen {
        return Err(AppError::Validation("上传表单缺少 file 字段".into()));
    }
    Ok(ReceivedUpload {
        provider_name: crate::validation::required_text(
            provider_name.as_deref().unwrap_or("本站"),
            "来源名称",
            100,
        )?,
    })
}

pub(crate) async fn readiness_probe(download_root: &Path) -> AppResult<()> {
    let layout = UploadLayout::prepare(download_root).await?;
    let object_id = Uuid::now_v7();
    let temp_path = layout.temp_path(object_id);
    let mut cleanup = CleanupFile::new(temp_path.clone());
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .await
        .map_err(|_| AppError::Storage("无法创建下载就绪探针".into()))?;
    file.write_all(b"ready")
        .await
        .map_err(|_| AppError::Storage("无法写入下载就绪探针".into()))?;
    file.sync_all()
        .await
        .map_err(|_| AppError::Storage("无法同步下载就绪探针".into()))?;
    drop(file);
    let (final_path, _) = layout.promote(&temp_path, object_id, &mut cleanup).await?;
    fs::remove_file(&final_path)
        .await
        .map_err(|_| AppError::Storage("无法清理下载就绪探针".into()))?;
    cleanup.disarm();
    Ok(())
}

async fn stream_file(
    field: &mut Field<'_>,
    temp_path: &Path,
    expected_size: u64,
    expected_sha256: &str,
) -> AppResult<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
        .await
        .map_err(|_| AppError::Conflict("上传临时文件已经存在".into()))?;
    let mut identity = UploadIdentity::new(expected_size, expected_sha256)?;
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| AppError::Validation("上传文件流提前中断".into()))?
    {
        identity.observe(&chunk)?;
        file.write_all(&chunk)
            .await
            .map_err(|_| AppError::Internal("写入上传临时文件失败".into()))?;
    }
    identity.finish()?;
    file.flush()
        .await
        .map_err(|_| AppError::Internal("刷新上传临时文件失败".into()))?;
    file.sync_all()
        .await
        .map_err(|_| AppError::Internal("同步上传临时文件失败".into()))
}

async fn read_provider_name(field: &mut Field<'_>) -> AppResult<String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| AppError::Validation("读取来源名称失败".into()))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_PROVIDER_FIELD_BYTES {
            return Err(AppError::Validation("来源名称过长".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| AppError::Validation("来源名称必须是 UTF-8".into()))
}

pub(crate) struct UploadIdentity {
    expected_size: u64,
    expected_sha256: String,
    size: u64,
    hasher: Sha256,
}

impl UploadIdentity {
    pub fn new(expected_size: u64, expected_sha256: &str) -> AppResult<Self> {
        let expected_sha256 = expected_sha256.trim().to_ascii_lowercase();
        validate_asset_identity(expected_size, &expected_sha256)?;
        Ok(Self {
            expected_size,
            expected_sha256,
            size: 0,
            hasher: Sha256::new(),
        })
    }

    pub fn observe(&mut self, chunk: &[u8]) -> AppResult<()> {
        let chunk_size = u64::try_from(chunk.len())
            .map_err(|_| AppError::Validation("上传分块大小无效".into()))?;
        let next_size = self
            .size
            .checked_add(chunk_size)
            .ok_or_else(|| AppError::Validation("上传文件大小溢出".into()))?;
        if next_size > self.expected_size || next_size > MAX_ASSET_BYTES {
            return Err(AppError::Validation("上传文件超过资产声明大小".into()));
        }
        self.hasher.update(chunk);
        self.size = next_size;
        Ok(())
    }

    pub fn finish(self) -> AppResult<()> {
        if self.size != self.expected_size {
            return Err(AppError::Conflict("上传文件大小与资产身份不一致".into()));
        }
        let actual = format!("{:x}", self.hasher.finalize());
        if actual != self.expected_sha256 {
            return Err(AppError::Conflict(
                "上传文件 SHA256 与资产身份不一致".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn validate_asset_identity(expected_size: u64, expected_sha256: &str) -> AppResult<()> {
    if expected_size == 0
        || expected_size > MAX_ASSET_BYTES
        || expected_sha256.len() != 64
        || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AppError::Conflict("资产身份不允许上传".into()));
    }
    Ok(())
}

async fn ensure_root(path: &Path) -> AppResult<PathBuf> {
    if let Err(error) = fs::create_dir_all(path).await
        && error.kind() != ErrorKind::AlreadyExists
    {
        return Err(AppError::Internal("无法创建下载根目录".into()));
    }
    let metadata = fs::symlink_metadata(path)
        .await
        .map_err(|_| AppError::Internal("下载根目录不可用".into()))?;
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(AppError::Forbidden("下载根目录不能是符号链接".into()));
    }
    fs::canonicalize(path)
        .await
        .map_err(|_| AppError::Internal("下载根目录不可用".into()))
}

async fn ensure_child_directory(root: &Path, name: &str) -> AppResult<PathBuf> {
    let path = root.join(name);
    match fs::create_dir(&path).await {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(_) => return Err(AppError::Internal("无法创建上传存储目录".into())),
    }
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|_| AppError::Internal("上传存储目录不可用".into()))?;
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(AppError::Forbidden("上传存储目录不能是符号链接".into()));
    }
    restrict_directory(&path).await?;
    let canonical = fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::Internal("上传存储目录不可用".into()))?;
    if canonical.parent() != Some(root) {
        return Err(AppError::Forbidden("上传存储目录越出下载根目录".into()));
    }
    Ok(canonical)
}

#[cfg(unix)]
async fn restrict_directory(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .await
        .map_err(|_| AppError::Internal("无法收紧上传存储目录权限".into()))
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
    {
        false
    }
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
