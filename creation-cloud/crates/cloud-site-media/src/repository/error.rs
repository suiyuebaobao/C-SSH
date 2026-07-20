//! 将 SQLx 错误收敛为不泄露 SQL、路径和数据库细节的领域错误。

use cloud_domain::AppError;

pub(crate) fn map_read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取站点媒体数据失败".into())
}

pub(crate) fn map_write_error(error: sqlx::Error, conflict: &str) -> AppError {
    if error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| matches!(code.as_ref(), "23505" | "23514" | "P0001"))
    {
        return AppError::Conflict(conflict.into());
    }
    AppError::Storage("写入站点媒体数据失败".into())
}
