//! 提供模型配置非敏感元数据的独立 CRUD 与相对 Axum 路由。
//! API Key、Token、密码和其它凭据在触及数据库前被明确拒绝。

mod handler;
mod repository;
mod router;
mod service;
mod types;
mod use_case;
mod validation;

pub use router::router;
pub use service::Service;
pub use types::{CreateModelInput, ModelProfile, UpdateModelInput};

#[cfg(test)]
mod tests;
