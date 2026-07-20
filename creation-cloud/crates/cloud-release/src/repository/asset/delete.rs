//! 删除已经由用例层确认仍可编辑的资产。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM release_assets WHERE id = $1")
        .bind(id)
        .execute(connection)
        .await
        .map_err(|error| map_write_error(error, "资产仍被下载来源或事件引用"))?;
    Ok(())
}
