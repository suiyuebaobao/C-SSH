//! 更新尚未发布的资产身份字段。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ReleaseAsset, UpdateAssetInput, model::AssetRow, repository::map_write_error};

pub(crate) async fn execute(
    connection: &mut PgConnection,
    id: Uuid,
    input: &UpdateAssetInput,
) -> AppResult<ReleaseAsset> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        UPDATE release_assets
        SET platform = COALESCE($2, platform),
            architecture = COALESCE($3, architecture),
            package_kind = COALESCE($4, package_kind),
            file_name = COALESCE($5, file_name),
            byte_size = COALESCE($6, byte_size),
            sha256 = COALESCE($7, sha256)
        WHERE id = $1
        RETURNING id, release_id, platform, architecture, package_kind,
                  file_name, byte_size, sha256, created_at
        "#,
    )
    .bind(id)
    .bind(input.platform.as_deref())
    .bind(input.architecture.as_deref())
    .bind(input.package_kind.as_deref())
    .bind(input.file_name.as_deref())
    .bind(input.byte_size)
    .bind(input.sha256.as_deref())
    .fetch_one(connection)
    .await
    .map_err(|error| map_write_error(error, "资产更新发生冲突"))
}
