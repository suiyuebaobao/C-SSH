//! 只按当前账号读取其受保护入口产生的下载事件与公开资产元数据。

use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{DownloadHistoryItem, repository::map_read_error};

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<DownloadHistoryItem>, i64)> {
    let page = page.normalized();
    let total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM download_events WHERE account_id = $1")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .map_err(map_read_error)?;
    let items = sqlx::query_as::<_, DownloadHistoryItem>(
        r#"
        SELECT events.id, events.asset_id, events.source_id, releases.version,
               assets.platform, assets.architecture, assets.package_kind,
               assets.file_name, sources.provider_name, sources.source_kind,
               events.occurred_at
        FROM download_events AS events
        JOIN release_assets AS assets ON assets.id = events.asset_id
        JOIN releases ON releases.id = assets.release_id
        JOIN release_sources AS sources ON sources.id = events.source_id
        WHERE events.account_id = $1
        ORDER BY events.occurred_at DESC, events.id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;
    Ok((items, total))
}
