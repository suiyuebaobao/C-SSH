//! 只更新来源排序和启停状态，不原地替换来源身份。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ReleaseSource, UpdateSourceInput, model::SourceRow, repository::map_write_error};

pub(crate) async fn execute(
    connection: &mut PgConnection,
    source_id: Uuid,
    input: &UpdateSourceInput,
) -> AppResult<ReleaseSource> {
    let row = sqlx::query_as::<_, SourceRow>(
        r#"
        UPDATE release_sources
        SET sort_order = COALESCE($2, sort_order),
            enabled = COALESCE($3, enabled),
            updated_at = now()
        WHERE id = $1
        RETURNING id, asset_id, source_kind, provider_name, local_path,
                  external_url, sort_order, enabled, created_at, updated_at
        "#,
    )
    .bind(source_id)
    .bind(input.sort_order)
    .bind(input.enabled)
    .fetch_one(connection)
    .await
    .map_err(|error| map_write_error(error, "下载来源更新发生冲突"))?;
    ReleaseSource::try_from(row)
}
