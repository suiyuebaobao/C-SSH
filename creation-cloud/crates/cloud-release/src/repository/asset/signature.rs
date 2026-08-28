//! 发布前同时拒绝 Windows 缺签名与任何非 Windows 多签名资产。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

pub(crate) async fn has_invalid_metadata(pool: &PgPool, release_id: Uuid) -> AppResult<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM release_assets WHERE release_id = $1 \
         AND ((platform = 'windows' AND package_kind IN ('exe', 'msi', 'zip') \
               AND updater_signature IS NULL) \
           OR (NOT (platform = 'windows' AND package_kind IN ('exe', 'msi', 'zip')) \
               AND updater_signature IS NOT NULL)))",
    )
    .bind(release_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Storage("检查发布资产签名失败".into()))
}
