//! 以只读流式哈希和前后文件身份复核分类站点媒体观察结果。

use std::{io::ErrorKind, path::Path};

use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncReadExt};

use crate::inspection_model::{InspectedFile, SiteMediaInspectionStatus};

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStamp {
    values: [u64; 7],
}

pub(crate) async fn inspect(
    path: &Path,
    expected_size: i64,
    expected_sha256: &str,
) -> InspectedFile {
    let mut file = match fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return result(SiteMediaInspectionStatus::Missing, None, None);
        }
        Err(_) => return result(SiteMediaInspectionStatus::IoError, None, None),
    };
    let before_metadata = match file.metadata().await {
        Ok(metadata) if is_regular_single_link(&metadata) => metadata,
        _ => return result(SiteMediaInspectionStatus::IoError, None, None),
    };
    let before = file_stamp(&before_metadata);
    let observed_size = i64::try_from(before_metadata.len()).ok();
    if observed_size != Some(expected_size) {
        return result(SiteMediaInspectionStatus::SizeMismatch, observed_size, None);
    }

    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = match file.read(&mut buffer).await {
            Ok(read) => read,
            Err(_) => return result(SiteMediaInspectionStatus::IoError, observed_size, None),
        };
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let after = match file.metadata().await {
        Ok(metadata) if is_regular_single_link(&metadata) => file_stamp(&metadata),
        _ => return result(SiteMediaInspectionStatus::IoError, observed_size, None),
    };
    if after != before {
        return result(SiteMediaInspectionStatus::IoError, observed_size, None);
    }
    let observed_sha256 = format!("{:x}", hasher.finalize());
    let status = if observed_sha256.eq_ignore_ascii_case(expected_sha256) {
        SiteMediaInspectionStatus::Healthy
    } else {
        SiteMediaInspectionStatus::HashMismatch
    };
    result(status, observed_size, Some(observed_sha256))
}

fn result(
    status: SiteMediaInspectionStatus,
    observed_byte_size: Option<i64>,
    observed_sha256: Option<String>,
) -> InspectedFile {
    InspectedFile {
        status,
        observed_byte_size,
        observed_sha256,
    }
}

#[cfg(unix)]
fn file_stamp(metadata: &std::fs::Metadata) -> FileStamp {
    use std::os::unix::fs::MetadataExt;

    FileStamp {
        values: [
            metadata.len(),
            metadata.dev(),
            metadata.ino(),
            metadata.mtime() as u64,
            metadata.mtime_nsec() as u64,
            metadata.ctime() as u64,
            metadata.ctime_nsec() as u64,
        ],
    }
}

#[cfg(windows)]
fn file_stamp(metadata: &std::fs::Metadata) -> FileStamp {
    use std::os::windows::fs::MetadataExt;

    FileStamp {
        values: [
            metadata.file_size(),
            metadata.last_write_time(),
            metadata.creation_time(),
            u64::from(metadata.file_attributes()),
            0,
            0,
            0,
        ],
    }
}

#[cfg(not(any(unix, windows)))]
fn file_stamp(metadata: &std::fs::Metadata) -> FileStamp {
    FileStamp {
        values: [metadata.len(), 0, 0, 0, 0, 0, 0],
    }
}

fn is_regular_single_link(metadata: &std::fs::Metadata) -> bool {
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        metadata.nlink() == 1
    }
    #[cfg(not(unix))]
    {
        true
    }
}
