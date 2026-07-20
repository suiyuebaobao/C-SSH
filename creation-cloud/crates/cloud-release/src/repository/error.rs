//! 将 SQLx 错误收敛为不泄露 SQL 与数据库细节的统一错误。

use cloud_domain::AppError;

pub(crate) fn map_read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取版本数据失败".into())
}

pub(crate) fn map_transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("提交版本事务失败".into())
}

pub(crate) fn map_write_error(error: sqlx::Error, conflict: &str) -> AppError {
    if error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| is_conflict_code(code.as_ref()))
    {
        return AppError::Conflict(conflict.into());
    }
    AppError::Storage("写入版本数据失败".into())
}

fn is_conflict_code(code: &str) -> bool {
    matches!(code, "23503" | "23505" | "P0001")
}

#[cfg(test)]
mod tests {
    use super::is_conflict_code;

    #[test]
    fn maps_foreign_key_unique_and_guard_failures_to_conflict() {
        assert!(is_conflict_code("23503"));
        assert!(is_conflict_code("23505"));
        assert!(is_conflict_code("P0001"));
    }
}
