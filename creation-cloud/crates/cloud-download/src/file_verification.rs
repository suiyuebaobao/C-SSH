//! 对公开本站文件执行按身份单飞的 SHA256 复核。
//! 有效 Range 才触发校验，缓存受文件身份、期望摘要、时限和容量共同约束。

use std::{
    collections::HashMap,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt},
    sync::{Notify, Semaphore},
};

use crate::model::{AssetInspectionStatus, FileInspection};

const CACHE_TTL: Duration = Duration::from_secs(600);
const MAX_CACHE_ENTRIES: usize = 128;
pub(crate) const MAX_PARALLEL_HASHES: usize = 2;

#[derive(Clone)]
pub(crate) struct FileVerifier {
    entries: Arc<Mutex<HashMap<VerificationKey, Arc<VerificationEntry>>>>,
    permits: Arc<Semaphore>,
}

#[derive(Clone)]
pub(crate) struct InspectionVerifier {
    permits: Arc<Semaphore>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct VerificationKey {
    path: PathBuf,
    expected_size: u64,
    expected_sha256: String,
    stamp: FileStamp,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct FileStamp {
    values: [u64; 7],
}

struct VerificationEntry {
    outcome: Mutex<Option<Result<Instant, VerificationFailure>>>,
    notify: Notify,
}

#[derive(Clone)]
enum VerificationFailure {
    NotFound,
    Storage,
    Conflict,
}

impl Default for FileVerifier {
    fn default() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            permits: Arc::new(Semaphore::new(MAX_PARALLEL_HASHES)),
        }
    }
}

impl Default for InspectionVerifier {
    fn default() -> Self {
        Self {
            // 巡检必须与在线下载的缓存和等待队列完全隔离，避免后台批次阻塞用户请求。
            permits: Arc::new(Semaphore::new(MAX_PARALLEL_HASHES)),
        }
    }
}

impl FileVerifier {
    pub(crate) async fn verify(
        &self,
        path: &Path,
        file: &mut fs::File,
        expected_size: u64,
        expected_sha256: &str,
    ) -> AppResult<()> {
        let stamp = file_stamp(
            &file
                .metadata()
                .await
                .map_err(|_| AppError::Internal("无法读取本站文件元数据".into()))?,
        );
        if stamp.values[0] != expected_size {
            return Err(AppError::Conflict("本站文件大小与发布资产不一致".into()));
        }
        let key = VerificationKey {
            path: path.to_path_buf(),
            expected_size,
            expected_sha256: expected_sha256.to_ascii_lowercase(),
            stamp,
        };
        let entry = self.entry(key.clone())?;
        entry.wait().await.map_err(VerificationFailure::error)?;
        let current = file_stamp(
            &file
                .metadata()
                .await
                .map_err(|_| AppError::Internal("无法复核本站文件元数据".into()))?,
        );
        if current != key.stamp {
            return Err(AppError::Conflict("本站文件在身份校验期间发生变化".into()));
        }
        file.seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(|_| AppError::Internal("重置本站文件读取位置失败".into()))?;
        Ok(())
    }

    fn entry(&self, key: VerificationKey) -> AppResult<Arc<VerificationEntry>> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = entries.get(&key)
            && entry.is_fresh()
        {
            return Ok(entry.clone());
        }
        entries.retain(|_, entry| entry.is_fresh());
        while entries.len() >= MAX_CACHE_ENTRIES {
            let Some(completed) = entries
                .iter()
                .find_map(|(key, entry)| entry.is_completed().then(|| key.clone()))
            else {
                break;
            };
            entries.remove(&completed);
        }
        if entries.len() >= MAX_CACHE_ENTRIES {
            return Err(AppError::Unavailable(
                "本站文件身份校验队列繁忙，请稍后重试".into(),
            ));
        }
        let entry = Arc::new(VerificationEntry::new());
        entries.insert(key.clone(), entry.clone());
        drop(entries);
        self.spawn_verification(key, entry.clone());
        Ok(entry)
    }

    fn spawn_verification(&self, key: VerificationKey, entry: Arc<VerificationEntry>) {
        let permits = self.permits.clone();
        let entries = self.entries.clone();
        tokio::spawn(async move {
            let result = verify_path(&key, permits).await;
            entry.finish(result.clone());
            if result.is_err() {
                let mut active = entries
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if active
                    .get(&key)
                    .is_some_and(|current| Arc::ptr_eq(current, &entry))
                {
                    active.remove(&key);
                }
            }
        });
    }
}

impl InspectionVerifier {
    pub(crate) async fn inspect(
        &self,
        path: &Path,
        expected_size: i64,
        expected_sha256: &str,
    ) -> FileInspection {
        let Ok(_permit) = self.permits.clone().acquire_owned().await else {
            return inspection(AssetInspectionStatus::IoError, None, None);
        };
        let mut file = match fs::File::open(path).await {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return inspection(AssetInspectionStatus::Missing, None, None);
            }
            Err(_) => return inspection(AssetInspectionStatus::IoError, None, None),
        };
        let before_metadata = match file.metadata().await {
            Ok(metadata) if is_regular_single_link(&metadata) => metadata,
            _ => return inspection(AssetInspectionStatus::IoError, None, None),
        };
        let before = file_stamp(&before_metadata);
        let observed_size = i64::try_from(before_metadata.len()).ok();
        if observed_size != Some(expected_size) {
            return inspection(AssetInspectionStatus::SizeMismatch, observed_size, None);
        }

        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = match file.read(&mut buffer).await {
                Ok(read) => read,
                Err(_) => {
                    return inspection(AssetInspectionStatus::IoError, observed_size, None);
                }
            };
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let after = match file.metadata().await {
            Ok(metadata) if is_regular_single_link(&metadata) => file_stamp(&metadata),
            _ => return inspection(AssetInspectionStatus::IoError, observed_size, None),
        };
        if after != before {
            return inspection(AssetInspectionStatus::IoError, observed_size, None);
        }
        let observed_sha256 = format!("{:x}", hasher.finalize());
        let status = if observed_sha256.eq_ignore_ascii_case(expected_sha256) {
            AssetInspectionStatus::Healthy
        } else {
            AssetInspectionStatus::HashMismatch
        };
        inspection(status, observed_size, Some(observed_sha256))
    }
}

