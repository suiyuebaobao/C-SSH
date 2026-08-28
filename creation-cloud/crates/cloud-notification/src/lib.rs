//! 提供账号安全与同步通知的不可变投递、稳定分页和跨设备已读回执。

mod event;
mod model;
mod repository;
mod router;
mod service;
mod validation;

#[cfg(test)]
mod tests;

pub use event::{AccountNotificationEvent, record_account_event};
pub use model::{
    AccountNotification, AdminNotification, CreateNotificationInput, NotificationKind,
    NotificationListQuery, NotificationListResponse, NotificationPriority, NotificationReceipt,
    ReceiptInput,
};
pub use router::{account_router, management_router};
pub use service::Service;
