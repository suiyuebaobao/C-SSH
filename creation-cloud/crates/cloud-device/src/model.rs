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
