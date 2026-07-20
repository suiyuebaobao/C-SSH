//! 保存保险库领域唯一需要的 PostgreSQL 连接池。

use cloud_store::PgPool;

#[derive(Clone)]
pub struct Service {
    pub(crate) pool: PgPool,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
