//! 已认证账号锁内定位同一已识别设备的活动安装实例；旧记录只命中自身。
use cloud_domain::AppResult;
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

pub(crate) const ACTIVE_MEMBERS_SQL: &str = "SELECT d.id FROM devices d JOIN device_identity_groups g \
         ON d.id=ANY(g.registration_ids) AND d.account_id=g.account_id \
         JOIN devices chosen ON chosen.id=$2 AND chosen.account_id=$1 AND chosen.revoked_at IS NULL \
         WHERE g.account_id=$1 AND chosen.id=ANY(g.registration_ids) AND d.revoked_at IS NULL \
         ORDER BY d.id FOR UPDATE OF d";

pub(crate) async fn active_ids(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    selected: Uuid,
) -> AppResult<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(ACTIVE_MEMBERS_SQL)
        .bind(account_id)
        .bind(selected)
        .fetch_all(&mut **tx)
        .await
        .map_err(super::error::storage)
}
