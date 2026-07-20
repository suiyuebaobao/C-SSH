//! 读取已发布版本、资产和已启用来源的扁平公开投影。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{model::PublicCatalogRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<Vec<PublicCatalogRow>> {
    sqlx::query_as::<_, PublicCatalogRow>(
        r#"
        SELECT releases.id AS release_id, releases.version, releases.channel,
               releases.title_zh, releases.title_en, releases.notes_zh,
               releases.notes_en, releases.published_at,
               assets.id AS asset_id, assets.platform, assets.architecture,
               assets.package_kind, assets.file_name, assets.byte_size, assets.sha256,
               sources.id AS source_id, sources.source_kind, sources.provider_name,
               sources.sort_order
        FROM releases
        JOIN release_assets AS assets ON assets.release_id = releases.id
        JOIN release_sources AS sources ON sources.asset_id = assets.id
        WHERE releases.status = 'published' AND sources.enabled = TRUE
        ORDER BY releases.published_at DESC, releases.id DESC,
                 assets.platform, assets.architecture, assets.package_kind,
                 sources.sort_order, sources.created_at, sources.id
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_read_error)
}
