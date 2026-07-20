//! 在同一事务内串行化槽位发布、撤销旧版本并发布目标草稿。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{SiteMedia, SiteMediaSlot, model::SiteMediaRow, repository::map_write_error};

pub(crate) async fn lock_slot(connection: &mut PgConnection, slot: SiteMediaSlot) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("site_media:{}", slot.as_str()))
        .execute(connection)
        .await
        .map_err(crate::repository::map_read_error)?;
    Ok(())
}

pub(crate) async fn revoke_current(
    connection: &mut PgConnection,
    slot: SiteMediaSlot,
    except_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE site_media
        SET state = 'revoked', revoked_at = now(), updated_at = now()
        WHERE slot = $1 AND state = 'published' AND id <> $2
        "#,
    )
    .bind(slot.as_str())
    .bind(except_id)
    .execute(connection)
    .await
    .map_err(|error| map_write_error(error, "撤下旧站点媒体失败"))?;
    Ok(())
}

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<SiteMedia> {
    let row = sqlx::query_as::<_, SiteMediaRow>(
        r#"
        UPDATE site_media
        SET state = 'published', published_at = now(), revoked_at = NULL, updated_at = now()
        WHERE id = $1 AND state = 'draft'
        RETURNING id, slot, state, storage_key, content_type, byte_size, sha256,
                  width, height, alt_zh, alt_en, created_by, published_at,
                  revoked_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(connection)
    .await
    .map_err(|error| map_write_error(error, "站点媒体发布状态冲突"))?
    .ok_or_else(|| AppError::Conflict("只有草稿站点媒体可以发布".into()))?;
    SiteMedia::try_from(row)
}
