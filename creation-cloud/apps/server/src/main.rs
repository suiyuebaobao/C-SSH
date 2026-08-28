//! Creation Cloud 单进程服务入口，仅负责配置、依赖装配和优雅退出。
//! 业务逻辑必须留在各自模块，禁止在此实现具体用例。

mod admin_overview;
mod app;
mod client_config;
mod command;
mod http_trace;
mod maintenance;
mod maintenance_cli;
mod request_id;
mod runtime;
mod services;
mod shutdown;
mod smtp_mailer;

use anyhow::Result;
use cloud_config::CloudConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let command = command::from_env()?;
    let config = CloudConfig::from_env()?;
    let pool = cloud_store::connect(&config.database_url).await?;
    cloud_store::migrate(&pool).await?;
    match command {
        command::Command::CreateAdmin(admin_login_name) => {
            let mut password = rpassword::prompt_password("管理员密码：")?;
            let confirmation = rpassword::prompt_password("再次输入管理员密码：")?;
            if password != confirmation {
                anyhow::bail!("两次输入的管理员密码不一致");
            }
            let password_hash = cloud_auth::hash_admin_password(&password).await?;
            password.clear();
            cloud_admin::create_local_admin(&pool, &admin_login_name, &password_hash).await?;
            seed_system_model_catalog(&pool).await?;
            println!("管理员账号已创建");
            Ok(())
        }
        command::Command::PromoteAdmin(email) => {
            cloud_admin::promote_registered_admin(&pool, &email).await?;
            seed_system_model_catalog(&pool).await?;
            println!("管理员角色已更新");
            Ok(())
        }
        command::Command::SetAdminLogin {
            registered_email,
            admin_login_name,
        } => {
            cloud_admin::set_registered_admin_login(&pool, &registered_email, &admin_login_name)
                .await?;
            println!("管理员登录名已更新");
            Ok(())
        }
        command::Command::Serve => {
            seed_system_model_catalog(&pool).await?;
            let services = services::AppServices::new(pool, &config)?;
            runtime::serve(services, config).await
        }
        command::Command::MaintenanceRun(task) => {
            let maintenance_config = config.maintenance.clone();
            let services = services::AppServices::new(pool, &config)?;
            maintenance_cli::run(services, maintenance_config, task).await
        }
        command::Command::MaintenanceStatus(task) => {
            let services = services::AppServices::new(pool, &config)?;
            maintenance_cli::status(services, task).await
        }
        command::Command::SchemaMigrate => migration_summary(&pool).await,
    }
}

async fn migration_summary(pool: &cloud_store::PgPool) -> Result<()> {
    let (max_version, count, failed): (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version), 0),
               COUNT(*),
               COUNT(*) FILTER (WHERE NOT success)
        FROM _sqlx_migrations
        "#,
    )
    .fetch_one(pool)
    .await?;
    println!(
        "{}",
        serde_json::json!({
            "count": count,
            "failed": failed,
            "max_version": max_version,
            "status": "migrated"
        })
    );
    Ok(())
}

async fn seed_system_model_catalog(pool: &cloud_store::PgPool) -> Result<()> {
    let changed = cloud_model::Service::new(pool.clone())
        .seed_system_catalog()
        .await?;
    if changed > 0 {
        tracing::info!(changed, "已同步未编辑的系统默认模型目录");
    }
    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("CLOUD_LOG")
        .unwrap_or_else(|_| "creation_cloud=info,tower_http=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();
}
