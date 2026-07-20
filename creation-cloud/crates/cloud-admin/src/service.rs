//! 保存管理域 PostgreSQL 连接池并提供统一构造入口。

pub(crate) mod promote;

use cloud_store::PgPool;

pub use promote::promote_registered_admin;

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
