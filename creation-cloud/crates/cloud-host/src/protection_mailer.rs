//! Narrow mail delivery boundary for the independent protection-reset purpose.

use std::{future::Future, pin::Pin};

use cloud_domain::{AppError, AppResult};

pub type ProtectionResetMailerFuture<'a> = Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;

pub trait ProtectionResetMailer: Send + Sync {
    fn send_protection_reset<'a>(
        &'a self,
        email: &'a str,
        code: &'a str,
    ) -> ProtectionResetMailerFuture<'a>;
}

#[derive(Default)]
pub(crate) struct UnavailableProtectionResetMailer;

impl ProtectionResetMailer for UnavailableProtectionResetMailer {
    fn send_protection_reset<'a>(
        &'a self,
        _email: &'a str,
        _code: &'a str,
    ) -> ProtectionResetMailerFuture<'a> {
        Box::pin(async {
            Err(AppError::Unavailable(
                "数据保护清空邮箱验证服务尚未配置".to_owned(),
            ))
        })
    }
}
