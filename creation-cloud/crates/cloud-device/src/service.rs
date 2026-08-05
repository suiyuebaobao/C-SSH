//! 持有设备域数据库连接并统一转发五个设备 CRUD 用例。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    CreateDeviceOutcome, Device, SessionView,
    use_case::{self, CreateDevice, UpdateDevice, create::TrustedRequestMetadata},
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
        self.create_with_metadata(session, command, TrustedRequestMetadata::default())
            .await
    }

    pub async fn create_with_metadata(
        &self,
        session: &AuthenticatedSession,
        command: CreateDevice,
        metadata: TrustedRequestMetadata,
    ) -> AppResult<CreateDeviceOutcome> {
        use_case::create::execute(&self.pool, session, command, metadata).await
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

    pub async fn list_sessions(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<SessionView>> {
        use_case::session::list_self(&self.pool, session, page).await
    }

    /// Returns whether the target belonged to the authenticated account.
    pub async fn revoke_session(
        &self,
        session: &AuthenticatedSession,
        session_id: Uuid,
    ) -> AppResult<bool> {
        use_case::session::revoke_self(&self.pool, session, session_id).await
    }

    pub async fn admin_list_sessions(
        &self,
        session: &AuthenticatedSession,
        account_id: Option<Uuid>,
        page: PageQuery,
    ) -> AppResult<Page<SessionView>> {
        use_case::session::list_admin(&self.pool, session, account_id, page).await
    }

    /// Physically deletes an administrator-selected session.
    pub async fn admin_delete_session(
        &self,
        session: &AuthenticatedSession,
        session_id: Uuid,
    ) -> AppResult<bool> {
        use_case::session::delete_admin(&self.pool, session, session_id).await
    }

    /// Compatibility entrypoint for the server-rendered administrator page.
    pub async fn admin_revoke_session(
        &self,
        session: &AuthenticatedSession,
        session_id: Uuid,
    ) -> AppResult<bool> {
        self.admin_delete_session(session, session_id).await
    }
}
