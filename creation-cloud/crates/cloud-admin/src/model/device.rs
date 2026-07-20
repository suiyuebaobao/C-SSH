//! 定义管理端设备筛选和只暴露非 SSH 元数据的脱敏响应。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, PageQuery};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::redaction;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdminDevicePlatform {
    Windows,
    Linux,
    Android,
    Ios,
    Macos,
    Web,
}

impl AdminDevicePlatform {
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

impl TryFrom<&str> for AdminDevicePlatform {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            "macos" => Ok(Self::Macos),
            "web" => Ok(Self::Web),
            _ => Err(AppError::Internal("数据库中的设备平台无效".to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct AdminDeviceListQuery {
    pub page: PageQuery,
    pub account_id: Option<Uuid>,
    pub platform: Option<AdminDevicePlatform>,
    pub revoked: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminDeviceListQueryWire {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_size")]
    size: u32,
    account_id: Option<Uuid>,
    platform: Option<AdminDevicePlatform>,
    revoked: Option<bool>,
}

impl<'de> Deserialize<'de> for AdminDeviceListQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AdminDeviceListQueryWire::deserialize(deserializer)?;
        Ok(Self {
            page: PageQuery {
                page: wire.page,
                size: wire.size,
            },
            account_id: wire.account_id,
            platform: wire.platform,
            revoked: wire.revoked,
        })
    }
}

const fn default_page() -> u32 {
    1
}

const fn default_size() -> u32 {
    20
}

#[derive(Clone, Debug)]
pub(crate) struct AdminDeviceListFilter {
    pub page: PageQuery,
    pub account_id: Option<Uuid>,
    pub platform: Option<AdminDevicePlatform>,
    pub revoked: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminDevice {
    pub id: Uuid,
    pub account_id: Uuid,
    pub owner_masked_email: String,
    pub name: String,
    pub platform: AdminDevicePlatform,
    pub public_id: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct AdminDeviceRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub owner_email: String,
    pub name: String,
    pub platform: String,
    pub public_id: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AdminDeviceRow> for AdminDevice {
    type Error = AppError;

    fn try_from(row: AdminDeviceRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            account_id: row.account_id,
            owner_masked_email: redaction::email(&row.owner_email),
            name: row.name,
            platform: AdminDevicePlatform::try_from(row.platform.as_str())?,
            public_id: row.public_id,
            last_seen_at: row.last_seen_at,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
