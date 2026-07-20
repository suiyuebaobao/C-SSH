//! 声明并重导出反馈枚举、输入、持久化行与对外响应模型。

mod admin;
mod enums;
mod input;
mod overview;
mod query;
mod row;
mod submission;

pub use admin::{AdminFeedbackDetail, AdminFeedbackSummary};
pub use enums::{FeedbackCategory, FeedbackPlatform, FeedbackStatus};
pub use input::{CreateFeedbackInput, RedactFeedbackInput, UpdateFeedbackStatusInput};
pub use overview::FeedbackOverview;
pub use query::AdminFeedbackListQuery;
pub(crate) use row::{FeedbackRow, FeedbackSummaryRow};
pub use submission::FeedbackSubmission;
