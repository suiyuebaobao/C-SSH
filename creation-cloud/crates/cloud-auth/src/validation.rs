//! 集中维护认证输入的长度、格式和规范化规则。

use cloud_domain::{AppError, AppResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginIdentifierKind {
    Email,
    AdminLoginName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoginIdentifier {
    pub value: String,
    pub kind: LoginIdentifierKind,
}

impl LoginIdentifier {
    #[must_use]
    pub const fn is_admin_login_name(&self) -> bool {
        matches!(self.kind, LoginIdentifierKind::AdminLoginName)
    }
}

pub(crate) fn login_identifier(value: &str) -> AppResult<LoginIdentifier> {
    if value.trim().contains('@') {
        return Ok(LoginIdentifier {
            value: normalize_email(value)?,
            kind: LoginIdentifierKind::Email,
        });
    }
    Ok(LoginIdentifier {
        value: cloud_domain::normalize_admin_login_name(value)?,
        kind: LoginIdentifierKind::AdminLoginName,
    })
}

pub(crate) fn normalize_email(value: &str) -> AppResult<String> {
    let email = value.trim().to_lowercase();
    let valid = email.len() <= 254
        && email
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
    if !valid {
        return Err(AppError::Validation("邮箱格式无效".to_owned()));
    }
    Ok(email)
}

pub(crate) fn password(value: &str) -> AppResult<()> {
    if !(12..=128).contains(&value.len()) {
        return Err(AppError::Validation(
            "密码长度必须为 12 至 128 字节".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn login_password(value: &str) -> AppResult<()> {
    if value.len() > 128 {
        return Err(AppError::Validation("登录密码不得超过 128 字节".to_owned()));
    }
    Ok(())
}

pub(crate) fn display_name(value: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 80 {
        return Err(AppError::Validation(
            "显示名称长度必须为 1 至 80 个字符".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub(crate) fn locale(value: &str) -> AppResult<String> {
    match value {
        "zh-CN" | "en" => Ok(value.to_owned()),
        _ => Err(AppError::Validation("语言设置无效".to_owned())),
    }
}
