//! 将管理域数据库错误映射为统一且脱敏的应用错误。

use cloud_domain::AppError;

pub(crate) fn map_read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取管理数据失败".into())
}

pub(crate) fn map_write_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("写入管理数据失败".into())
}
