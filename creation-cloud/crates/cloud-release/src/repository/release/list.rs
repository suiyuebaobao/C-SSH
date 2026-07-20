//! 按创建时间倒序分页读取版本目录。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use sqlx::FromRow;

use crate::{Release, model::ReleaseRow, repository::map_read_error};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

pub(crate) async fn execute(pool: &PgPool, query: PageQuery) -> AppResult<Page<Release>> {
    let query = query.normalized();
    let count = sqlx::query_as::<_, CountRow>("SELECT count(*)::BIGINT AS total FROM releases")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)?;
    let rows = sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT id, version, channel, status, title_zh, title_en,
               notes_zh, notes_en, published_at, created_at, updated_at
        FROM releases
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(i64::from(query.size))
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;
    let items = rows
        .into_iter()
        .map(Release::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Page {
        items,
        page: query.page,
        size: query.size,
        total: count.total,
    })
}
