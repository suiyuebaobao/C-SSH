//! 将数据库错误收敛为不泄露 SQL 与内部细节的领域错误。

use cloud_domain::AppError;

pub(crate) fn storage(_: sqlx::Error) -> AppError {
    AppError::Storage("数据库操作失败".to_owned())
}

pub(crate) fn create_account(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("无法创建账号".to_owned())
    } else {
        storage(error)
    }
}
