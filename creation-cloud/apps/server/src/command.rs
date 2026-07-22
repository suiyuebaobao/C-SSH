//! 解析服务启动与带外管理员命令，不把账号或凭据写入日志。

use anyhow::{Result, bail};
use cloud_maintenance::MaintenanceTask;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Serve,
    PromoteAdmin(String),
    SetAdminLogin {
        registered_email: String,
        admin_login_name: String,
    },
    MaintenanceRun(MaintenanceTask),
    MaintenanceStatus(Option<MaintenanceTask>),
}

pub fn from_env() -> Result<Command> {
    parse(std::env::args().skip(1))
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    match arguments.as_slice() {
        [] => Ok(Command::Serve),
        [admin, promote, email] if admin == "admin" && promote == "promote" => {
            Ok(Command::PromoteAdmin(email.clone()))
        }
        [admin, set_login, email, login_name] if admin == "admin" && set_login == "set-login" => {
            Ok(Command::SetAdminLogin {
                registered_email: email.clone(),
                admin_login_name: login_name.clone(),
            })
        }
        [maintenance, run, task, json]
            if maintenance == "maintenance" && run == "run" && json == "--json" =>
        {
            Ok(Command::MaintenanceRun(parse_task(task)?))
        }
        [maintenance, status, json]
            if maintenance == "maintenance" && status == "status" && json == "--json" =>
        {
            Ok(Command::MaintenanceStatus(None))
        }
        [maintenance, status, task, json]
            if maintenance == "maintenance" && status == "status" && json == "--json" =>
        {
            Ok(Command::MaintenanceStatus(Some(parse_task(task)?)))
        }
        _ => bail!(
            "用法：creation-cloud-server [admin promote <registered-email> | admin set-login <registered-email> <admin-login-name> | maintenance run <task> --json | maintenance status [<task>] --json]"
        ),
    }
}

fn parse_task(value: &str) -> Result<MaintenanceTask> {
    MaintenanceTask::from_str(value).map_err(|()| anyhow::anyhow!("未知维护任务"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_commands_without_accepting_credentials_or_extra_arguments() {
        assert_eq!(parse(Vec::new()).expect("空参数应启动服务"), Command::Serve);
        assert!(matches!(
            parse(["admin", "promote", "admin@example.com"].map(str::to_owned)),
            Ok(Command::PromoteAdmin(_))
        ));
        assert_eq!(
            parse(["admin", "set-login", "admin@example.com", "ops_admin"].map(str::to_owned))
                .expect("完整管理员登录名命令应通过解析"),
            Command::SetAdminLogin {
                registered_email: "admin@example.com".to_owned(),
                admin_login_name: "ops_admin".to_owned(),
            }
        );
        assert!(parse(["admin", "promote"].map(str::to_owned)).is_err());
        assert!(parse(["admin", "set-login", "admin@example.com"].map(str::to_owned)).is_err());
        assert!(
            parse(
                [
                    "admin",
                    "set-login",
                    "admin@example.com",
                    "ops_admin",
                    "password-placeholder",
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
        assert!(parse(["serve", "extra"].map(str::to_owned)).is_err());
    }

    #[test]
    fn parses_only_exact_json_maintenance_commands() {
        assert_eq!(
            parse(["maintenance", "run", "expired-sessions", "--json",].map(str::to_owned))
                .expect("完整手动运行命令应通过解析"),
            Command::MaintenanceRun(MaintenanceTask::ExpiredSessions)
        );
        assert_eq!(
            parse(["maintenance", "status", "--json"].map(str::to_owned))
                .expect("全任务状态命令应通过解析"),
            Command::MaintenanceStatus(None)
        );
        assert_eq!(
            parse(["maintenance", "status", "backup-freshness", "--json",].map(str::to_owned))
                .expect("单任务状态命令应通过解析"),
            Command::MaintenanceStatus(Some(MaintenanceTask::BackupFreshness))
        );
        assert!(parse(["maintenance", "run", "expired-sessions"].map(str::to_owned)).is_err());
        assert!(
            parse(["maintenance", "run", "unknown-task", "--json",].map(str::to_owned)).is_err()
        );
        assert!(
            parse(
                [
                    "maintenance",
                    "status",
                    "backup-freshness",
                    "--json",
                    "extra",
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
    }
}
