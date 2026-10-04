//! 拥有教学发布、编辑、删除与清理状态机；取消请求不打断已开始的提交。

use cloud_domain::{AdminActor, AppError, AppResult};
use cloud_store::PgPool;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    model::{Metadata, Record},
    multipart::Upload,
    repository, storage, validation,
};

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
    pub(crate) media_base: Arc<PathBuf>,
    pub(crate) uploads: Arc<Semaphore>,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, media_base: impl Into<PathBuf>) -> Self {
        Self {
            pool,
            media_base: Arc::new(media_base.into()),
            uploads: Arc::new(Semaphore::new(2)),
        }
    }

    pub(crate) async fn create(
        &self,
        actor: AdminActor,
        root: PathBuf,
        upload: Upload,
    ) -> AppResult<Record> {
        let service = self.clone();
        let request_id = request_id();
        // 临时文件由自身所有权清理；进入提交后独立任务负责最终结算，不依赖浏览器连接存活。
        let row = tokio::spawn(async move {
            let id = Uuid::now_v7();
            let mut tx = service.pool.begin().await.map_err(repository::error)?;
            let row = repository::create(&mut tx, actor, id, &upload).await?;
            repository::audit(&mut tx, actor, "tutorial_video.created", &row, &request_id).await?;
            storage::persist(upload.video, &root, id, false)?;
            if let Err(error) = storage::persist(upload.poster, &root, id, true) {
                storage::remove(&root, id, false, row.video_bytes as u64, &row.video_sha256)
                    .await?;
                return Err(error);
            }
            storage::sync_directory(&root).await?;
            // commit错误可能已经送达数据库；此时保留文件，列表刷新负责确认，不删除可能已提交的对象。
            tx.commit().await.map_err(repository::error)?;
            Ok::<_, AppError>(row)
        })
        .await
        .map_err(task_error)??;
        cloud_domain::mark_semantic_audit_recorded();
        Ok(row)
    }

    pub(crate) async fn update(
        &self,
        actor: AdminActor,
        id: Uuid,
        expected: i64,
        metadata: Metadata,
    ) -> AppResult<Record> {
        validation::id(id)?;
        validation::revision(expected)?;
        let metadata = validation::metadata(metadata)?;
        let service = self.clone();
        let request_id = request_id();
        let row = tokio::spawn(async move {
            let mut tx = service.pool.begin().await.map_err(repository::error)?;
            let row = repository::lock(&mut tx, id).await?;
            check_revision(&row, expected)?;
            if row.state == "deleted" {
                return Err(AppError::Conflict("已删除的视频不能编辑".into()));
            }
            let row = repository::update(&mut tx, id, &metadata).await?;
            repository::audit(&mut tx, actor, "tutorial_video.updated", &row, &request_id).await?;
            tx.commit().await.map_err(repository::error)?;
            Ok::<_, AppError>(row)
        })
        .await
        .map_err(task_error)??;
        cloud_domain::mark_semantic_audit_recorded();
        Ok(row)
    }

    pub(crate) async fn transition(
        &self,
        actor: AdminActor,
        id: Uuid,
        expected: i64,
        target: &'static str,
    ) -> AppResult<Record> {
        validation::id(id)?;
        validation::revision(expected)?;
        let service = self.clone();
        let request_id = request_id();
        let row = tokio::spawn(async move {
            let root = storage::prepare(&service.media_base).await?;
            let mut tx = service.pool.begin().await.map_err(repository::error)?;
            let row = repository::lock(&mut tx, id).await?;
            check_revision(&row, expected)?;
            let allowed = match target {
                "published" => matches!(row.state.as_str(), "draft" | "hidden"),
                "hidden" => row.state == "published",
                _ => false,
            };
            if !allowed {
                return Err(AppError::Conflict(
                    "教学视频状态已变化，请刷新后确认".into(),
                ));
            }
            if target == "published" {
                storage::verify(&root, id, false, row.video_bytes as u64, &row.video_sha256)
                    .await?;
                storage::verify(&root, id, true, row.poster_bytes as u64, &row.poster_sha256)
                    .await?;
            }
            let row = repository::transition(&mut tx, id, target).await?;
            let action = if target == "published" {
                "tutorial_video.published"
            } else {
                "tutorial_video.hidden"
            };
            repository::audit(&mut tx, actor, action, &row, &request_id).await?;
            tx.commit().await.map_err(repository::error)?;
            Ok::<_, AppError>(row)
        })
        .await
        .map_err(task_error)??;
        cloud_domain::mark_semantic_audit_recorded();
        Ok(row)
    }

    pub(crate) async fn delete(&self, actor: AdminActor, id: Uuid, expected: i64) -> AppResult<()> {
        validation::id(id)?;
        validation::revision(expected)?;
        let service = self.clone();
        let request_id = request_id();
        tokio::spawn(async move {
            let root = storage::prepare(&service.media_base).await?;
            let mut tx = service.pool.begin().await.map_err(repository::error)?;
            let row = repository::lock(&mut tx, id).await?;
            check_revision(&row, expected)?;
            if row.state != "deleted" {
                let row = repository::transition(&mut tx, id, "deleted").await?;
                repository::audit(&mut tx, actor, "tutorial_video.deleted", &row, &request_id)
                    .await?;
            }
            tx.commit().await.map_err(repository::error)?;
            // 已确认墓碑后再清文件；中断或失败时后台仍能看到待清理记录，并明确继续同一次删除。
            let mut tx = service.pool.begin().await.map_err(repository::error)?;
            let row = repository::lock(&mut tx, id).await?;
            if row.purged_at.is_none() {
                storage::remove(&root, id, false, row.video_bytes as u64, &row.video_sha256)
                    .await?;
                storage::remove(&root, id, true, row.poster_bytes as u64, &row.poster_sha256)
                    .await?;
                storage::sync_directory(&root).await?;
                repository::purged(&mut tx, id).await?;
            }
            tx.commit().await.map_err(repository::error)?;
            Ok::<_, AppError>(())
        })
        .await
        .map_err(task_error)??;
        cloud_domain::mark_semantic_audit_recorded();
        Ok(())
    }
}

fn check_revision(row: &Record, expected: i64) -> AppResult<()> {
    if row.revision != expected {
        return Err(AppError::Conflict("教学视频已被修改，请刷新后确认".into()));
    }
    Ok(())
}

fn request_id() -> String {
    cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string())
}
fn task_error(_: tokio::task::JoinError) -> AppError {
    AppError::Internal("教学视频操作尚未确认，请刷新列表".into())
}