fn inspection(
    status: AssetInspectionStatus,
    observed_byte_size: Option<i64>,
    observed_sha256: Option<String>,
) -> FileInspection {
    FileInspection {
        status,
        observed_byte_size,
        observed_sha256,
    }
}

impl VerificationEntry {
    fn new() -> Self {
        Self {
            outcome: Mutex::new(None),
            notify: Notify::new(),
        }
    }

    fn is_fresh(&self) -> bool {
        self.outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_none_or(|outcome| match outcome {
                Ok(at) => at.elapsed() <= CACHE_TTL,
                Err(_) => false,
            })
    }

    fn is_completed(&self) -> bool {
        self.outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    async fn wait(&self) -> Result<(), VerificationFailure> {
        loop {
            let notified = self.notify.notified();
            if let Some(outcome) = self
                .outcome
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
            {
                return outcome.map(|_| ());
            }
            notified.await;
        }
    }

    fn finish(&self, result: Result<(), VerificationFailure>) {
        *self
            .outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(result.map(|()| Instant::now()));
        self.notify.notify_waiters();
    }
}

impl VerificationFailure {
    fn error(self) -> AppError {
        match self {
            Self::NotFound => AppError::NotFound("本站下载文件不可读".into()),
            Self::Storage => AppError::Internal("读取本站文件校验身份失败".into()),
            Self::Conflict => AppError::Conflict("本站文件 SHA256 与发布资产不一致".into()),
        }
    }
}

async fn verify_path(
    key: &VerificationKey,
    permits: Arc<Semaphore>,
) -> Result<(), VerificationFailure> {
    let _permit = permits
        .acquire_owned()
        .await
        .map_err(|_| VerificationFailure::Storage)?;
    let mut file = fs::File::open(&key.path)
        .await
        .map_err(|_| VerificationFailure::NotFound)?;
    let before = file
        .metadata()
        .await
        .map(|metadata| file_stamp(&metadata))
        .map_err(|_| VerificationFailure::Storage)?;
    if before != key.stamp || before.values[0] != key.expected_size {
        return Err(VerificationFailure::Conflict);
    }
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|_| VerificationFailure::Storage)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let after = file
        .metadata()
        .await
        .map(|metadata| file_stamp(&metadata))
        .map_err(|_| VerificationFailure::Storage)?;
    let actual = format!("{:x}", hasher.finalize());
    if after != before || !actual.eq_ignore_ascii_case(&key.expected_sha256) {
        return Err(VerificationFailure::Conflict);
    }
    Ok(())
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
    fn encoded(value: std::io::Result<std::time::SystemTime>) -> (u64, u64) {
        value
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or((0, 0), |duration| {
                (duration.as_secs(), u64::from(duration.subsec_nanos()))
            })
    }
    let modified = encoded(metadata.modified());
    let created = encoded(metadata.created());
    FileStamp {
        values: [
            metadata.len(),
            modified.0,
            modified.1,
            created.0,
            created.1,
            0,
            0,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_size_change_invalidates_a_positive_verification() {
        let root = tempfile::tempdir().expect("临时下载目录应可创建");
        let path = root.path().join("asset");
        fs::write(&path, b"verified")
            .await
            .expect("测试文件应可写入");
        let expected = format!("{:x}", Sha256::digest(b"verified"));
        let verifier = FileVerifier::default();
        let mut original = fs::File::open(&path).await.expect("测试文件应可打开");
        verifier
            .verify(&path, &mut original, 8, &expected)
            .await
            .expect("真实摘要应通过校验");

        std::thread::sleep(Duration::from_millis(2));
        fs::write(&path, b"tampered")
            .await
            .expect("同长度篡改应可写入");
        let mut changed = fs::File::open(&path).await.expect("篡改文件应可打开");
        assert!(
            verifier
                .verify(&path, &mut changed, 8, &expected)
                .await
                .is_err()
        );
    }

    #[test]
    fn pending_entries_are_reused_and_never_evicted_by_admission() {
        let verifier = FileVerifier::default();
        let mut first = None;
        {
            let mut entries = verifier
                .entries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for index in 0..MAX_CACHE_ENTRIES {
                let key = VerificationKey {
                    path: PathBuf::from(format!("objects/{index}")),
                    expected_size: 8,
                    expected_sha256: format!("{index:064x}"),
                    stamp: FileStamp {
                        values: [8, index as u64, 0, 0, 0, 0, 0],
                    },
                };
                let entry = Arc::new(VerificationEntry::new());
                if index == 0 {
                    first = Some((key.clone(), entry.clone()));
                }
                entries.insert(key, entry);
            }
        }
        let (first_key, first_entry) = first.expect("首个等待项应存在");
        let reused = verifier.entry(first_key).expect("同一等待项应复用");
        assert!(Arc::ptr_eq(&reused, &first_entry));

        let overflow = VerificationKey {
            path: PathBuf::from("objects/overflow"),
            expected_size: 8,
            expected_sha256: "f".repeat(64),
            stamp: FileStamp {
                values: [8, u64::MAX, 0, 0, 0, 0, 0],
            },
        };
        assert!(matches!(
            verifier.entry(overflow),
            Err(AppError::Unavailable(_))
        ));
    }
}
