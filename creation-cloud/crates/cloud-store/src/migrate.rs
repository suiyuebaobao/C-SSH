//! 在服务启动阶段执行版本化迁移，失败时拒绝继续运行。

use cloud_domain::{AppError, AppResult};
use sqlx::PgPool;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

pub async fn migrate(pool: &PgPool) -> AppResult<()> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|error| AppError::Storage(format!("数据库迁移失败: {error}")))
}
