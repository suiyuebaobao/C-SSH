//! 在单事务内完成未处理下载事件的 UTC 日桶聚合和原始事件标记。

use std::collections::BTreeMap;

use cloud_domain::{AppError, AppResult};
use sqlx::{Acquire, PgConnection};

use crate::{
    DownloadAggregationReport, Service,
    model::{DownloadAggregateBucket, DownloadAudience, PendingDownloadEvent},
    repository,
};

const MAX_BATCH_SIZE: u32 = 1_000;

impl Service {
    pub async fn aggregate_download_events(
        &self,
        batch_size: u32,
    ) -> AppResult<DownloadAggregationReport> {
        validate_batch_size(batch_size)?;
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(repository::map_transaction_error)?;
        self.aggregate_download_events_with_connection(&mut connection, batch_size)
            .await
    }

    pub async fn aggregate_download_events_with_connection(
        &self,
        connection: &mut PgConnection,
        batch_size: u32,
    ) -> AppResult<DownloadAggregationReport> {
        validate_batch_size(batch_size)?;
        let mut transaction = connection
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let events = repository::aggregation::select_pending(&mut transaction, batch_size).await?;
        let buckets = group_events(&events);
        for (bucket, event_count) in &buckets {
            repository::aggregation::upsert_bucket(&mut transaction, bucket, *event_count).await?;
        }
        let event_ids = events.iter().map(|event| event.id).collect::<Vec<_>>();
        if !event_ids.is_empty() {
            repository::aggregation::mark_aggregated(&mut transaction, &event_ids).await?;
        }
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(DownloadAggregationReport {
            processed_events: events.len() as u64,
            updated_buckets: buckets.len() as u64,
        })
    }
}

fn validate_batch_size(batch_size: u32) -> AppResult<()> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(AppError::Validation(
            "下载事件聚合批次必须在 1 到 1000 之间".into(),
        ));
    }
    Ok(())
}

pub(crate) fn group_events(
    events: &[PendingDownloadEvent],
) -> BTreeMap<DownloadAggregateBucket, i64> {
    let mut buckets = BTreeMap::new();
    for event in events {
        let bucket = DownloadAggregateBucket {
            bucket_date: event.bucket_date,
            asset_id: event.asset_id,
            source_id: event.source_id,
            audience: DownloadAudience::from_account(event.account_id),
        };
        *buckets.entry(bucket).or_insert(0) += 1;
    }
    buckets
}
