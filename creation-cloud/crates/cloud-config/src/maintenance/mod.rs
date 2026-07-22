//! 读取五项维护任务的生产周期、超时、保留窗口和备份只读边界。

mod bounds;
mod parse;

use std::{ffi::OsString, path::PathBuf, time::Duration};

use anyhow::{Result, bail};

use bounds::*;
use parse::{duration_from, number_from};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskSchedule {
    pub interval: Duration,
    pub timeout: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceConfig {
    pub expired_sessions: TaskSchedule,
    pub sync_retention: TaskSchedule,
    pub download_aggregation: TaskSchedule,
    pub published_asset_inspection: TaskSchedule,
    pub backup_freshness: TaskSchedule,
    pub expired_session_retention: Duration,
    pub sync_active_window: Duration,
    pub sync_retention_window: Duration,
    pub backup_root: PathBuf,
    pub backup_freshness_window: Duration,
    pub shutdown_timeout: Duration,
    pub cleanup_batch_size: u32,
}

impl MaintenanceConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var_os(name))
    }

    fn from_lookup(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Result<Self> {
        let expired_sessions = schedule(
            &mut lookup,
            "CLOUD_MAINTENANCE_EXPIRED_SESSIONS_INTERVAL_SECONDS",
            DEFAULT_EXPIRED_INTERVAL_SECONDS,
            "CLOUD_MAINTENANCE_EXPIRED_SESSIONS_TIMEOUT_SECONDS",
            DEFAULT_EXPIRED_TIMEOUT_SECONDS,
        )?;
        let sync_retention = schedule(
            &mut lookup,
            "CLOUD_MAINTENANCE_SYNC_RETENTION_INTERVAL_SECONDS",
            DEFAULT_SYNC_INTERVAL_SECONDS,
            "CLOUD_MAINTENANCE_SYNC_RETENTION_TIMEOUT_SECONDS",
            DEFAULT_SYNC_TIMEOUT_SECONDS,
        )?;
        let download_aggregation = schedule(
            &mut lookup,
            "CLOUD_MAINTENANCE_DOWNLOAD_AGGREGATION_INTERVAL_SECONDS",
            DEFAULT_DOWNLOAD_INTERVAL_SECONDS,
            "CLOUD_MAINTENANCE_DOWNLOAD_AGGREGATION_TIMEOUT_SECONDS",
            DEFAULT_DOWNLOAD_TIMEOUT_SECONDS,
        )?;
        let published_asset_inspection = schedule(
            &mut lookup,
            "CLOUD_MAINTENANCE_ASSET_INSPECTION_INTERVAL_SECONDS",
            DEFAULT_ASSET_INTERVAL_SECONDS,
            "CLOUD_MAINTENANCE_ASSET_INSPECTION_TIMEOUT_SECONDS",
            DEFAULT_ASSET_TIMEOUT_SECONDS,
        )?;
        let backup_freshness = schedule(
            &mut lookup,
            "CLOUD_MAINTENANCE_BACKUP_FRESHNESS_INTERVAL_SECONDS",
            DEFAULT_BACKUP_INTERVAL_SECONDS,
            "CLOUD_MAINTENANCE_BACKUP_FRESHNESS_TIMEOUT_SECONDS",
            DEFAULT_BACKUP_TIMEOUT_SECONDS,
        )?;
        let expired_session_retention = duration_from(
            &mut lookup,
            "CLOUD_SESSION_RETENTION_HOURS",
            DEFAULT_SESSION_RETENTION_HOURS,
            SESSION_RETENTION_HOURS,
            3600,
        )?;
        let active_days = number_from(
            &mut lookup,
            "CLOUD_SYNC_ACTIVE_WINDOW_DAYS",
            DEFAULT_SYNC_ACTIVE_DAYS,
            SYNC_WINDOW_DAYS,
        )?;
        let retention_days = number_from(
            &mut lookup,
            "CLOUD_SYNC_RETENTION_DAYS",
            DEFAULT_SYNC_RETENTION_DAYS,
            SYNC_WINDOW_DAYS,
        )?;
        if retention_days < active_days {
            bail!("CLOUD_SYNC_RETENTION_DAYS 不得小于 CLOUD_SYNC_ACTIVE_WINDOW_DAYS");
        }
        let backup_freshness_hours = number_from(
            &mut lookup,
            "CLOUD_BACKUP_FRESHNESS_HOURS",
            DEFAULT_BACKUP_FRESHNESS_HOURS,
            BACKUP_FRESHNESS_HOURS,
        )?;
        let shutdown_seconds = number_from(
            &mut lookup,
            "CLOUD_MAINTENANCE_SHUTDOWN_TIMEOUT_SECONDS",
            DEFAULT_SHUTDOWN_SECONDS,
            SHUTDOWN_SECONDS,
        )?;
        let backup_root = lookup("CLOUD_BACKUP_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BACKUP_ROOT));
        if backup_root.as_os_str().is_empty() {
            bail!("CLOUD_BACKUP_ROOT 不能为空");
        }
        Ok(Self {
            expired_sessions,
            sync_retention,
            download_aggregation,
            published_asset_inspection,
            backup_freshness,
            expired_session_retention,
            sync_active_window: days(active_days),
            sync_retention_window: days(retention_days),
            backup_root,
            backup_freshness_window: hours(backup_freshness_hours),
            shutdown_timeout: Duration::from_secs(shutdown_seconds),
            cleanup_batch_size: 1000,
        })
    }
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self::from_lookup(|_| None).expect("固定维护默认配置必须有效")
    }
}

fn schedule(
    lookup: &mut impl FnMut(&str) -> Option<OsString>,
    interval_name: &'static str,
    interval_default: u64,
    timeout_name: &'static str,
    timeout_default: u64,
) -> Result<TaskSchedule> {
    let interval = duration_from(
        &mut *lookup,
        interval_name,
        interval_default,
        TASK_INTERVAL_SECONDS,
        1,
    )?;
    let timeout = duration_from(
        &mut *lookup,
        timeout_name,
        timeout_default,
        TASK_TIMEOUT_SECONDS,
        1,
    )?;
    if timeout > interval {
        bail!("{timeout_name} 不得大于 {interval_name}");
    }
    Ok(TaskSchedule { interval, timeout })
}

fn hours(value: u64) -> Duration {
    Duration::from_secs(value * 3600)
}

fn days(value: u64) -> Duration {
    hours(value * 24)
}

#[cfg(test)]
mod tests;
