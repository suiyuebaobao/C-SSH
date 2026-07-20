//! 安全解析本站相对路径供公开下载读取。

use std::path::{Component, Path, PathBuf};

use cloud_domain::{AppError, AppResult};
use tokio::fs;

pub(crate) async fn resolve(download_root: &Path, relative: &str) -> AppResult<PathBuf> {
    let root_metadata = fs::symlink_metadata(download_root)
        .await
        .map_err(|_| AppError::Internal("下载根目录不可用".into()))?;
    if is_link_like(&root_metadata) || !root_metadata.is_dir() {
        return Err(AppError::Forbidden("下载根目录不能是符号链接".into()));
    }
    let root = fs::canonicalize(download_root)
        .await
        .map_err(|_| AppError::Internal("下载根目录不可用".into()))?;
    let relative = crate::validation::local_path(relative)?;
    let mut unresolved = root.clone();
    for component in Path::new(&relative).components() {
        let Component::Normal(component) = component else {
            return Err(AppError::Validation("本站下载路径格式无效".into()));
        };
        unresolved.push(component);
        let metadata = fs::symlink_metadata(&unresolved)
            .await
            .map_err(|_| AppError::NotFound("本站下载文件不存在".into()))?;
        if is_link_like(&metadata) {
            return Err(AppError::Forbidden("本站下载路径不能包含符号链接".into()));
        }
    }
    let target = fs::canonicalize(&unresolved)
        .await
        .map_err(|_| AppError::NotFound("本站下载文件不存在".into()))?;
    if !target.starts_with(&root) {
        return Err(AppError::Forbidden("本站下载路径越出发布目录".into()));
    }
    let metadata = fs::symlink_metadata(&target)
        .await
        .map_err(|_| AppError::NotFound("本站下载文件不存在".into()))?;
    if is_link_like(&metadata) || !metadata.is_file() || !has_single_link(&metadata) {
        return Err(AppError::Validation("本站来源必须指向普通文件".into()));
    }
    Ok(target)
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
