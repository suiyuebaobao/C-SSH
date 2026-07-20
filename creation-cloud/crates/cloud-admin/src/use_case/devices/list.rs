//! 验证管理员身份并规范化筛选后分页读取非 SSH 设备元数据。

use cloud_domain::{AdminActor, AppResult, Page};

use crate::{
    AdminDevice, AdminDeviceListQuery, Service, model::AdminDeviceListFilter, repository,
    validation,
};

impl Service {
    pub async fn list_devices(
        &self,
        actor: &AdminActor,
        query: AdminDeviceListQuery,
    ) -> AppResult<Page<AdminDevice>> {
        validation::admin_actor(actor)?;
        repository::devices::list::execute(&self.pool, &normalize(query)?).await
    }
}

pub(crate) fn normalize(query: AdminDeviceListQuery) -> AppResult<AdminDeviceListFilter> {
    Ok(AdminDeviceListFilter {
        page: validation::page(query.page),
        account_id: query
            .account_id
            .map(|value| validation::valid_id(value, "账号标识"))
            .transpose()?,
        platform: query.platform,
        revoked: query.revoked,
    })
}
