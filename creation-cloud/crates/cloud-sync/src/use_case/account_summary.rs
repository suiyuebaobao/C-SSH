//! 为已认证网页会话提供账号级只读摘要，不改变正式同步设备授权。

use cloud_domain::{AppError, AppResult, AuthenticatedSession};

use crate::{AccountSyncSummary, Service, repository};

impl Service {
    pub async fn account_summary(
        &self,
        session: &AuthenticatedSession,
    ) -> AppResult<AccountSyncSummary> {
        if session.account_id.is_nil() || session.session_id.is_nil() {
            return Err(AppError::Unauthorized("同步摘要会话身份无效".to_owned()));
        }
        repository::account_summary(&self.pool, session).await
    }
}
