//! 只把受控发布编排提供的安装身份摘要写入尚未取证的正式资产。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ReleaseAsset, model::AssetRow, repository::map_write_error};

pub(crate) async fn execute(
    connection: &mut PgConnection,
    id: Uuid,
    installed_sha256: &str,
) -> AppResult<ReleaseAsset> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        UPDATE release_assets
        SET installed_sha256 = $2
        WHERE id = $1 AND installed_sha256 IS NULL
        RETURNING id, release_id, platform, architecture, package_kind,
                  file_name, byte_size, sha256, installed_sha256, created_at
        "#,
    )
    .bind(id)
    .bind(installed_sha256)
    .fetch_optional(connection)
    .await
    .map_err(|error| map_write_error(error, "保存安装身份摘要发生冲突"))?
    .ok_or_else(|| AppError::Conflict("安装身份摘要已经存在".into()))
}
