//! 读取公开下载请求对应的已发布且已启用来源。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{model::DownloadTarget, repository::map_read_error};

pub(crate) async fn execute(
    pool: &PgPool,
    asset_id: Uuid,
    source_id: Uuid,
) -> AppResult<DownloadTarget> {
    sqlx::query_as::<_, DownloadTarget>(
        r#"
        SELECT assets.id AS asset_id, sources.id AS source_id,
               assets.file_name, assets.byte_size, assets.sha256,
               sources.source_kind, sources.local_path, sources.external_url
        FROM release_sources AS sources
        JOIN release_assets AS assets ON assets.id = sources.asset_id
        JOIN releases ON releases.id = assets.release_id
        WHERE assets.id = $1 AND sources.id = $2
          AND releases.status = 'published' AND sources.enabled = TRUE
        "#,
    )
    .bind(asset_id)
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("公开下载来源不存在或已停用".into()))
}
