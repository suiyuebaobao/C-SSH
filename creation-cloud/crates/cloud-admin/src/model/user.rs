//! 定义管理端脱敏 API 用户响应及页面完整邮箱投影。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, PageQuery};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
#[serde(rename_all = "snake_case")]
pub enum AdminUserStatus {
    PendingVerification,
    Active,
    Disabled,
}

impl AdminUserStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PendingVerification => "pending_verification",
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for AdminUserStatus {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "pending_verification" => Ok(Self::PendingVerification),
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            _ => Err(AppError::Internal("数据库中的账号状态无效".to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct AdminUserListQuery {
    pub page: PageQuery,
    pub search: Option<String>,
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
    search: Option<String>,
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
            search: wire.search,
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
    pub search: Option<String>,
    pub email: Option<String>,
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminCreateUserInput {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
    pub admin_login_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminUpdateUserInput {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub admin_login_name: Option<String>,
    #[serde(default)]
    pub clear_admin_login_name: bool,
    pub role: Option<AdminUserRole>,
    pub status: Option<AdminUserStatus>,
    pub new_password: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub masked_email: String,
    #[serde(skip_serializing)]
    full_email: Option<String>,
    pub admin_login_name: Option<String>,
    pub email_verified: bool,
    pub display_name: String,
    pub role: AdminUserRole,
    pub status: AdminUserStatus,
    pub device_count: i64,
    pub host_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct AdminUserRow {
    pub id: Uuid,
    pub email: Option<String>,
    pub admin_login_name: Option<String>,
    pub email_verified: bool,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub device_count: i64,
    pub host_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AdminUserRow> for AdminUser {
    type Error = AppError;

    fn try_from(row: AdminUserRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            masked_email: row
                .email
                .as_deref()
                .map(crate::redaction::email)
                .unwrap_or_else(|| "—".to_owned()),
            full_email: row.email,
            admin_login_name: row.admin_login_name,
            email_verified: row.email_verified,
            display_name: row.display_name,
            role: AdminUserRole::try_from(row.role.as_str())?,
            status: AdminUserStatus::try_from(row.status.as_str())?,
            device_count: row.device_count,
            host_count: row.host_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl AdminUser {
    #[must_use]
    pub fn email_for_admin_page(&self) -> &str {
        self.full_email.as_deref().unwrap_or("—")
    }
}

#[cfg(test)]
mod tests {
    use super::{AdminUser, AdminUserRow};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn admin_user_keeps_the_complete_email() {
        let email = "person@example.com";
        let user = AdminUser::try_from(AdminUserRow {
            id: Uuid::nil(),
            email: Some(email.to_owned()),
            admin_login_name: None,
            email_verified: true,
            display_name: "Person".to_owned(),
            role: "user".to_owned(),
            status: "active".to_owned(),
            device_count: 1,
            host_count: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .expect("有效用户投影应转换成功");

        assert_eq!(user.email_for_admin_page(), email);
        assert_eq!(user.masked_email, "p***@e***.com");
        let json = serde_json::to_value(user).expect("管理用户应可序列化");
        assert_eq!(json["masked_email"], "p***@e***.com");
        assert!(json.get("full_email").is_none());
        assert!(json.get("email").is_none());
    }

    #[test]
    fn admin_user_without_email_uses_a_neutral_marker() {
        let user = AdminUser::try_from(AdminUserRow {
            id: Uuid::nil(),
            email: None,
            admin_login_name: Some("ops-admin".to_owned()),
            email_verified: false,
            display_name: "Administrator".to_owned(),
            role: "admin".to_owned(),
            status: "active".to_owned(),
            device_count: 0,
            host_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .expect("无邮箱管理员投影应转换成功");

        assert_eq!(user.email_for_admin_page(), "—");
        assert_eq!(user.masked_email, "—");
    }
}
