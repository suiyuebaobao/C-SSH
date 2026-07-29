//! 从认证中间件给出的会话派生不可伪造的账号与设备身份。

use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AccountActor {
    account_id: Uuid,
}

impl AccountActor {
    pub(crate) fn from_session(session: &AuthenticatedSession) -> AppResult<Self> {
        if session.account_id.is_nil() || session.session_id.is_nil() {
            return Err(AppError::Unauthorized("当前会话身份无效".to_owned()));
        }
        Ok(Self {
            account_id: session.account_id,
        })
    }

    pub(crate) const fn account_id(self) -> Uuid {
        self.account_id
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeviceActor {
    account_id: Uuid,
    device_id: Uuid,
}

impl DeviceActor {
    pub(crate) fn from_session(session: &AuthenticatedSession) -> AppResult<Self> {
        let account = AccountActor::from_session(session)?;
        let device_id = session
            .device_id
            .filter(|value| !value.is_nil())
            .ok_or_else(|| AppError::Forbidden("当前会话未绑定有效设备".to_owned()))?;
        Ok(Self {
            account_id: account.account_id(),
            device_id,
        })
    }

    pub(crate) const fn account_id(self) -> Uuid {
        self.account_id
    }

    pub(crate) const fn device_id(self) -> Uuid {
        self.device_id
    }
}
