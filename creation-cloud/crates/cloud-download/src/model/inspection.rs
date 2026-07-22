//! 定义已发布本站资产巡检状态、候选身份、观察值和公开报告。

use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetInspectionStatus {
    Healthy,
    Missing,
    SizeMismatch,
    HashMismatch,
    IoError,
}

impl AssetInspectionStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Missing => "missing",
            Self::SizeMismatch => "size_mismatch",
            Self::HashMismatch => "hash_mismatch",
            Self::IoError => "io_error",
        }
    }

    #[must_use]
    pub const fn is_finding(self) -> bool {
        !matches!(self, Self::Healthy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PublishedAssetInspectionReport {
    pub inspected_sources: u64,
    pub finding_count: u64,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct PublishedLocalAsset {
    pub source_id: Uuid,
    pub local_path: String,
    pub expected_byte_size: i64,
    pub expected_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssetInspectionObservation {
    pub source_id: Uuid,
    pub status: AssetInspectionStatus,
    pub observed_byte_size: Option<i64>,
    pub observed_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileInspection {
    pub status: AssetInspectionStatus,
    pub observed_byte_size: Option<i64>,
    pub observed_sha256: Option<String>,
}

impl AssetInspectionObservation {
    #[must_use]
    pub const fn finding(&self) -> bool {
        self.status.is_finding()
    }
}
