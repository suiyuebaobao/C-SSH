//! 定义下载事件聚合的批次输入、UTC 日桶和公开报告。

use chrono::NaiveDate;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DownloadAggregationReport {
    pub processed_events: u64,
    pub updated_buckets: u64,
}

#[derive(Debug, FromRow)]
pub(crate) struct PendingDownloadEvent {
    pub id: Uuid,
    pub bucket_date: NaiveDate,
    pub asset_id: Uuid,
    pub source_id: Uuid,
    pub account_id: Option<Uuid>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DownloadAggregateBucket {
    pub bucket_date: NaiveDate,
    pub asset_id: Uuid,
    pub source_id: Uuid,
    pub audience: DownloadAudience,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DownloadAudience {
    Anonymous,
    Authenticated,
}

impl DownloadAudience {
    #[must_use]
    pub const fn from_account(account_id: Option<Uuid>) -> Self {
        if account_id.is_some() {
            Self::Authenticated
        } else {
            Self::Anonymous
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::Authenticated => "authenticated",
        }
    }
}
