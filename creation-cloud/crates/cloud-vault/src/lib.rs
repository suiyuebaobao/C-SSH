//! 提供客户端密文信封与账号级 master-key 包装密文的独立 CRUD 路由。
//! 服务端只校验密文结构并持久化字节，不派生密钥、不解密也不接收明文。

mod handler;
mod repository;
mod router;
mod service;
mod types;
mod use_case;
mod validation;
mod wrapper_types;
mod wrapper_validation;

pub use router::router;
pub use service::Service;
pub use types::{
    CreateVaultEnvelopeInput, DeleteVaultInput, DeleteVaultOutcome, KdfMetadata,
    UpdateVaultEnvelopeInput, VaultEnvelope,
};
pub use wrapper_types::{
    CreateVaultKeyWrapperInput, DeleteVaultKeyWrapperInput, DeleteVaultKeyWrapperOutcome,
    UpdateVaultKeyWrapperInput, VaultKeyWrapper,
};

#[cfg(test)]
mod tests;
