//! 只读验证原子发布的备份清单及两个根目录直接子文件。
//! 检查器不创建目录、不写文件，也不把清单新鲜冒充可恢复证据。

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncReadExt};

use crate::ObservationCode;

const MANIFEST_NAME: &str = "latest-backup.json";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_FILE_NAME_BYTES: usize = 255;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupManifest {
    pub schema_version: u32,
    pub completed_at: DateTime<Utc>,
    pub database: BackupFileDeclaration,
    pub release_bundle: BackupFileDeclaration,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupFileDeclaration {
    pub file_name: String,
    pub byte_size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackupCheck {
    pub observation: ObservationCode,
    pub completed_at: Option<DateTime<Utc>>,
    pub checked_file_count: u64,
}

impl BackupCheck {
    fn observed(observation: ObservationCode) -> Self {
        Self {
            observation,
            completed_at: None,
            checked_file_count: 0,
        }
    }
}

pub async fn check_latest_backup(
    root: &Path,
    freshness: Duration,
    now: DateTime<Utc>,
) -> BackupCheck {
    let root = match validate_root(root).await {
        Ok(root) => root,
        Err(observation) => return BackupCheck::observed(observation),
    };
    let manifest = match read_manifest(&root).await {
        Ok(manifest) => manifest,
        Err(observation) => return BackupCheck::observed(observation),
    };
    if manifest.schema_version != 2 || manifest.completed_at > now {
        return BackupCheck::observed(ObservationCode::Invalid);
    }
    if manifest.database.file_name == manifest.release_bundle.file_name
        || manifest.database.file_name == MANIFEST_NAME
        || manifest.release_bundle.file_name == MANIFEST_NAME
    {
        return BackupCheck::observed(ObservationCode::Invalid);
    }
    for declaration in [&manifest.database, &manifest.release_bundle] {
        if let Err(observation) = validate_declared_file(&root, declaration).await {
            return BackupCheck {
                observation,
                completed_at: Some(manifest.completed_at),
                checked_file_count: 0,
            };
        }
    }
    let age = now.signed_duration_since(manifest.completed_at);
    let freshness = match chrono::Duration::from_std(freshness) {
        Ok(value) => value,
        Err(_) => return BackupCheck::observed(ObservationCode::Invalid),
    };
    let observation = if age > freshness {
        ObservationCode::Stale
    } else {
        ObservationCode::Healthy
    };
    BackupCheck {
        observation,
        completed_at: Some(manifest.completed_at),
        checked_file_count: 2,
    }
}

async fn validate_root(root: &Path) -> Result<PathBuf, ObservationCode> {
    let metadata = match fs::symlink_metadata(root).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(ObservationCode::Missing);
        }
        Err(_) => return Err(ObservationCode::Invalid),
    };
    if is_link_like(&metadata) || !metadata.is_dir() {
        return Err(ObservationCode::Invalid);
    }
    fs::canonicalize(root)
        .await
        .map_err(|_| ObservationCode::Invalid)
}

async fn read_manifest(root: &Path) -> Result<BackupManifest, ObservationCode> {
    let path = root.join(MANIFEST_NAME);
    let path_metadata = match fs::symlink_metadata(&path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(ObservationCode::Missing);
        }
        Err(_) => return Err(ObservationCode::Invalid),
    };
    if is_link_like(&path_metadata)
        || !path_metadata.is_file()
        || path_metadata.len() == 0
        || path_metadata.len() > MAX_MANIFEST_BYTES
    {
        return Err(ObservationCode::Invalid);
    }
    let file = fs::File::open(&path)
        .await
        .map_err(|error| classify_io(&error))?;
    let opened = file
        .metadata()
        .await
        .map_err(|_| ObservationCode::Invalid)?;
    if !same_file_identity(&path_metadata, &opened) {
        return Err(ObservationCode::Invalid);
    }
    let mut bytes = Vec::with_capacity(path_metadata.len() as usize);
    file.take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| ObservationCode::Invalid)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(ObservationCode::Invalid);
    }
    let current = fs::symlink_metadata(&path)
        .await
        .map_err(|error| classify_io(&error))?;
    if is_link_like(&current) || !same_file_identity(&opened, &current) {
        return Err(ObservationCode::Invalid);
    }
    serde_json::from_slice(&bytes).map_err(|_| ObservationCode::Invalid)
}

async fn validate_declared_file(
    root: &Path,
    declaration: &BackupFileDeclaration,
) -> Result<(), ObservationCode> {
    if !valid_direct_file_name(&declaration.file_name)
        || declaration.byte_size == 0
        || !valid_sha256(&declaration.sha256)
    {
        return Err(ObservationCode::Invalid);
    }
    let path = root.join(&declaration.file_name);
    if path.parent() != Some(root) {
        return Err(ObservationCode::Invalid);
    }
    let metadata = match fs::symlink_metadata(&path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(ObservationCode::Missing);
        }
        Err(_) => return Err(ObservationCode::Invalid),
    };
    if is_link_like(&metadata) || !metadata.is_file() || metadata.len() != declaration.byte_size {
        return Err(ObservationCode::Invalid);
    }
    let file = fs::File::open(&path)
        .await
        .map_err(|error| classify_io(&error))?;
    let opened = file
        .metadata()
        .await
        .map_err(|_| ObservationCode::Invalid)?;
    if opened.len() == 0 || !same_file_identity(&metadata, &opened) {
        return Err(ObservationCode::Invalid);
    }
    let current = fs::symlink_metadata(&path)
        .await
        .map_err(|error| classify_io(&error))?;
    if is_link_like(&current) || !same_file_identity(&opened, &current) {
        return Err(ObservationCode::Invalid);
    }
    Ok(())
}

fn valid_direct_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FILE_NAME_BYTES
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn classify_io(error: &std::io::Error) -> ObservationCode {
    if error.kind() == ErrorKind::NotFound {
        ObservationCode::Missing
    } else {
        ObservationCode::Invalid
    }
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
fn same_file_identity(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(windows)]
fn same_file_identity(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    left.file_size() == right.file_size()
        && left.last_write_time() == right.last_write_time()
        && left.creation_time() == right.creation_time()
        && left.file_attributes() == right.file_attributes()
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.len() == right.len()
        && left.modified().ok() == right.modified().ok()
        && left.created().ok() == right.created().ok()
}

#[cfg(test)]
mod backup_tests;
