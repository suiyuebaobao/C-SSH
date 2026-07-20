//! 把 SQLx 错误收敛为不泄露 SQL、路径或数据库内部信息的领域错误。

use cloud_domain::AppError;

pub(crate) fn read(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取反馈数据失败".to_owned())
}

pub(crate) fn write(error: sqlx::Error) -> AppError {
    if error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| matches!(code.as_ref(), "23503" | "23505" | "23514" | "P0001"))
    {
        return AppError::Conflict("反馈写入发生冲突".to_owned());
    }
    AppError::Storage("写入反馈数据失败".to_owned())
}

pub(crate) fn transaction(_error: sqlx::Error) -> AppError {
    AppError::Storage("提交反馈事务失败".to_owned())
}
