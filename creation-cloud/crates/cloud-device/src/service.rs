//! 持有设备域数据库连接并统一转发五个设备 CRUD 用例。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    CreateDeviceOutcome, Device,
    use_case::{self, CreateDevice, UpdateDevice},
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

    pub async fn create(
        &self,
        session: &AuthenticatedSession,
        command: CreateDevice,
    ) -> AppResult<CreateDeviceOutcome> {
        use_case::create::execute(&self.pool, session, command).await
    }

    pub async fn get(&self, session: &AuthenticatedSession, device_id: Uuid) -> AppResult<Device> {
        use_case::get::execute(&self.pool, session, device_id).await
    }

    pub async fn list(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<Device>> {
        use_case::list::execute(&self.pool, session, page).await
    }

    pub async fn update(
        &self,
        session: &AuthenticatedSession,
        device_id: Uuid,
        command: UpdateDevice,
    ) -> AppResult<Device> {
        use_case::update::execute(&self.pool, session, device_id, command).await
    }

    pub async fn delete(&self, session: &AuthenticatedSession, device_id: Uuid) -> AppResult<()> {
        use_case::delete::execute(&self.pool, session, device_id).await
    }
}
