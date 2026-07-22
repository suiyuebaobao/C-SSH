//! 固定五类维护任务的公开名称、遍历顺序与数据库锁身份。

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

const LOCK_NAMESPACE: i32 = 1_129_528_148;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaintenanceTask {
    ExpiredSessions,
    SyncRetention,
    DownloadAggregation,
    PublishedAssetInspection,
    BackupFreshness,
}

impl MaintenanceTask {
    pub const ALL: [Self; 5] = [
        Self::ExpiredSessions,
        Self::SyncRetention,
        Self::DownloadAggregation,
        Self::PublishedAssetInspection,
        Self::BackupFreshness,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExpiredSessions => "expired-sessions",
            Self::SyncRetention => "sync-retention",
            Self::DownloadAggregation => "download-aggregation",
            Self::PublishedAssetInspection => "published-asset-inspection",
            Self::BackupFreshness => "backup-freshness",
        }
    }

    #[must_use]
    pub const fn advisory_lock_identity(self) -> (i32, i32) {
        let task_id = match self {
            Self::ExpiredSessions => 1,
            Self::SyncRetention => 2,
            Self::DownloadAggregation => 3,
            Self::PublishedAssetInspection => 4,
            Self::BackupFreshness => 5,
        };
        (LOCK_NAMESPACE, task_id)
    }
}

impl fmt::Display for MaintenanceTask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MaintenanceTask {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "expired-sessions" => Ok(Self::ExpiredSessions),
            "sync-retention" => Ok(Self::SyncRetention),
            "download-aggregation" => Ok(Self::DownloadAggregation),
            "published-asset-inspection" => Ok(Self::PublishedAssetInspection),
            "backup-freshness" => Ok(Self::BackupFreshness),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod task_tests;
