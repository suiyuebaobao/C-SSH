//! 设备域的配额读取接口与账号覆盖DTO；认证域策略经装配层注入。
use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

pub type DeviceQuotaResolver =
    for<'a, 'c> fn(
        &'a mut Transaction<'c, Postgres>,
    ) -> Pin<Box<dyn Future<Output = AppResult<i64>> + Send + 'a>>;

#[derive(Clone, Debug, Serialize)]
pub struct AccountDevicePolicy {
    pub maximum: Option<i32>,
    pub global_maximum: i64,
    pub effective_maximum: i64,
    pub active_devices: i64,
    pub revision: i64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub enum DeviceLimitChoice {
    Count(i32),
    Inherit(()),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateDevicePolicy {
    pub maximum: DeviceLimitChoice,
    pub expected_revision: i64,
}

impl UpdateDevicePolicy {
    pub(crate) fn validated_maximum(&self) -> AppResult<Option<i32>> {
        if self.expected_revision < 0 {
            return Err(AppError::Validation("设备策略版本无效".to_owned()));
        }
        match self.maximum {
            DeviceLimitChoice::Count(n) if (1..=1000).contains(&n) => Ok(Some(n)),
            DeviceLimitChoice::Inherit(()) => Ok(None),
            _ => Err(AppError::Validation("设备上限必须在1到1000之间".to_owned())),
        }
    }
}
