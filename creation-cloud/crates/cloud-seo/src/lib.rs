//! 管理 Creation Cloud 的全局双语 SEO 主题词。
//! 主题词只作为公开页面内容组织输入，不等同于搜索引擎收录承诺。

mod authorization;
mod model;
mod repository;
mod router;
mod service;
mod use_case;
mod validation;

#[cfg(test)]
mod tests;

pub use model::{CreateSeoTopicInput, SeoLocale, SeoTopic, UpdateSeoTopicInput};
pub use router::management_router;
pub use service::Service;
