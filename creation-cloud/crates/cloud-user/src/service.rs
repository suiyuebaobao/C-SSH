//! 持有用户域数据库连接并统一转发五个资料 CRUD 用例。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    Profile,
    use_case::{self, CreateProfile, UpdateProfile},
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
        command: CreateProfile,
    ) -> AppResult<Profile> {
        use_case::create::execute(&self.pool, session, command).await
    }

    pub async fn get(
        &self,
        session: &AuthenticatedSession,
        account_id: Uuid,
    ) -> AppResult<Profile> {
        use_case::get::execute(&self.pool, session, account_id).await
    }

    pub async fn list(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<Profile>> {
        use_case::list::execute(&self.pool, session, page).await
    }

    pub async fn update(
        &self,
        session: &AuthenticatedSession,
        account_id: Uuid,
        command: UpdateProfile,
    ) -> AppResult<Profile> {
        use_case::update::execute(&self.pool, session, account_id, command).await
    }

    pub async fn delete(&self, session: &AuthenticatedSession, account_id: Uuid) -> AppResult<()> {
        use_case::delete::execute(&self.pool, session, account_id).await
    }
}
