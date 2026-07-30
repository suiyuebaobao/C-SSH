//! 邮箱验证码发送边界。实现方只能消费本次投递所需的邮箱和验证码。

use std::{future::Future, pin::Pin};

use cloud_domain::AppResult;

pub type MailerFuture<'a> = Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationPurpose {
    Registration,
    Login,
}

pub trait VerificationMailer: Send + Sync {
    fn send_verification<'a>(
        &'a self,
        email: &'a str,
        code: &'a str,
        purpose: VerificationPurpose,
    ) -> MailerFuture<'a>;
}

#[derive(Default)]
pub(crate) struct UnavailableVerificationMailer;

impl VerificationMailer for UnavailableVerificationMailer {
    fn send_verification<'a>(
        &'a self,
        _email: &'a str,
        _code: &'a str,
        _purpose: VerificationPurpose,
    ) -> MailerFuture<'a> {
        Box::pin(async {
            Err(cloud_domain::AppError::Unavailable(
                "邮箱验证码发送服务尚未配置".to_owned(),
            ))
        })
    }
}
