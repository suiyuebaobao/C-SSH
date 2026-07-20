//! 将 PostgreSQL commit 与文件 guard 决策移交给脱离 HTTP 请求生命周期的 owner。
//! 请求取消只会停止等待；owner 仍完成提交核对，身份不确定时始终保留文件。

use std::future::Future;

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};

use crate::{
    SiteMedia,
    repository::finalization::{self as identity, DatabaseIdentity},
    storage::{DeletedFile, StagedFile},
};

pub(crate) async fn create(
    pool: PgPool,
    transaction: Transaction<'static, Postgres>,
    mut staged: StagedFile,
    media: SiteMedia,
) -> AppResult<SiteMedia> {
    // 移交与 spawn 之间没有取消点；owner 异常退出时也保留文件供人工对账。
    staged.preserve_on_drop();
    run_owned(async move {
        let commit_failed = transaction.commit().await.is_err();
        match identity::inspect(
            &pool,
            media.id,
            &media.storage_key,
            media.byte_size,
            &media.sha256,
        )
        .await?
        {
            DatabaseIdentity::Exact => Ok(media),
            DatabaseIdentity::Absent => {
                staged.cleanup().await;
                Err(AppError::Storage(if commit_failed {
                    "提交站点媒体创建事务失败".into()
                } else {
                    "站点媒体创建提交后未找到数据库记录".into()
                }))
            }
            DatabaseIdentity::Conflict => Err(AppError::Storage(
                "站点媒体创建提交后的数据库身份冲突，文件已保留待对账".into(),
            )),
        }
    })
    .await
}

pub(crate) async fn delete(
    pool: PgPool,
    transaction: Transaction<'static, Postgres>,
    mut deleted: DeletedFile,
    media: SiteMedia,
) -> AppResult<()> {
    // 删除提交不确定时，Drop 既不能恢复也不能清除；只能由 owner 按数据库身份决定。
    deleted.preserve_on_drop();
    run_owned(async move {
        let commit_failed = transaction.commit().await.is_err();
        match identity::inspect(
            &pool,
            media.id,
            &media.storage_key,
            media.byte_size,
            &media.sha256,
        )
        .await?
        {
            DatabaseIdentity::Absent => deleted.finish().await,
            DatabaseIdentity::Exact => {
                deleted.restore().await?;
                Err(AppError::Storage(if commit_failed {
                    "提交站点媒体删除事务失败，文件已恢复".into()
                } else {
                    "站点媒体删除提交后记录仍存在，文件已恢复".into()
                }))
            }
            DatabaseIdentity::Conflict => Err(AppError::Storage(
                "站点媒体删除提交后的数据库身份冲突，隔离文件已保留待对账".into(),
            )),
        }
    })
    .await
}

async fn run_owned<T>(future: impl Future<Output = AppResult<T>> + Send + 'static) -> AppResult<T>
where
    T: Send + 'static,
{
    tokio::spawn(future)
        .await
        .map_err(|_| AppError::Storage("站点媒体事务收尾 owner 异常终止".into()))?
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot;

    use super::*;

    #[tokio::test]
    async fn request_cancellation_does_not_cancel_owned_finalization() {
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let (completed_tx, completed_rx) = oneshot::channel();
        let request = tokio::spawn(run_owned(async move {
            let _ = started_tx.send(());
            let _ = release_rx.await;
            let _ = completed_tx.send(());
            Ok(())
        }));

        started_rx.await.expect("收尾 owner 应已启动");
        request.abort();
        let _ = request.await;
        let _ = release_tx.send(());
        completed_rx.await.expect("请求取消后 owner 仍应完成");
    }
}
