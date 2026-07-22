//! 保存同步领域共享的 PostgreSQL 连接池与进程内有界限流器。

use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_store::PgPool;

use crate::{
    actor::SyncActor,
    limiter::{AccessKind, SyncLimiter, SyncPermit},
};

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
    limiter: SyncLimiter,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            limiter: SyncLimiter::default(),
        }
    }

    pub(crate) fn authorize(
        &self,
        session: &AuthenticatedSession,
        kind: AccessKind,
    ) -> AppResult<(SyncActor, SyncPermit)> {
        let actor = SyncActor::from_session(session)?;
        let permit = self.limiter.acquire(actor.account_id(), kind)?;
        Ok((actor, permit))
    }
}
