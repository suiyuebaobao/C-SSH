//! epoch0保护迁移时把旧Host墓碑重建为无业务字段的opaque代次快照。

use cloud_domain::{AppError, AppResult};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{ResourceKind, ResourceRevision, actor::DeviceActor};

use super::super::super::{DbTransaction, storage};

#[derive(FromRow)]
struct Tombstone {
    id: Uuid,
    revision: i64,
}

pub(super) async fn rebaseline_tombstones(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    current_revision: &mut i64,
    results: &mut Vec<(ResourceRevision, i64)>,
) -> AppResult<()> {
    let rows = sqlx::query_as::<_, Tombstone>(
        "SELECT id,revision FROM cloud_hosts
         WHERE account_id=$1 AND is_deleted ORDER BY id FOR UPDATE",
    )
    .bind(actor.account_id())
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    for row in rows {
        *current_revision = current_revision
            .checked_add(1)
            .ok_or_else(|| AppError::Conflict("current_revision cannot advance".to_owned()))?;
        sqlx::query(
            "UPDATE cloud_hosts
             SET address=NULL,port=NULL,name=NULL,platform=NULL,tags=NULL,status=NULL,
                 ciphertext=NULL,source_device_id=$3,revision=$4,
                 metadata_encrypted=TRUE,updated_at=now()
             WHERE account_id=$1 AND id=$2 AND is_deleted",
        )
        .bind(actor.account_id())
        .bind(row.id)
        .bind(actor.device_id())
        .bind(*current_revision)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
        sqlx::query(
            "INSERT INTO cloud_host_versions
                 (account_id,host_id,revision,address,port,name,platform,tags,status,
                  ciphertext,source_device_id,is_deleted,metadata_encrypted)
             VALUES ($1,$2,$3,NULL,NULL,NULL,NULL,NULL,NULL,NULL,$4,TRUE,TRUE)",
        )
        .bind(actor.account_id())
        .bind(row.id)
        .bind(*current_revision)
        .bind(actor.device_id())
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
        results.push((
            ResourceRevision {
                resource_kind: ResourceKind::Host,
                resource_id: row.id,
                cloud_revision: *current_revision,
            },
            row.revision,
        ));
    }
    Ok(())
}
