//! 编排通知本地化投影、账号分页、receipt 与管理员定向投递事务。

use cloud_domain::{
    AdminActor, AppError, AppResult, Page, PageQuery, mark_semantic_audit_recorded,
};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    AccountNotification, AdminNotification, CreateNotificationInput, NotificationListQuery,
    NotificationListResponse, NotificationReceipt, ReceiptInput, repository, validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_account(
        &self,
        account_id: Uuid,
        query: NotificationListQuery,
    ) -> AppResult<NotificationListResponse> {
        if account_id.is_nil() {
            return Err(AppError::Unauthorized("账号会话身份无效".into()));
        }
        let query = validation::list_query(query)?;
        let (mut rows, unread_count) = tokio::try_join!(
            repository::list_account(&self.pool, account_id, &query),
            repository::unread_count(&self.pool, account_id)
        )?;
        let has_more = rows.len() > query.limit as usize;
        if has_more {
            rows.pop();
        }
        let next_cursor = if has_more {
            let last = rows
                .last()
                .ok_or_else(|| AppError::Storage("通知分页状态无效".into()))?;
            Some(validation::encode_cursor(
                validation::priority_rank(&last.priority)?,
                last.published_at,
                last.id,
            ))
        } else {
            None
        };
        let items = rows
            .into_iter()
            .map(|row| localize(row, query.locale))
            .collect::<AppResult<Vec<_>>>()?;
        Ok(NotificationListResponse {
            items,
            next_cursor,
            unread_count,
        })
    }

    pub async fn receipt(
        &self,
        account_id: Uuid,
        notification_id: Uuid,
        input: ReceiptInput,
    ) -> AppResult<NotificationReceipt> {
        if account_id.is_nil() || notification_id.is_nil() {
            return Err(AppError::Validation("通知回执身份无效".into()));
        }
        repository::receipt(
            &self.pool,
            account_id,
            notification_id,
            validation::receipt_revision(input.revision)?,
        )
        .await
    }

    pub async fn list_admin(
        &self,
        actor: &AdminActor,
        page: PageQuery,
    ) -> AppResult<Page<AdminNotification>> {
        require_actor(actor)?;
        repository::list_admin(&self.pool, page).await
    }

    pub async fn create_admin(
        &self,
        actor: &AdminActor,
        input: CreateNotificationInput,
    ) -> AppResult<AdminNotification> {
        let actor_id = require_actor(actor)?;
        let input = validation::create(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AppError::Storage("通知事务启动失败".into()))?;
        let record = repository::create(&mut transaction, actor_id, &input).await?;
        transaction
            .commit()
            .await
            .map_err(|_| AppError::Storage("通知事务提交失败".into()))?;
        mark_semantic_audit_recorded();
        Ok(record)
    }
}

fn localize(
    row: crate::model::NotificationRow,
    locale: validation::Locale,
) -> AppResult<AccountNotification> {
    let kind = repository::parse_kind(&row.kind)?;
    let priority = repository::parse_priority(&row.priority)?;
    let (title, body) = localized_copy(&row.code, locale)?;
    let action = match kind {
        crate::NotificationKind::AccountSecurity => "open_account_security",
        crate::NotificationKind::Sync => "open_sync",
    };
    Ok(AccountNotification {
        id: row.id,
        revision: row.revision,
        kind,
        priority,
        code: row.code,
        resource_id: row.resource_id,
        title: title.to_owned(),
        body: body.to_owned(),
        published_at: row.published_at,
        expires_at: row.expires_at,
        action: action.to_owned(),
        read_state: if row.read_at.is_some() {
            "read"
        } else {
            "unread"
        }
        .to_owned(),
    })
}

fn localized_copy(
    code: &str,
    locale: validation::Locale,
) -> AppResult<(&'static str, &'static str)> {
    let copy = match (code, locale) {
        ("security_review_required", validation::Locale::ZhCn) => {
            ("请检查账号安全", "账号安全状态需要你确认。")
        }
        ("security_review_required", validation::Locale::En) => (
            "Review account security",
            "Your account security state needs review.",
        ),
        ("password_changed", validation::Locale::ZhCn) => {
            ("密码已更新", "账号密码已完成更新，请确认这是你的操作。")
        }
        ("password_changed", validation::Locale::En) => (
            "Password updated",
            "Your account password was updated. Please confirm this was you.",
        ),
        ("device_revoked", validation::Locale::ZhCn) => ("设备已撤销", "一个账号设备已被撤销。"),
        ("device_revoked", validation::Locale::En) => {
            ("Device revoked", "An account device was revoked.")
        }
        ("session_revoked", validation::Locale::ZhCn) => ("会话已撤销", "一个账号会话已被撤销。"),
        ("session_revoked", validation::Locale::En) => {
            ("Session revoked", "An account session was revoked.")
        }
        ("sync_review_required", validation::Locale::ZhCn) => {
            ("请检查同步状态", "Cloud 同步状态需要你确认。")
        }
        ("sync_review_required", validation::Locale::En) => {
            ("Review sync status", "Your Cloud sync state needs review.")
        }
        ("sync_upload_completed", validation::Locale::ZhCn) => {
            ("上传已完成", "本次加密同步上传已经完成。")
        }
        ("sync_upload_completed", validation::Locale::En) => {
            ("Upload completed", "The encrypted sync upload completed.")
        }
        ("sync_download_completed", validation::Locale::ZhCn) => {
            ("下载已完成", "本次加密同步下载已经完成。")
        }
        ("sync_download_completed", validation::Locale::En) => (
            "Download completed",
            "The encrypted sync download completed.",
        ),
        ("sync_reset_completed", validation::Locale::ZhCn) => {
            ("同步数据已重置", "Cloud 端加密同步数据已经重置。")
        }
        ("sync_reset_completed", validation::Locale::En) => {
            ("Sync data reset", "Cloud encrypted sync data was reset.")
        }
        _ => return Err(AppError::Storage("数据库中的通知 code 无效".into())),
    };
    Ok(copy)
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let id = actor.account_id();
    if id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".into()))
    } else {
        Ok(id)
    }
}
