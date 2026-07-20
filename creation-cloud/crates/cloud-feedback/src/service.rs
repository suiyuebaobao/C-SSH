//! 保存反馈域所需的 PostgreSQL 连接池，具体行为由独立用例实现。

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
