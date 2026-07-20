//! 封装 PostgreSQL 连接池、迁移入口与数据库健康检查。

mod connect;
mod health;
mod migrate;

pub use connect::connect;
pub use health::health;
pub use migrate::migrate;
pub use sqlx::{PgPool, Postgres, Transaction};
