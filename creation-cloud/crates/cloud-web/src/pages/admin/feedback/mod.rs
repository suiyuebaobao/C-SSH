//! 装配管理后台反馈列表、详情与受控写操作页面模块。
//! 各处理器只调用反馈领域公开用例，不直接访问数据库或回显列表正文。

mod list;
mod query;
pub(crate) mod redact;
pub(crate) mod status;
mod view;

pub(crate) use list::page;

#[cfg(test)]
mod tests;
