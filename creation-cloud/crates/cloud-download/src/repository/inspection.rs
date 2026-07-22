//! 只读取已发布本站来源，并原子保存不含路径的最新巡检观察。

use cloud_domain::AppResult;
use sqlx::{Acquire, PgConnection};

use crate::model::{AssetInspectionObservation, PublishedLocalAsset};

use super::{map_read_error, map_transaction_error, map_write_error};

pub(crate) const PUBLISHED_LOCAL_ASSETS_SQL: &str = r#"
    SELECT source.id AS source_id,
           source.local_path,
           asset.byte_size AS expected_byte_size,
           asset.sha256 AS expected_sha256
    FROM release_sources AS source
    JOIN release_assets AS asset ON asset.id = source.asset_id
    JOIN releases AS release ON release.id = asset.release_id
    WHERE release.status = 'published'
      AND source.enabled
      AND source.source_kind = 'local'
    ORDER BY source.id
"#;

pub(crate) const UPSERT_INSPECTION_SQL: &str = r#"
    INSERT INTO release_source_inspections (
        source_id, status, inspected_at, observed_byte_size, observed_sha256
    )
    VALUES ($1, $2, now(), $3, $4)
    ON CONFLICT (source_id)
    DO UPDATE SET
        status = EXCLUDED.status,
        inspected_at = EXCLUDED.inspected_at,
        observed_byte_size = EXCLUDED.observed_byte_size,
        observed_sha256 = EXCLUDED.observed_sha256
"#;

pub(crate) async fn list_published_local(
    connection: &mut PgConnection,
) -> AppResult<Vec<PublishedLocalAsset>> {
    sqlx::query_as::<_, PublishedLocalAsset>(PUBLISHED_LOCAL_ASSETS_SQL)
        .fetch_all(&mut *connection)
        .await
        .map_err(map_read_error)
}

pub(crate) async fn save_observations(
    connection: &mut PgConnection,
    observations: &[AssetInspectionObservation],
) -> AppResult<()> {
    let mut transaction = connection.begin().await.map_err(map_transaction_error)?;
    for observation in observations {
        sqlx::query(UPSERT_INSPECTION_SQL)
            .bind(observation.source_id)
            .bind(observation.status.as_str())
            .bind(observation.observed_byte_size)
            .bind(observation.observed_sha256.as_deref())
            .execute(&mut *transaction)
            .await
            .map_err(|error| map_write_error(error, "发布来源巡检结果发生冲突"))?;
    }
    transaction.commit().await.map_err(map_transaction_error)
}
