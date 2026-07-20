//! 声明管理员反馈摘要、详情、状态更新和不可逆脱敏用例。

pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod overview;
pub(crate) mod redact;
pub(crate) mod status;

#[cfg(test)]
mod tests;
