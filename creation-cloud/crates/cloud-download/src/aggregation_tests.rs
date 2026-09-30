//! 验证下载事件分桶、批次边界和事务 SQL 不删除或重复选取原始事件。

use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use crate::{
    Service,
    model::{DownloadAudience, PendingDownloadEvent},
    repository::aggregation::{MARK_AGGREGATED_SQL, SELECT_PENDING_SQL, UPSERT_BUCKET_SQL},
    use_case::aggregation::group_events,
};

#[test]
fn groups_utc_day_asset_source_and_audience_without_account_identity() {
    let day = NaiveDate::from_ymd_opt(2026, 7, 21).expect("固定 UTC 日期应有效");
    let asset_id = Uuid::now_v7();
    let source_id = Uuid::now_v7();
    let account_id = Uuid::now_v7();
    let events = vec![
        event(day, asset_id, source_id, None),
        event(day, asset_id, source_id, None),
        event(day, asset_id, source_id, Some(account_id)),
    ];

    let buckets = group_events(&events);
    assert_eq!(buckets.len(), 2);
    assert_eq!(
        buckets
            .iter()
            .find(|(bucket, _)| bucket.audience == DownloadAudience::Anonymous)
            .map(|(_, count)| *count),
        Some(2)
    );
    assert_eq!(
        buckets
            .iter()
            .find(|(bucket, _)| bucket.audience == DownloadAudience::Authenticated)
            .map(|(_, count)| *count),
        Some(1)
    );
}

#[tokio::test]
async fn rejects_invalid_batch_before_touching_postgresql() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://example:example@example.com/example")
        .expect("固定惰性连接应可创建");
    let service = Service::new(pool, "downloads");
    assert!(service.aggregate_download_events(0).await.is_err());
    assert!(service.aggregate_download_events(1_001).await.is_err());
}

#[test]
fn exposes_same_connection_aggregation_entrypoint() {
    let _ = Service::aggregate_download_events_with_connection;
}

#[test]
fn aggregation_sql_is_locked_idempotent_and_preserves_raw_events() {
    assert!(SELECT_PENDING_SQL.contains("aggregated_at IS NULL"));
    assert!(SELECT_PENDING_SQL.contains("FOR UPDATE SKIP LOCKED"));
    assert!(UPSERT_BUCKET_SQL.contains("ON CONFLICT"));
    assert!(UPSERT_BUCKET_SQL.contains("event_count + EXCLUDED.event_count"));
    assert!(MARK_AGGREGATED_SQL.contains("SET aggregated_at = now()"));
    assert!(MARK_AGGREGATED_SQL.contains("aggregated_at IS NULL"));
    let all_sql = format!("{SELECT_PENDING_SQL}{UPSERT_BUCKET_SQL}{MARK_AGGREGATED_SQL}");
    assert!(
        !all_sql
            .to_ascii_uppercase()
            .contains("DELETE FROM DOWNLOAD_EVENTS")
    );
}

#[test]
fn migration_adds_partial_queue_and_account_free_daily_buckets() {
    let migration = include_str!("../../../migrations/0021_download_aggregation.sql");
    assert!(migration.contains("ADD COLUMN aggregated_at TIMESTAMPTZ"));
    assert!(migration.contains("WHERE aggregated_at IS NULL"));
    assert!(migration.contains("download_event_daily_aggregates"));
    assert!(migration.contains("'anonymous', 'authenticated'"));
    assert!(!migration.contains("account_id"));
    assert!(
        !migration
            .to_ascii_uppercase()
            .contains("DELETE FROM DOWNLOAD_EVENTS")
    );
}

fn event(
    bucket_date: NaiveDate,
    asset_id: Uuid,
    source_id: Uuid,
    account_id: Option<Uuid>,
) -> PendingDownloadEvent {
    PendingDownloadEvent {
        id: Uuid::now_v7(),
        bucket_date,
        asset_id,
        source_id,
        account_id,
    }
}
