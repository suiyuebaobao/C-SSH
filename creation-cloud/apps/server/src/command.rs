//! 解析服务启动与带外管理员提升命令，不把账号或凭据写入日志。

use anyhow::{Result, bail};

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Serve,
    PromoteAdmin(String),
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
        _ => bail!("用法：creation-cloud-server [admin promote <registered-email>]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_serve_and_promote_without_accepting_extra_arguments() {
        assert_eq!(parse(Vec::new()).expect("空参数应启动服务"), Command::Serve);
        assert!(matches!(
            parse(["admin", "promote", "admin@example.com"].map(str::to_owned)),
            Ok(Command::PromoteAdmin(_))
        ));
        assert!(parse(["admin", "promote"].map(str::to_owned)).is_err());
        assert!(parse(["serve", "extra"].map(str::to_owned)).is_err());
    }
}
