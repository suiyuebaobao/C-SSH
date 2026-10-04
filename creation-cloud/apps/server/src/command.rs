//! 解析服务启动与带外管理员命令，不把账号或凭据写入日志。

use anyhow::{Result, bail};
use cloud_maintenance::MaintenanceTask;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Serve,
    CreateAdmin(String),
    PromoteAdmin(String),
    SetAdminLogin {
        registered_email: String,
        admin_login_name: String,
    },
    MaintenanceRun(MaintenanceTask),
    MaintenanceStatus(Option<MaintenanceTask>),
    SchemaMigrate,
}

pub fn from_env() -> Result<Command> {
    parse(std::env::args().skip(1))
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    match arguments.as_slice() {
        [] => Ok(Command::Serve),
        [admin, create, login_name] if admin == "admin" && create == "create" => {
            Ok(Command::CreateAdmin(login_name.clone()))
        }
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
        [schema, migrate, json]
            if schema == "schema" && migrate == "migrate" && json == "--json" =>
        {
            Ok(Command::SchemaMigrate)
        }
        _ => bail!(
            "用法：creation-cloud-server [admin promote <registered-email> | admin set-login <registered-email> <admin-login-name> | maintenance run <task> --json | maintenance status [<task>] --json | schema migrate --json]"
        ),
    }
}

fn parse_task(value: &str) -> Result<MaintenanceTask> {
    MaintenanceTask::from_str(value).map_err(|()| anyhow::anyhow!("未知维护任务"))
}
