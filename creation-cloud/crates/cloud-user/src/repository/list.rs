//! 按当前账号范围分页查询资料及总数。

use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Profile, model::ProfileRow};

use super::error;

pub(crate) async fn find(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<Profile>, i64)> {
    let items = sqlx::query_as::<_, ProfileRow>(
        "SELECT account_id, display_name, locale, created_at, updated_at \
         FROM user_profiles WHERE account_id = $1 \
         ORDER BY created_at, account_id LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(error::storage)?;

    let count = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) AS total FROM user_profiles WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(error::storage)?;
    Ok((items.into_iter().map(Profile::from_row).collect(), count.0))
}
