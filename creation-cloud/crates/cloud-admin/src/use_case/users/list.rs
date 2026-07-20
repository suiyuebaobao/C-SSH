//! 验证管理员身份并规范化精确邮箱筛选后分页列出脱敏账号。

use cloud_domain::{AdminActor, AppResult, Page};

use crate::{
    AdminUser, AdminUserListQuery, Service, model::AdminUserListFilter, repository, validation,
};

impl Service {
    pub async fn list_users(
        &self,
        actor: &AdminActor,
        query: AdminUserListQuery,
    ) -> AppResult<Page<AdminUser>> {
        validation::admin_actor(actor)?;
        repository::users::list::execute(&self.pool, &normalize(query)?).await
    }
}

pub(crate) fn normalize(query: AdminUserListQuery) -> AppResult<AdminUserListFilter> {
    Ok(AdminUserListFilter {
        page: validation::page(query.page),
        email: query
            .email
            .as_deref()
            .map(validation::email_filter)
            .transpose()?,
        role: query.role,
        status: query.status,
    })
}
