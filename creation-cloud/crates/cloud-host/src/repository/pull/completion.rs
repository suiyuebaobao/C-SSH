//! 封装下载完成通知与 rekey 只读结算，保持 pull 主事务职责有界。

use cloud_domain::AppResult;
use cloud_notification::{AccountNotificationEvent, record_account_event};

use crate::{ResourceRevision, actor::DeviceActor};

use super::super::DbTransaction;

pub(super) async fn record_download_completed(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
) -> AppResult<()> {
    record_account_event(
        tx,
        actor.account_id(),
        AccountNotificationEvent::SyncDownloadCompleted {
            device_id: actor.device_id(),
        },
    )
    .await?;
    Ok(())
}

pub(in crate::repository) async fn record_rekey_snapshot(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    revisions: &[ResourceRevision],
    snapshot_revision: i64,
) -> AppResult<()> {
    // Rekey 核验本身不建立 pull delivery；后续显式下载才提供 ACK 证据。
    let _ = (tx, actor, revisions, snapshot_revision);
    Ok(())
}
