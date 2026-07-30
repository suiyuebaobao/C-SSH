//! 声明并重导出按认证动作拆分的业务用例。

pub(crate) mod admin_captcha;
pub(crate) mod auth_settings;
pub(crate) mod change_password;
pub(crate) mod cleanup_expired_sessions;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod register;
pub(crate) mod resend_login_verification;
pub(crate) mod resend_verification;
pub(crate) mod session;
pub(crate) mod verify_email;
pub(crate) mod verify_login;

pub use auth_settings::{AuthSettings, UpdateAuthSettings};
pub use change_password::ChangePassword;
pub use login::{Login, LoginOutcome, LoginVerificationRequired};
pub use register::{Register, RegistrationOutcome, RegistrationStatus};
pub use resend_login_verification::ResendLoginVerification;
pub use resend_verification::{ResendStatus, ResendVerification};
pub use verify_email::VerifyEmail;
pub use verify_login::VerifyLogin;
