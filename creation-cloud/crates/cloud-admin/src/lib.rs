//! 提供管理端只读概览和不可变审计事件接口。
//! 本包通过数据库只读投影视图聚合其它领域，不依赖其它业务包。

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
    AdminDevice, AdminDeviceListQuery, AdminDevicePlatform, AdminOverview, AdminUpdateUserInput,
    AdminUser, AdminUserListQuery, AdminUserRole, AdminUserStatus, AuditEvent, AuditOutcome,
    DeviceOverview, ReleaseOverview, SecurityAuditOverview, UserOverview,
};
pub use router::{router, router_without_overview};
pub use service::{Service, promote_registered_admin};

#[cfg(test)]
mod tests;
