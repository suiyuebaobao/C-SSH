//! 删除用例层已确认属于未发布版本的来源。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) async fn execute(connection: &mut PgConnection, source_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM release_sources WHERE id = $1")
        .bind(source_id)
        .execute(connection)
        .await
        .map_err(|error| map_write_error(error, "已发布或已有下载事件的来源必须保留"))?;
    Ok(())
}
