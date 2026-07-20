//! 删除已经由用例层确认仍为草稿的版本。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM releases WHERE id = $1")
        .bind(id)
        .execute(connection)
        .await
        .map_err(|error| map_write_error(error, "版本仍被其它数据引用"))?;
    Ok(())
}
