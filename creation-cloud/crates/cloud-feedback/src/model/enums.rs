//! 定义反馈类别、来源平台与受控处理状态。

use cloud_domain::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackCategory {
    Bug,
    Feature,
    Docs,
    Compatibility,
    Other,
}

impl FeedbackCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Docs => "docs",
            Self::Compatibility => "compatibility",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for FeedbackCategory {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "docs" => Ok(Self::Docs),
            "compatibility" => Ok(Self::Compatibility),
            "other" => Ok(Self::Other),
            _ => Err(AppError::Internal("数据库中的反馈类别无效".to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackPlatform {
    Windows,
    Linux,
    Android,
    Macos,
    Ios,
    Cloud,
    Agent,
    Other,
}

impl FeedbackPlatform {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Macos => "macos",
            Self::Ios => "ios",
            Self::Cloud => "cloud",
            Self::Agent => "agent",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for FeedbackPlatform {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            "android" => Ok(Self::Android),
            "macos" => Ok(Self::Macos),
            "ios" => Ok(Self::Ios),
            "cloud" => Ok(Self::Cloud),
            "agent" => Ok(Self::Agent),
            "other" => Ok(Self::Other),
            _ => Err(AppError::Internal("数据库中的反馈平台无效".to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackStatus {
    New,
    Triaged,
    InProgress,
    Resolved,
    Closed,
}

impl FeedbackStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Triaged => "triaged",
            Self::InProgress => "in_progress",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

impl TryFrom<&str> for FeedbackStatus {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "new" => Ok(Self::New),
            "triaged" => Ok(Self::Triaged),
            "in_progress" => Ok(Self::InProgress),
            "resolved" => Ok(Self::Resolved),
            "closed" => Ok(Self::Closed),
            _ => Err(AppError::Internal("数据库中的反馈状态无效".to_owned())),
        }
    }
}
