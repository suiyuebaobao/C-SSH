//! 按排序值和创建时间读取资产的全部来源。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ReleaseSource, model::SourceRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, asset_id: Uuid) -> AppResult<Vec<ReleaseSource>> {
    let rows = sqlx::query_as::<_, SourceRow>(
        r#"
        SELECT id, asset_id, source_kind, provider_name, local_path,
               external_url, sort_order, enabled, created_at, updated_at
        FROM release_sources
        WHERE asset_id = $1
        ORDER BY sort_order, created_at, id
        "#,
    )
    .bind(asset_id)
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;
    rows.into_iter()
        .map(ReleaseSource::try_from)
        .collect::<AppResult<Vec<_>>>()
}
