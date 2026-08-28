//! 为业务域提供只能在既有 PostgreSQL 事务内调用的闭集账号事件通知入口。

use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::{NotificationKind, NotificationPriority};

/// 业务域已成功提交的账号事件；调用方不能自定义 kind、priority 或 code。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountNotificationEvent {
    PasswordChanged,
    DeviceRevoked { device_id: Uuid },
    SessionRevoked { session_id: Uuid },
    SyncUploadCompleted { mutation_id: Uuid },
    SyncDownloadCompleted { device_id: Uuid },
    SyncResetCompleted { mutation_id: Uuid },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EventDescriptor {
    kind: NotificationKind,
    priority: NotificationPriority,
    code: &'static str,
    resource_id: Option<Uuid>,
}

/// 在拥有业务写入的同一事务中插入账号通知；通知失败会阻止业务提交。
pub async fn record_account_event(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    event: AccountNotificationEvent,
) -> AppResult<Uuid> {
    if account_id.is_nil() {
        return Err(AppError::Validation("账号通知身份无效".into()));
    }
    let descriptor = event.descriptor()?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO account_notifications
         (id, account_id, kind, priority, code, resource_id, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $2)",
    )
    .bind(id)
    .bind(account_id)
    .bind(descriptor.kind.as_str())
    .bind(descriptor.priority.as_str())
    .bind(descriptor.code)
    .bind(descriptor.resource_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存账号事件通知失败".into()))?;
    Ok(id)
}

impl AccountNotificationEvent {
    fn descriptor(self) -> AppResult<EventDescriptor> {
        let (kind, priority, code, resource_id) = match self {
            Self::PasswordChanged => (
                NotificationKind::AccountSecurity,
                NotificationPriority::Critical,
                "password_changed",
                None,
            ),
            Self::DeviceRevoked { device_id } => (
                NotificationKind::AccountSecurity,
                NotificationPriority::Important,
                "device_revoked",
                Some(valid_resource(device_id)?),
            ),
            Self::SessionRevoked { session_id } => (
                NotificationKind::AccountSecurity,
                NotificationPriority::Important,
                "session_revoked",
                Some(valid_resource(session_id)?),
            ),
            Self::SyncUploadCompleted { mutation_id } => (
                NotificationKind::Sync,
                NotificationPriority::Normal,
                "sync_upload_completed",
                Some(valid_resource(mutation_id)?),
            ),
            Self::SyncDownloadCompleted { device_id } => (
                NotificationKind::Sync,
                NotificationPriority::Normal,
                "sync_download_completed",
                Some(valid_resource(device_id)?),
            ),
            Self::SyncResetCompleted { mutation_id } => (
                NotificationKind::Sync,
                NotificationPriority::Critical,
                "sync_reset_completed",
                Some(valid_resource(mutation_id)?),
            ),
        };
        Ok(EventDescriptor {
            kind,
            priority,
            code,
            resource_id,
        })
    }
}

fn valid_resource(resource_id: Uuid) -> AppResult<Uuid> {
    if resource_id.is_nil() {
        Err(AppError::Validation("账号通知匿名资源标识无效".into()))
    } else {
        Ok(resource_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn business_events_have_fixed_safe_descriptors() {
        let resource_id = Uuid::now_v7();
        for (event, kind, priority, code, resource) in [
            (
                AccountNotificationEvent::PasswordChanged,
                NotificationKind::AccountSecurity,
                NotificationPriority::Critical,
                "password_changed",
                None,
            ),
            (
                AccountNotificationEvent::DeviceRevoked {
                    device_id: resource_id,
                },
                NotificationKind::AccountSecurity,
                NotificationPriority::Important,
                "device_revoked",
                Some(resource_id),
            ),
            (
                AccountNotificationEvent::SessionRevoked {
                    session_id: resource_id,
                },
                NotificationKind::AccountSecurity,
                NotificationPriority::Important,
                "session_revoked",
                Some(resource_id),
            ),
            (
                AccountNotificationEvent::SyncUploadCompleted {
                    mutation_id: resource_id,
                },
                NotificationKind::Sync,
                NotificationPriority::Normal,
                "sync_upload_completed",
                Some(resource_id),
            ),
            (
                AccountNotificationEvent::SyncDownloadCompleted {
                    device_id: resource_id,
                },
                NotificationKind::Sync,
                NotificationPriority::Normal,
                "sync_download_completed",
                Some(resource_id),
            ),
            (
                AccountNotificationEvent::SyncResetCompleted {
                    mutation_id: resource_id,
                },
                NotificationKind::Sync,
                NotificationPriority::Critical,
                "sync_reset_completed",
                Some(resource_id),
            ),
        ] {
            let descriptor = event.descriptor().expect("闭集事件必须能投影");
            assert_eq!(descriptor.kind, kind);
            assert_eq!(descriptor.priority, priority);
            assert_eq!(descriptor.code, code);
            assert_eq!(descriptor.resource_id, resource);
        }
        assert!(
            AccountNotificationEvent::DeviceRevoked {
                device_id: Uuid::nil()
            }
            .descriptor()
            .is_err()
        );
    }
}
