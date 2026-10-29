//! 为永久账号删除清理本领域的设备引用；与调用方账号删除共用一笔事务。

use cloud_domain::{AppError, AppResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// 只供账号删除用例调用；不提交事务，不修改其他账号或全局资源。
pub async fn prepare_account_deletion(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    // 同步写入持有账号FOR SHARE；独占锁确保清理后不会插入新的设备依赖。
    let account = sqlx::query_scalar::<_, Uuid>("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(account_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(super::storage)?;
    if account.is_none() {
        return Err(AppError::NotFound("账号不存在".to_owned()));
    }
    // 后台维护也可能持同步状态锁；沿既有账号→同步状态锁顺序，且不创建空状态。
    super::lock_existing_sync_state(transaction, account_id).await?;

    // 这些行对设备使用RESTRICT，不能依赖accounts级联触发器的执行次序。
    // 其余账号自有行仍交由原账号DELETE级联；不可变全局资源继续阻挡并回滚。
    for statement in [
        "DELETE FROM cloud_sync_rekey_mutations WHERE account_id=$1",
        "DELETE FROM cloud_sync_reset_mutations WHERE account_id=$1",
        "DELETE FROM cloud_sync_push_mutations WHERE account_id=$1",
        "DELETE FROM cloud_data_protection_mutations WHERE account_id=$1",
        "DELETE FROM cloud_host_metadata_migrations WHERE account_id=$1",
        "DELETE FROM cloud_ai_provider_config_versions WHERE account_id=$1",
        "DELETE FROM cloud_ai_provider_configs WHERE account_id=$1",
        "DELETE FROM cloud_proxy_profile_versions WHERE account_id=$1",
        "DELETE FROM cloud_proxy_profiles WHERE account_id=$1",
        "DELETE FROM cloud_host_versions WHERE account_id=$1",
        "DELETE FROM cloud_hosts WHERE account_id=$1",
    ] {
        sqlx::query(statement)
            .bind(account_id)
            .execute(&mut **transaction)
            .await
            .map_err(super::storage)?;
    }
    Ok(())
}
