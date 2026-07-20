//! 按账号主键读取单个用户资料。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Profile, model::ProfileRow};

use super::error;

pub(crate) async fn find(pool: &PgPool, account_id: Uuid) -> AppResult<Option<Profile>> {
    sqlx::query_as::<_, ProfileRow>(
        "SELECT account_id, display_name, locale, created_at, updated_at \
         FROM user_profiles WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Profile::from_row))
    .map_err(error::storage)
}
