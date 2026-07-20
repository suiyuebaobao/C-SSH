//! 在调用方事务内删除已经锁定且确认仍为草稿的站点媒体记录。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM site_media WHERE id = $1 AND state = 'draft'")
        .bind(id)
        .execute(connection)
        .await
        .map_err(|error| map_write_error(error, "只有草稿站点媒体可以删除"))?;
    if result.rows_affected() != 1 {
        return Err(AppError::Conflict("只有草稿站点媒体可以删除".into()));
    }
    Ok(())
}
