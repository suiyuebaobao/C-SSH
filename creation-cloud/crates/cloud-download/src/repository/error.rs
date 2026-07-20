//! 将下载域数据库错误映射为统一且脱敏的应用错误。

use cloud_domain::AppError;

pub(crate) fn map_read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取下载数据失败".into())
}

pub(crate) fn map_transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("提交下载事务失败".into())
}

pub(crate) fn map_write_error(error: sqlx::Error, conflict: &str) -> AppError {
    if error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| matches!(code.as_ref(), "23503" | "23505" | "P0001"))
    {
        return AppError::Conflict(conflict.into());
    }
    AppError::Storage("写入下载数据失败".into())
}
