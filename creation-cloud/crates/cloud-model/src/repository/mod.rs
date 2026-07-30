//! PostgreSQL 持久化边界。

mod catalog;
mod secret;
mod seed;

pub(crate) use catalog::{create, delete, get_admin, get_public, list_admin, list_public, replace};
pub(crate) use secret::{delete_secret, get_secret, put_secret};
pub(crate) use seed::seed_system_catalog;
#[cfg(test)]
pub(crate) use seed::{ACTIVE_ADMIN_SQL, INSERT_SEED_SQL, LOCK_CATALOG_SQL, SYSTEM_MODEL_SEEDS};

use cloud_domain::AppError;

fn storage(context: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |error| {
        if error
            .as_database_error()
            .and_then(|database| database.code())
            .is_some_and(|code| code.as_ref() == "23505")
        {
            AppError::Conflict("模型名称或默认项与现有记录冲突".to_owned())
        } else {
            AppError::Storage(context.to_owned())
        }
    }
}
