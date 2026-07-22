//! 定义管理员登录名的唯一规范化规则，供认证与带外管理命令共同复用。

use crate::{AppError, AppResult};

pub const ADMIN_LOGIN_NAME_MIN_LEN: usize = 3;
pub const ADMIN_LOGIN_NAME_MAX_LEN: usize = 32;

/// 去除首尾空白并统一为小写 ASCII 管理员登录名。
pub fn normalize_admin_login_name(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_ascii_lowercase();
    let bytes = normalized.as_bytes();
    let valid = (ADMIN_LOGIN_NAME_MIN_LEN..=ADMIN_LOGIN_NAME_MAX_LEN).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(*byte, b'-' | b'_')
        });
    if !valid {
        return Err(AppError::Validation("管理员登录名格式无效".to_owned()));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::normalize_admin_login_name;

    #[test]
    fn normalizes_supported_ascii_name() {
        assert_eq!(
            normalize_admin_login_name("  Ops_Admin-01  ").expect("合法名称应规范化"),
            "ops_admin-01"
        );
    }

    #[test]
    fn rejects_non_letter_prefix_and_unsupported_characters() {
        for invalid in ["1admin", "op", "ops.admin", "管理员", "ops admin"] {
            assert!(
                normalize_admin_login_name(invalid).is_err(),
                "应拒绝：{invalid}"
            );
        }
    }
}
