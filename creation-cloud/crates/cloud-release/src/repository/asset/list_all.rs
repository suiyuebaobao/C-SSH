//! 跨版本按创建时间倒序分页读取全部资产。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use sqlx::FromRow;

use crate::{ReleaseAsset, model::AssetRow, repository::map_read_error};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

pub(crate) async fn execute(pool: &PgPool, query: PageQuery) -> AppResult<Page<ReleaseAsset>> {
    let query = query.normalized();
    let count =
        sqlx::query_as::<_, CountRow>("SELECT count(*)::BIGINT AS total FROM release_assets")
            .fetch_one(pool)
            .await
            .map_err(map_read_error)?;
    let items = sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT id, release_id, platform, architecture, package_kind,
               file_name, byte_size, sha256, created_at
        FROM release_assets
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(i64::from(query.size))
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;

    Ok(Page {
        items,
        page: query.page,
        size: query.size,
        total: count.total,
    })
}
