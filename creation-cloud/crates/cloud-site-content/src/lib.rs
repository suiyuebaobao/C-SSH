//! 管理首页结构化内容的草稿、发布快照、预览与历史。

mod model;
mod repository;
mod router;
mod service;
mod validation;

pub use model::{
    CreateSiteContentInput, PublicSiteContent, SiteContentDocumentKey, SiteContentListQuery,
    SiteContentPayload, SiteContentRevision, SiteContentState, SiteContentTransitionInput,
    UpdateSiteContentInput,
};
pub use router::management_router;
pub use service::Service;

#[cfg(test)]
mod tests;
