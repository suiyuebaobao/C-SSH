//! 只读取 published 站点媒体，并原子保存不含存储键的最新巡检观察。

use cloud_domain::{AppError, AppResult};
use sqlx::{Acquire, PgConnection};

use crate::inspection_model::{PublishedMediaCandidate, SiteMediaInspectionObservation};

use super::{map_read_error, map_write_error};

pub(crate) const PUBLISHED_MEDIA_SQL: &str = r#"
    SELECT id AS media_id,
           storage_key,
           byte_size AS expected_byte_size,
           sha256 AS expected_sha256
    FROM site_media
    WHERE state = 'published'
    ORDER BY id
"#;

pub(crate) const UPSERT_INSPECTION_SQL: &str = r#"
    INSERT INTO site_media_inspections (
        media_id, status, inspected_at, observed_byte_size, observed_sha256
    )
    VALUES ($1, $2, now(), $3, $4)
    ON CONFLICT (media_id)
    DO UPDATE SET
        status = EXCLUDED.status,
        inspected_at = EXCLUDED.inspected_at,
        observed_byte_size = EXCLUDED.observed_byte_size,
        observed_sha256 = EXCLUDED.observed_sha256
"#;

pub(crate) async fn list_published(
    connection: &mut PgConnection,
) -> AppResult<Vec<PublishedMediaCandidate>> {
    sqlx::query_as::<_, PublishedMediaCandidate>(PUBLISHED_MEDIA_SQL)
        .fetch_all(&mut *connection)
        .await
        .map_err(map_read_error)
}

pub(crate) async fn save_observations(
    connection: &mut PgConnection,
    observations: &[SiteMediaInspectionObservation],
) -> AppResult<()> {
    let mut transaction = connection
        .begin()
        .await
        .map_err(|_| AppError::Storage("无法开始站点媒体巡检结果事务".into()))?;
    for observation in observations {
        sqlx::query(UPSERT_INSPECTION_SQL)
            .bind(observation.media_id)
            .bind(observation.status.as_str())
            .bind(observation.observed_byte_size)
            .bind(observation.observed_sha256.as_deref())
            .execute(&mut *transaction)
            .await
            .map_err(|error| map_write_error(error, "站点媒体巡检结果发生冲突"))?;
    }
    transaction
        .commit()
        .await
        .map_err(|_| AppError::Storage("无法提交站点媒体巡检结果事务".into()))
}
