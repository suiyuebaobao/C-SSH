//! 为可编辑版本登记一个不可与来源混淆的资产身份。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{CreateAssetInput, ReleaseAsset, model::AssetRow, repository::map_write_error};

pub(crate) async fn execute(
    connection: &mut PgConnection,
    input: &CreateAssetInput,
) -> AppResult<ReleaseAsset> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        INSERT INTO release_assets (
            id, release_id, platform, architecture, package_kind,
            file_name, byte_size, sha256
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, release_id, platform, architecture, package_kind,
                  file_name, byte_size, sha256, created_at
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(input.release_id)
    .bind(&input.platform)
    .bind(&input.architecture)
    .bind(&input.package_kind)
    .bind(&input.file_name)
    .bind(input.byte_size)
    .bind(&input.sha256)
    .fetch_one(connection)
    .await
    .map_err(|error| map_write_error(error, "同平台、架构和包类型的资产已经存在"))
}
