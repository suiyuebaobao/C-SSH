//! 管理员全局模型目录与账号级客户端原样密文。
//!
//! 普通用户只能读取启用目录；模型元数据写入仅由管理员路由提供。
//! API Key/Token 密文由客户端生成，服务端没有解密密钥。

mod repository;
mod router;
mod service;
mod types;
mod validation;

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod seed_tests;

pub use router::{management_router, router, secret_router};
pub use service::Service;
pub use types::{
    CreateGlobalModelInput, DeleteGlobalModelInput, DeleteModelSecretInput, GlobalModel,
    ModelSecret, PutModelSecretInput, ReplaceGlobalModelInput,
};
