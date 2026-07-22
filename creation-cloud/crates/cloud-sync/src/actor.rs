//! 从认证中间件给出的完整会话派生同步领域内部身份。
//! 字段保持私有，仓储只能消费该身份，不能由请求参数伪造账号、会话或设备。

use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyncActor {
    account_id: Uuid,
    session_id: Uuid,
    device_id: Uuid,
}

impl SyncActor {
    pub(crate) fn from_session(session: &AuthenticatedSession) -> AppResult<Self> {
        if session.account_id.is_nil() || session.session_id.is_nil() {
            return Err(AppError::Unauthorized("同步会话身份无效".to_owned()));
        }
        let device_id = session
            .device_id
            .filter(|device_id| !device_id.is_nil())
            .ok_or_else(|| AppError::Forbidden("当前会话未绑定有效设备".to_owned()))?;
        Ok(Self {
            account_id: session.account_id,
            session_id: session.session_id,
            device_id,
        })
    }

    pub(crate) const fn account_id(&self) -> Uuid {
        self.account_id
    }

    pub(crate) const fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub(crate) const fn device_id(&self) -> Uuid {
        self.device_id
    }
}
