//! 按内部标识读取单个下载来源。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ReleaseSource, model::SourceRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, source_id: Uuid) -> AppResult<ReleaseSource> {
    let row = sqlx::query_as::<_, SourceRow>(
        r#"
        SELECT id, asset_id, source_kind, provider_name, local_path,
               external_url, sort_order, enabled, created_at, updated_at
        FROM release_sources
        WHERE id = $1
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("下载来源不存在".into()))?;
    ReleaseSource::try_from(row)
}
