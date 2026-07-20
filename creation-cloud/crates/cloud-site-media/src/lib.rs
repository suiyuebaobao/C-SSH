//! 管理 Creation Cloud 官网受控同源媒体及首页二维码发布状态。
//! 本领域不解析二维码载荷，也不连接 SSH 数据面或其它业务领域。

mod finalization;
mod handler;
mod image_validation;
mod model;
mod multipart;
mod readiness;
mod repository;
mod router;
mod service;
mod storage;
mod use_case;
mod validation;

#[cfg(test)]
mod image_validation_tests;
#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod router_tests;
#[cfg(test)]
mod storage_tests;

pub use model::{
    CreateSiteMediaInput, PublicHomeQr, SiteMedia, SiteMediaSlot, SiteMediaState,
    UpdateSiteMediaInput,
};
pub use router::{management_router, public_router};
pub use service::Service;
