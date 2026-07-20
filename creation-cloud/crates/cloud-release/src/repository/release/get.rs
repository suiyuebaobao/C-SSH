//! 按内部标识读取单个版本。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Release, model::ReleaseRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, id: Uuid) -> AppResult<Release> {
    let row = sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT id, version, channel, status, title_zh, title_en,
               notes_zh, notes_en, published_at, created_at, updated_at
        FROM releases
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("版本不存在".into()))?;

    Release::try_from(row)
}
