//! 在同一下载事务内锁定未聚合事件、累加 UTC 日桶并标记原始事件。

use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::model::{DownloadAggregateBucket, PendingDownloadEvent};

use super::{map_read_error, map_write_error};

pub(crate) const SELECT_PENDING_SQL: &str = r#"
    SELECT id, (occurred_at AT TIME ZONE 'UTC')::date AS bucket_date,
           asset_id, source_id, account_id
    FROM download_events
    WHERE aggregated_at IS NULL
    ORDER BY occurred_at, id
    LIMIT $1
    FOR UPDATE SKIP LOCKED
"#;

pub(crate) const UPSERT_BUCKET_SQL: &str = r#"
    INSERT INTO download_event_daily_aggregates (
        bucket_date, asset_id, source_id, audience, event_count
    )
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (bucket_date, asset_id, source_id, audience)
    DO UPDATE SET
        event_count = download_event_daily_aggregates.event_count + EXCLUDED.event_count,
        updated_at = now()
"#;

pub(crate) const MARK_AGGREGATED_SQL: &str = r#"
    UPDATE download_events
    SET aggregated_at = now()
    WHERE id = ANY($1) AND aggregated_at IS NULL
"#;

pub(crate) async fn select_pending(
    transaction: &mut Transaction<'_, Postgres>,
    batch_size: u32,
) -> AppResult<Vec<PendingDownloadEvent>> {
    sqlx::query_as::<_, PendingDownloadEvent>(SELECT_PENDING_SQL)
        .bind(i64::from(batch_size))
        .fetch_all(&mut **transaction)
        .await
        .map_err(map_read_error)
}

pub(crate) async fn upsert_bucket(
    transaction: &mut Transaction<'_, Postgres>,
    bucket: &DownloadAggregateBucket,
    event_count: i64,
) -> AppResult<()> {
    sqlx::query(UPSERT_BUCKET_SQL)
        .bind(bucket.bucket_date)
        .bind(bucket.asset_id)
        .bind(bucket.source_id)
        .bind(bucket.audience.as_str())
        .bind(event_count)
        .execute(&mut **transaction)
        .await
        .map_err(|error| map_write_error(error, "下载事件聚合桶发生冲突"))?;
    Ok(())
}

pub(crate) async fn mark_aggregated(
    transaction: &mut Transaction<'_, Postgres>,
    event_ids: &[Uuid],
) -> AppResult<()> {
    let updated = sqlx::query(MARK_AGGREGATED_SQL)
        .bind(event_ids)
        .execute(&mut **transaction)
        .await
        .map_err(|error| map_write_error(error, "下载事件聚合标记发生冲突"))?
        .rows_affected();
    if updated != event_ids.len() as u64 {
        return Err(AppError::Storage("下载事件聚合批次身份发生变化".into()));
    }
    Ok(())
}
