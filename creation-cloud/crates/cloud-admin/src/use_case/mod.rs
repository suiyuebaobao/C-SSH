//! 汇总管理用户、设备、只读审计和概览用例。

mod audit;
mod devices;
mod overview;
mod users;

pub(crate) use audit::record::HttpAuditRecord;
