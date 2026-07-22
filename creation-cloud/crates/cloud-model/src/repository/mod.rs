//! 汇总模型资源按 CRUD 动作拆分的 PostgreSQL 仓储函数。

mod create;
mod delete;
mod get;
mod list;
mod row;
mod update;

pub(crate) use create::create;
pub(crate) use delete::delete;
pub(crate) use get::get;
pub(crate) use list::list;
pub(crate) use row::{ModelRow, model_from_row};
pub(crate) use update::update;

use cloud_domain::AppError;

pub(crate) fn storage(message: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |_| AppError::Storage(message.to_owned())
}

pub(crate) fn write_error(error: sqlx::Error) -> AppError {
    if matches!(
        &error,
        sqlx::Error::Database(database)
            if database.code().as_deref() == Some("23503")
                && database.constraint() == Some("model_profiles_active_vault_envelope_fkey")
    ) {
        return AppError::NotFound("密文信封不存在".to_owned());
    }
    if matches!(
        &error,
        sqlx::Error::Database(database) if database.code().as_deref() == Some("23505")
    ) {
        AppError::Conflict("同一账号下的模型名称已存在".to_owned())
    } else {
        AppError::Storage("无法写入模型元数据".to_owned())
    }
}
