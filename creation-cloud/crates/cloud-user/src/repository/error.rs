//! 将用户资料数据库错误映射为稳定且脱敏的领域错误。

use cloud_domain::AppError;

pub(crate) fn storage(_: sqlx::Error) -> AppError {
    AppError::Storage("数据库操作失败".to_owned())
}

pub(crate) fn create(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("用户资料已存在".to_owned())
    } else {
        storage(error)
    }
}
