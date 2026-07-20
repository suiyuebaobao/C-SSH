//! 在调用方事务内插入一条只处于草稿状态的站点媒体记录。

use cloud_domain::AppResult;
use sqlx::PgConnection;

use crate::{
    SiteMedia,
    model::{NewSiteMedia, SiteMediaRow},
    repository::map_write_error,
};

pub(crate) async fn execute(
    connection: &mut PgConnection,
    input: &NewSiteMedia,
) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        INSERT INTO site_media (
            id, slot, state, storage_key, content_type, byte_size, sha256,
            width, height, alt_zh, alt_en, created_by
        )
        VALUES ($1, 'home_qr', 'draft', $2, 'image/png', $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, slot, state, storage_key, content_type, byte_size, sha256,
                  width, height, alt_zh, alt_en, created_by, published_at,
                  revoked_at, created_at, updated_at
        "#,
    )
    .bind(input.id)
    .bind(&input.storage_key)
    .bind(input.byte_size)
    .bind(&input.sha256)
    .bind(input.width)
    .bind(input.height)
    .bind(&input.alt_zh)
    .bind(&input.alt_en)
    .bind(input.created_by)
    .fetch_one(connection)
    .await
    .map_err(|error| map_write_error(error, "站点媒体草稿与现有数据冲突"))?;
    SiteMedia::try_from(row)
}
