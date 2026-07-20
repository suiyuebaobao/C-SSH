//! 定义用户、设备、版本和审计四类只读管理概览。

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct UserOverview {
    pub total_users: i64,
    pub active_users: i64,
    pub disabled_users: i64,
    pub admin_users: i64,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct DeviceOverview {
    pub total_devices: i64,
    pub active_devices: i64,
    pub revoked_devices: i64,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct ReleaseOverview {
    pub total_releases: i64,
    pub draft_releases: i64,
    pub validating_releases: i64,
    pub published_releases: i64,
    pub revoked_releases: i64,
    pub hidden_releases: i64,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct SecurityAuditOverview {
    pub total_events: i64,
    pub successful_events: i64,
    pub failed_events: i64,
    pub latest_event_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminOverview {
    pub users: UserOverview,
    pub devices: DeviceOverview,
    pub releases: ReleaseOverview,
    pub audit: SecurityAuditOverview,
}
