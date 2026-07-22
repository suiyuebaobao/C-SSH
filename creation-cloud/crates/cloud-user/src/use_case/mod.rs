//! 声明并重导出按用户资料创建、读取、列表与更新动作拆分的业务用例。

pub(crate) mod create;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod ownership;
pub(crate) mod update;

pub use create::CreateProfile;
pub use update::UpdateProfile;
