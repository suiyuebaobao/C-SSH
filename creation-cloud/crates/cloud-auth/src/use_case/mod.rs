//! 声明并重导出按认证动作拆分的业务用例。

pub(crate) mod auth_settings;
pub(crate) mod captcha;
pub(crate) mod change_password;
pub(crate) mod cleanup_expired_sessions;
pub(crate) mod login;
pub(crate) mod login_lockout;
pub(crate) mod logout;
pub(crate) mod register;
pub(crate) mod request_password_reset;
pub(crate) mod resend_login_verification;
pub(crate) mod resend_verification;
pub(crate) mod reset_password;
pub(crate) mod session;
pub(crate) mod verify_email;
pub(crate) mod verify_login;

pub use auth_settings::{
    AuthSettings, ClientLoginConfig, LoginCaptchaSettings, UpdateAuthSettings,
};
pub use change_password::ChangePassword;
pub use login::{Login, LoginOutcome, LoginVerificationRequired};
pub use register::{Register, RegistrationOutcome, RegistrationStatus};
pub use request_password_reset::{PasswordResetVerificationRequired, RequestPasswordReset};
pub use resend_login_verification::ResendLoginVerification;
pub use resend_verification::{ResendStatus, ResendVerification};
pub use reset_password::ResetPassword;
pub use verify_email::VerifyEmail;
pub use verify_login::VerifyLogin;
