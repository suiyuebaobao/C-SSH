//! 管理员全局模型目录。
//!
//! 普通用户只能读取启用目录；模型元数据写入仅由管理员路由提供。
//! AI API Key/Token 只保存在客户端本地，Cloud 不接收明文或密文。

mod repository;
mod router;
mod service;
mod types;
mod validation;

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod seed_tests;

pub use router::{management_router, router};
pub use service::Service;
pub use types::{
    CreateGlobalModelInput, DeleteGlobalModelInput, GlobalModel, ModelInterface, PublicGlobalModel,
    ReplaceGlobalModelInput,
};
