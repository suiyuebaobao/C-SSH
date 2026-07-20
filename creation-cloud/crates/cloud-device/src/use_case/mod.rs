//! 声明并重导出按设备 CRUD 动作拆分的业务用例。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod update;

pub use create::CreateDevice;
pub use update::UpdateDevice;
