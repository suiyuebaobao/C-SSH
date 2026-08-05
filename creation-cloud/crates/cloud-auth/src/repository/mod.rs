//! 声明按认证动作拆分的 PostgreSQL repository。

pub(crate) mod captcha;
pub(crate) mod change_password;
pub(crate) mod cleanup_expired_sessions;
pub(crate) mod error;
pub(crate) mod login;
pub(crate) mod login_verification;
pub(crate) mod logout;
pub(crate) mod password_reset;
pub(crate) mod register;
pub(crate) mod session;
pub(crate) mod settings;
pub(crate) mod verification;
