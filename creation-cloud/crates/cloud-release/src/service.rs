//! 保存版本域所需的 PostgreSQL 连接池并提供统一装配入口。

use cloud_store::PgPool;

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
