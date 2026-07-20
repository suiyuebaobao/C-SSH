//! 读取来源校验所需的最小资产与版本状态投影。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{model::AssetRecord, repository::map_read_error};

pub(crate) async fn get(pool: &PgPool, asset_id: Uuid) -> AppResult<AssetRecord> {
    sqlx::query_as::<_, AssetRecord>(
        r#"
        SELECT assets.release_id, assets.byte_size, assets.sha256,
               releases.status AS release_status
        FROM release_assets AS assets
        JOIN releases ON releases.id = assets.release_id
        WHERE assets.id = $1
        "#,
    )
    .bind(asset_id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("资产不存在".into()))
}
