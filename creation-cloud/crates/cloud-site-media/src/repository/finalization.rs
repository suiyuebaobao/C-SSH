//! 核对事务提交后的站点媒体数据库身份，供文件收尾 owner 做唯一决策。
//! 查询失败或身份冲突时不猜测提交结果，也不授权删除任何文件。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DatabaseIdentity {
    Exact,
    Absent,
    Conflict,
}

pub(crate) async fn inspect(
    pool: &PgPool,
    media_id: Uuid,
    storage_key: &str,
    byte_size: i64,
    sha256: &str,
) -> AppResult<DatabaseIdentity> {
    let (exact, occupied) = sqlx::query_as::<_, (bool, bool)>(
        r#"
        SELECT
            EXISTS(
                SELECT 1 FROM site_media
                WHERE id = $1 AND storage_key = $2
                  AND byte_size = $3 AND sha256 = $4
            ),
            EXISTS(
                SELECT 1 FROM site_media
                WHERE id = $1 OR storage_key = $2
            )
        "#,
    )
    .bind(media_id)
    .bind(storage_key)
    .bind(byte_size)
    .bind(sha256)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Storage("无法核对站点媒体事务提交结果".into()))?;

    Ok(if exact {
        DatabaseIdentity::Exact
    } else if occupied {
        DatabaseIdentity::Conflict
    } else {
        DatabaseIdentity::Absent
    })
}
