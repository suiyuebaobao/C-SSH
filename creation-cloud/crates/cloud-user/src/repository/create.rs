//! 插入单个用户资料并返回数据库权威值。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Profile, model::ProfileRow};

use super::error;

pub(crate) async fn insert(
    pool: &PgPool,
    account_id: Uuid,
    display_name: &str,
    locale: &str,
) -> AppResult<Profile> {
    sqlx::query_as::<_, ProfileRow>(
        "INSERT INTO user_profiles (account_id, display_name, locale) \
         VALUES ($1, $2, $3) \
         RETURNING account_id, display_name, locale, created_at, updated_at",
    )
    .bind(account_id)
    .bind(display_name)
    .bind(locale)
    .fetch_one(pool)
    .await
    .map(Profile::from_row)
    .map_err(error::create)
}
