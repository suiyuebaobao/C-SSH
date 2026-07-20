//! 按稳定平台顺序读取某个版本的全部资产。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{ReleaseAsset, model::AssetRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, release_id: Uuid) -> AppResult<Vec<ReleaseAsset>> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT id, release_id, platform, architecture, package_kind,
               file_name, byte_size, sha256, created_at
        FROM release_assets
        WHERE release_id = $1
        ORDER BY platform, architecture, package_kind, id
        "#,
    )
    .bind(release_id)
    .fetch_all(pool)
    .await
    .map_err(map_read_error)
}
