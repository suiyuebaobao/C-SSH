//! 声明按认证动作拆分的 PostgreSQL repository。

pub(crate) mod change_password;
pub(crate) mod error;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod register;
pub(crate) mod session;
