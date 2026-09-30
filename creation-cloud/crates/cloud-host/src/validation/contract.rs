//! 校验显式同步合同版本及第三资源能力边界。

use crate::{CURRENT_SYNC_CONTRACT_VERSION, LEGACY_SYNC_CONTRACT_VERSION};
use cloud_domain::{AppError, AppResult};

pub(super) fn validate(version: u16) -> AppResult<()> {
    if !(LEGACY_SYNC_CONTRACT_VERSION..=CURRENT_SYNC_CONTRACT_VERSION).contains(&version) {
        return Err(AppError::Validation(
            "unsupported sync_contract_version".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn require_proxy_profiles(version: u16, present: bool) -> AppResult<()> {
    if present && version < 3 {
        return Err(AppError::Validation(
            "proxy profiles require sync contract version 3 or newer".to_owned(),
        ));
    }
    Ok(())
}
