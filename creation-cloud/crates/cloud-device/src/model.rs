//! 定义设备平台、状态和 JSON API 返回模型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    Android,
    Ios,
    Macos,
    Web,
}

impl Platform {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Macos => "macos",
            Self::Web => "web",
        }
    }
}

pub(crate) type DeviceRow = (
    Uuid,
    Uuid,
    String,
    String,
    String,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
    DateTime<Utc>,
);

#[derive(Clone, Debug, Serialize)]
pub struct Device {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub platform: String,
    pub public_id: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateDeviceOutcome {
    pub device: Device,
    pub created: bool,
    pub session: DeviceSessionView,
    pub(crate) raw_token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceSessionView {
    pub session_id: Uuid,
    pub account_id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub status: String,
    pub is_current: bool,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub device_fingerprint: Option<String>,
    pub session_kind: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub csrf_token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SessionView {
    pub session_id: Uuid,
    pub status: String,
    pub is_current: bool,
    pub account_id: Uuid,
    pub account_label: String,
    pub device_id: Option<Uuid>,
    pub device_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub device_fingerprint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceSessionResult {
    pub device: Device,
    pub session: DeviceSessionView,
}

impl Device {
    pub(crate) fn from_row(row: DeviceRow) -> Self {
        Self {
            id: row.0,
            account_id: row.1,
            name: row.2,
            platform: row.3,
            public_id: row.4,
            last_seen_at: row.5,
            revoked_at: row.6,
            created_at: row.7,
            updated_at: row.8,
        }
    }
}
