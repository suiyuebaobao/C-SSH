//! 更新单个用户资料并返回数据库权威值。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Profile, model::ProfileRow};

use super::error;

pub(crate) async fn apply(
    pool: &PgPool,
    account_id: Uuid,
    display_name: Option<&str>,
    locale: Option<&str>,
) -> AppResult<Option<Profile>> {
    sqlx::query_as::<_, ProfileRow>(
        "UPDATE user_profiles \
         SET display_name = COALESCE($2, display_name), \
             locale = COALESCE($3, locale), updated_at = now() \
         WHERE account_id = $1 \
         RETURNING account_id, display_name, locale, created_at, updated_at",
    )
    .bind(account_id)
    .bind(display_name)
    .bind(locale)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Profile::from_row))
    .map_err(error::storage)
}
