//! 记录不含地址、凭据或用户代理的最小下载事件。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) async fn execute(pool: &PgPool, asset_id: Uuid, source_id: Uuid) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO download_events (id, asset_id, source_id, account_id)
        VALUES ($1, $2, $3, NULL)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(asset_id)
    .bind(source_id)
    .execute(pool)
    .await
    .map_err(|error| map_write_error(error, "下载事件关联的来源已失效"))?;
    Ok(())
}
