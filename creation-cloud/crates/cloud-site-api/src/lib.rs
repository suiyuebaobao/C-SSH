//! 网站公开内容JSON适配层；不包含HTML模板、静态资源或网页路由。

mod catalog;
mod manual_downloads;
mod public_api;
mod release_notes;
mod seo;
mod state;

use axum::{Router, routing::get};
pub use catalog::PublishedCatalogView;
pub use release_notes::{NotesBlock, parse as parse_notes};
pub use seo::{SeoConfig, SeoHead};
pub use state::SiteState;

#[must_use = "路由须挂载到统一API前缀"]
pub fn router(state: SiteState) -> Router {
    Router::new()
        .route("/pages/{page}", get(public_api::page))
        .with_state(state)
}

/// 仅用于不连接数据库的隔离原生预览，不挂到生产build。
#[must_use = "预览路由须显式挂载"]
pub fn preview_router(config: SeoConfig) -> Router {
    Router::new()
        .route("/api/v1/site/pages/{page}", get(public_api::preview))
        .with_state(config)
}
