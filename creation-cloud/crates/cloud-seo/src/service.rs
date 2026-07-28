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
