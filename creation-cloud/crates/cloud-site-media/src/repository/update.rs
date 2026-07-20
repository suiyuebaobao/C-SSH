//! 更新仍为草稿的站点媒体双语替代文本。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{SiteMedia, UpdateSiteMediaInput, model::SiteMediaRow, repository::map_write_error};

pub(crate) async fn execute(
    pool: &PgPool,
    id: Uuid,
    input: &UpdateSiteMediaInput,
) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        UPDATE site_media
        SET alt_zh = COALESCE($2, alt_zh),
            alt_en = COALESCE($3, alt_en),
            updated_at = now()
        WHERE id = $1 AND state = 'draft'
        RETURNING id, slot, state, storage_key, content_type, byte_size, sha256,
                  width, height, alt_zh, alt_en, created_by, published_at,
                  revoked_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(input.alt_zh.as_deref())
    .bind(input.alt_en.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(|error| map_write_error(error, "只有草稿站点媒体可以更新"))?
    .ok_or_else(|| AppError::Conflict("只有草稿站点媒体可以更新".into()))?;
    SiteMedia::try_from(row)
}
