//! 按创建时间倒序读取首页二维码历史记录。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{SiteMedia, model::SiteMediaRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, limit: u32) -> AppResult<Vec<SiteMedia>> {
    let rows = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        SELECT id, slot, state, storage_key, content_type, byte_size, sha256,
               width, height, alt_zh, alt_en, created_by, published_at,
               revoked_at, created_at, updated_at
        FROM site_media
        WHERE slot = 'home_qr'
        ORDER BY created_at DESC, id DESC
        LIMIT $1
        "#,
    )
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;
    rows.into_iter().map(SiteMedia::try_from).collect()
}
