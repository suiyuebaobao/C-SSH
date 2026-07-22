//! 定义已发布站点媒体巡检的固定状态、候选身份、观察和公开报告。

use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMediaInspectionStatus {
    Healthy,
    Missing,
    SizeMismatch,
    HashMismatch,
    IoError,
}

impl SiteMediaInspectionStatus {
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
pub struct PublishedMediaInspectionReport {
    pub inspected_media: u64,
    pub finding_count: u64,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct PublishedMediaCandidate {
    pub media_id: Uuid,
    pub storage_key: String,
    pub expected_byte_size: i64,
    pub expected_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SiteMediaInspectionObservation {
    pub media_id: Uuid,
    pub status: SiteMediaInspectionStatus,
    pub observed_byte_size: Option<i64>,
    pub observed_sha256: Option<String>,
}

impl SiteMediaInspectionObservation {
    #[must_use]
    pub const fn finding(&self) -> bool {
        self.status.is_finding()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectedFile {
    pub status: SiteMediaInspectionStatus,
    pub observed_byte_size: Option<i64>,
    pub observed_sha256: Option<String>,
}
