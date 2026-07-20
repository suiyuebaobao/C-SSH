//! 提供客户端加密后密文信封的独立 CRUD 与相对 Axum 路由。
//! 服务端只校验信封结构并持久化字节，不派生密钥、不解密也不接收明文。

mod handler;
mod repository;
mod router;
mod service;
mod types;
mod use_case;
mod validation;

pub use router::router;
pub use service::Service;
pub use types::{
    CreateVaultEnvelopeInput, DeleteVaultInput, DeleteVaultOutcome, KdfMetadata,
    UpdateVaultEnvelopeInput, VaultEnvelope,
};

#[cfg(test)]
mod tests;
