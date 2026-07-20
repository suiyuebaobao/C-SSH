//! 只读取当前唯一已发布的首页二维码元数据。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;

use crate::{SiteMedia, model::SiteMediaRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        SELECT id, slot, state, storage_key, content_type, byte_size, sha256,
               width, height, alt_zh, alt_en, created_by, published_at,
               revoked_at, created_at, updated_at
        FROM site_media
        WHERE slot = 'home_qr' AND state = 'published'
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("当前没有已发布首页二维码".into()))?;
    SiteMedia::try_from(row)
}
