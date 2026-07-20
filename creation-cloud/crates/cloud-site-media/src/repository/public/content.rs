//! 按标识读取已发布首页二维码的受控文件身份。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{SiteMedia, model::SiteMediaRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, id: Uuid) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        SELECT id, slot, state, storage_key, content_type, byte_size, sha256,
               width, height, alt_zh, alt_en, created_by, published_at,
               revoked_at, created_at, updated_at
        FROM site_media
        WHERE id = $1 AND slot = 'home_qr' AND state = 'published'
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("已发布首页二维码不存在".into()))?;
    SiteMedia::try_from(row)
}
