//! 为 runner 的真实 PostgreSQL 测试提供随机 schema、正式迁移和 max=1 连接池夹具。
//! 清理函数只接受本模块固定前缀，失败或 panic 时也不会扩大到其它 schema。

use std::{error::Error, str::FromStr, time::Duration};

use cloud_config::CloudConfig;
use cloud_maintenance::MaintenanceTask;
use cloud_store::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use uuid::Uuid;

use super::super::Runner;
use crate::services::AppServices;

const SCHEMA_PREFIX: &str = "maintenance_runner_nomock_";

pub(super) type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(super) struct TestDatabase {
    database_url: String,
    schema: String,
    pub(super) runner_application_name: String,
    pub(super) runner_pool: PgPool,
    pub(super) observer: PgPool,
}

impl TestDatabase {
    pub(super) async fn connect(database_url: &str, schema: &str) -> TestResult<Self> {
        let runner_application_name = format!("maintenance-runner-{}", Uuid::new_v4().simple());
        let runner_pool =
            connect_pool(database_url, &runner_application_name, Some(schema), 1).await?;
        let observer = match connect_pool(
            database_url,
            "maintenance-runner-observer",
            Some(schema),
            4,
        )
        .await
        {
            Ok(pool) => pool,
            Err(error) => {
                runner_pool.close().await;
                return Err(error);
            }
        };
        Ok(Self {
            database_url: database_url.to_owned(),
            schema: schema.to_owned(),
            runner_application_name,
            runner_pool,
            observer,
        })
    }

    pub(super) fn runner(&self) -> Runner {
        let config = test_config(&self.database_url, &self.schema);
        let maintenance = config.maintenance.clone();
        Runner::new(
            AppServices::new(self.runner_pool.clone(), &config),
            maintenance,
        )
    }

    pub(super) async fn run_count(&self) -> TestResult<i64> {
        Ok(
            sqlx::query_scalar("SELECT count(*) FROM maintenance_task_runs")
                .fetch_one(&self.observer)
                .await?,
        )
    }

    pub(super) async fn running_count(&self, task: MaintenanceTask) -> TestResult<i64> {
        Ok(sqlx::query_scalar(
            "SELECT count(*) FROM maintenance_task_runs WHERE task_name = $1 AND outcome = 'running'",
        )
        .bind(task.as_str())
        .fetch_one(&self.observer)
        .await?)
    }

    pub(super) async fn only_run_id(&self, task: MaintenanceTask) -> TestResult<Uuid> {
        Ok(sqlx::query_scalar(
            "SELECT run_id FROM maintenance_task_runs WHERE task_name = $1 ORDER BY started_at DESC, run_id DESC LIMIT 1",
        )
        .bind(task.as_str())
        .fetch_one(&self.observer)
        .await?)
    }

    pub(super) async fn close(&self) {
        self.runner_pool.close().await;
        self.observer.close().await;
    }
}

fn test_config(database_url: &str, schema: &str) -> CloudConfig {
    let data_root = std::env::temp_dir()
        .join("creation-cloud-maintenance")
        .join(schema);
    CloudConfig {
        bind_addr: "127.0.0.1:8088".parse().expect("固定测试地址必须有效"),
        database_url: database_url.to_owned(),
        public_base_url: "http://127.0.0.1:8088"
            .parse()
            .expect("固定测试 URL 必须有效"),
        google_site_verification: None,
        baidu_site_verification: None,
        download_root: data_root.join("downloads"),
        site_media_root: data_root.join("site-media"),
        session_ttl: Duration::from_secs(3600),
        environment: "test".to_owned(),
        maintenance: cloud_config::MaintenanceConfig::default(),
    }
}

pub(super) struct SchemaGuard {
    database_url: String,
    schema: String,
    active: bool,
}

impl SchemaGuard {
    pub(super) async fn create(database_url: &str) -> TestResult<Self> {
        let schema = format!("{SCHEMA_PREFIX}{}", Uuid::new_v4().simple());
        let admin = connect_pool(database_url, "maintenance-runner-schema-create", None, 1).await?;
        let created = sqlx::query(&format!("CREATE SCHEMA \"{schema}\""))
            .execute(&admin)
            .await
            .map(|_| ())
            .map_err(|_| boxed("无法创建本轮随机独占 schema"));
        admin.close().await;
        created?;
        Ok(Self {
            database_url: database_url.to_owned(),
            schema,
            active: true,
        })
    }

    pub(super) fn name(&self) -> &str {
        &self.schema
    }

    pub(super) async fn cleanup(&mut self) -> TestResult {
        if self.active {
            drop_owned_schema(&self.database_url, &self.schema).await?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for SchemaGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let database_url = std::mem::take(&mut self.database_url);
        let schema = std::mem::take(&mut self.schema);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            // panic 时仍只异步回收带固定随机前缀的本轮 schema，不触碰其它数据。
            std::mem::drop(runtime.spawn(async move {
                let _ = drop_owned_schema(&database_url, &schema).await;
            }));
        }
    }
}

pub(super) async fn migrate_schema(database_url: &str, schema: &str) -> TestResult {
    let pool = connect_pool(
        database_url,
        "maintenance-runner-migrations",
        Some(schema),
        4,
    )
    .await?;
    let result = cloud_store::migrate(&pool)
        .await
        .map_err(|_| boxed("formal migrations 执行失败"));
    pool.close().await;
    result
}

async fn drop_owned_schema(database_url: &str, schema: &str) -> TestResult {
    require(
        schema.starts_with(SCHEMA_PREFIX),
        "拒绝清理非本测试前缀 schema",
    )?;
    let admin = connect_pool(database_url, "maintenance-runner-schema-cleanup", None, 1).await?;
    let result = sqlx::query(&format!("DROP SCHEMA \"{schema}\" CASCADE"))
        .execute(&admin)
        .await
        .map(|_| ())
        .map_err(|_| boxed("无法清理本轮随机独占 schema"));
    admin.close().await;
    result
}

async fn connect_pool(
    database_url: &str,
    application_name: &str,
    schema: Option<&str>,
    max_connections: u32,
) -> TestResult<PgPool> {
    let mut options = PgConnectOptions::from_str(database_url)
        .map_err(|_| boxed("数据库 URL 格式无效"))?
        .application_name(application_name);
    if let Some(schema) = schema {
        options = options.options([("search_path", format!("{schema},public"))]);
    }
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
        .map_err(|_| boxed("无法连接真实 PostgreSQL"))
}

pub(super) fn require(condition: bool, message: impl Into<String>) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(boxed(message))
    }
}

pub(super) fn boxed(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(std::io::Error::other(message.into()))
}
