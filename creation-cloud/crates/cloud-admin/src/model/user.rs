//! 定义管理端脱敏用户响应、筛选条件以及角色和状态变更输入。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, PageQuery};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::redaction;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdminUserRole {
    User,
    Admin,
}

impl AdminUserRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
        }
    }
}

impl TryFrom<&str> for AdminUserRole {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "user" => Ok(Self::User),
            "admin" => Ok(Self::Admin),
            _ => Err(AppError::Internal("数据库中的账号角色无效".to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdminUserStatus {
    Active,
    Disabled,
}

impl AdminUserStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for AdminUserStatus {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            _ => Err(AppError::Internal("数据库中的账号状态无效".to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct AdminUserListQuery {
    pub page: PageQuery,
    pub email: Option<String>,
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminUserListQueryWire {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_size")]
    size: u32,
    email: Option<String>,
    role: Option<AdminUserRole>,
    status: Option<AdminUserStatus>,
}

impl<'de> Deserialize<'de> for AdminUserListQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AdminUserListQueryWire::deserialize(deserializer)?;
        Ok(Self {
            page: PageQuery {
                page: wire.page,
                size: wire.size,
            },
            email: wire.email,
            role: wire.role,
            status: wire.status,
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
pub(crate) struct AdminUserListFilter {
    pub page: PageQuery,
    pub email: Option<String>,
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminUpdateUserInput {
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub masked_email: String,
    pub display_name: String,
    pub role: AdminUserRole,
    pub status: AdminUserStatus,
    pub device_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct AdminUserRow {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub device_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AdminUserRow> for AdminUser {
    type Error = AppError;

    fn try_from(row: AdminUserRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            masked_email: redaction::email(&row.email),
            display_name: row.display_name,
            role: AdminUserRole::try_from(row.role.as_str())?,
            status: AdminUserStatus::try_from(row.status.as_str())?,
            device_count: row.device_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
