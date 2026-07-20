//! 将当前已发布站点媒体单向迁移为撤销状态。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{SiteMedia, model::SiteMediaRow, repository::map_write_error};

pub(crate) async fn execute(pool: &PgPool, id: Uuid) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        UPDATE site_media
        SET state = 'revoked', revoked_at = now(), updated_at = now()
        WHERE id = $1 AND state = 'published'
        RETURNING id, slot, state, storage_key, content_type, byte_size, sha256,
                  width, height, alt_zh, alt_en, created_by, published_at,
                  revoked_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|error| map_write_error(error, "站点媒体撤销状态冲突"))?
    .ok_or_else(|| AppError::Conflict("只有已发布站点媒体可以撤销".into()))?;
    SiteMedia::try_from(row)
}
