//! 集中维护默认生产参数和环境覆盖的保护上下界。

use std::ops::RangeInclusive;

pub(super) const DEFAULT_EXPIRED_INTERVAL_SECONDS: u64 = 3600;
pub(super) const DEFAULT_EXPIRED_TIMEOUT_SECONDS: u64 = 60;
pub(super) const DEFAULT_SYNC_INTERVAL_SECONDS: u64 = 24 * 3600;
pub(super) const DEFAULT_SYNC_TIMEOUT_SECONDS: u64 = 300;
pub(super) const DEFAULT_DOWNLOAD_INTERVAL_SECONDS: u64 = 3600;
pub(super) const DEFAULT_DOWNLOAD_TIMEOUT_SECONDS: u64 = 300;
pub(super) const DEFAULT_ASSET_INTERVAL_SECONDS: u64 = 6 * 3600;
pub(super) const DEFAULT_ASSET_TIMEOUT_SECONDS: u64 = 900;
pub(super) const DEFAULT_BACKUP_INTERVAL_SECONDS: u64 = 15 * 60;
pub(super) const DEFAULT_BACKUP_TIMEOUT_SECONDS: u64 = 60;

pub(super) const DEFAULT_SESSION_RETENTION_HOURS: u64 = 7 * 24;
pub(super) const DEFAULT_SYNC_ACTIVE_DAYS: u64 = 180;
pub(super) const DEFAULT_SYNC_RETENTION_DAYS: u64 = 365;
pub(super) const DEFAULT_BACKUP_FRESHNESS_HOURS: u64 = 26;
pub(super) const DEFAULT_SHUTDOWN_SECONDS: u64 = 30;
pub(super) const DEFAULT_BACKUP_ROOT: &str = "./backups";

pub(super) const TASK_INTERVAL_SECONDS: RangeInclusive<u64> = 60..=7 * 24 * 3600;
pub(super) const TASK_TIMEOUT_SECONDS: RangeInclusive<u64> = 10..=3600;
pub(super) const SESSION_RETENTION_HOURS: RangeInclusive<u64> = 24..=10 * 365 * 24;
pub(super) const SYNC_WINDOW_DAYS: RangeInclusive<u64> = 30..=10 * 365;
pub(super) const BACKUP_FRESHNESS_HOURS: RangeInclusive<u64> = 1..=30 * 24;
pub(super) const SHUTDOWN_SECONDS: RangeInclusive<u64> = 5..=30;
