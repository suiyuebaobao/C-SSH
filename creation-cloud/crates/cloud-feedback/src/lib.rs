//! 提供登录账号反馈创建、本人查询与管理员安全处理的独立领域。
//! 本域只保存受限纯文本，不采集 IP、UA、附件、凭据或额外邮箱。

mod authorization;
mod handler;
mod model;
mod repository;
mod router;
mod service;
mod use_case;
mod validation;

pub use model::{
    AdminFeedbackDetail, AdminFeedbackListQuery, AdminFeedbackSummary, CreateFeedbackInput,
    FeedbackCategory, FeedbackOverview, FeedbackPlatform, FeedbackStatus, FeedbackSubmission,
    RedactFeedbackInput, UpdateFeedbackStatusInput,
};
pub use router::{management_router, user_router};
pub use service::Service;

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod tests;
