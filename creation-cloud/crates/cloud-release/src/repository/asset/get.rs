//! 按内部标识读取单个资产身份。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ReleaseAsset, model::AssetRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, id: Uuid) -> AppResult<ReleaseAsset> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT id, release_id, platform, architecture, package_kind,
               file_name, byte_size, sha256, created_at
        FROM release_assets
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("资产不存在".into()))
}
