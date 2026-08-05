//! 提供管理端用户治理、只读概览和不可变审计事件接口。
//! 用户凭据只复用认证域哈希入口，其余管理动作由本包事务化完成。

mod handler;
mod middleware;
mod model;
mod redaction;
mod repository;
mod router;
mod service;
mod use_case;
mod validation;

pub use middleware::audit::audit_write_requests;
pub use model::{
    AdminCreateUserInput, AdminDevice, AdminDeviceListQuery, AdminDevicePlatform, AdminOverview,
    AdminUpdateUserInput, AdminUser, AdminUserListQuery, AdminUserRole, AdminUserStatus,
    AuditEvent, AuditOutcome, DeviceOverview, ReleaseOverview, SecurityAuditOverview, UserOverview,
};
pub use router::{router, router_without_overview};
pub use service::{Service, create_local_admin, promote_registered_admin};
pub use use_case::set_registered_admin_login;

#[cfg(test)]
mod tests;
