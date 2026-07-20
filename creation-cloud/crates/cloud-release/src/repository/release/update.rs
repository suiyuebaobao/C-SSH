//! 原子更新草稿元数据或执行受状态机约束的状态迁移。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Release, UpdateReleaseInput, model::ReleaseRow, repository::map_write_error};

pub(crate) async fn execute(
    pool: &PgPool,
    id: Uuid,
    input: &UpdateReleaseInput,
) -> AppResult<Release> {
    let status = input.status.map(crate::ReleaseStatus::as_str);
    let row = sqlx::query_as::<_, ReleaseRow>(
        r#"
        UPDATE releases
        SET title_zh = COALESCE($2, title_zh),
            title_en = COALESCE($3, title_en),
            notes_zh = COALESCE($4, notes_zh),
            notes_en = COALESCE($5, notes_en),
            status = COALESCE($6, status),
            published_at = CASE
                WHEN $6 = 'published' THEN COALESCE(published_at, now())
                ELSE published_at
            END,
            updated_at = now()
        WHERE id = $1
        RETURNING id, version, channel, status, title_zh, title_en,
                  notes_zh, notes_en, published_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(input.title_zh.as_deref())
    .bind(input.title_en.as_deref())
    .bind(input.notes_zh.as_deref())
    .bind(input.notes_en.as_deref())
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(|error| map_write_error(error, "版本更新发生冲突"))?;

    Release::try_from(row)
}
