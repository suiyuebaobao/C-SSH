//! 声明按认证动作拆分的 HTTP handler。

pub(crate) mod change_password;
pub(crate) mod form_login;
pub(crate) mod form_register;
pub(crate) mod form_resend_verification;
pub(crate) mod form_response;
pub(crate) mod form_verify_email;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod register;
pub(crate) mod resend_verification;
pub(crate) mod session;
pub(crate) mod verify_email;
