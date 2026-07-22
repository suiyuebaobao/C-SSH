//! 定义用户中心可展示且不包含地址、Cookie 或请求头的下载历史投影。

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct DownloadHistoryItem {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub source_id: Uuid,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub provider_name: String,
    pub source_kind: String,
    pub occurred_at: DateTime<Utc>,
}
