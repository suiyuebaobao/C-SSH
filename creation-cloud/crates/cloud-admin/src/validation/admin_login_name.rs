//! 校验并规范化带外命令提交的管理员登录名。

use cloud_domain::AppResult;

pub(crate) fn normalize(value: &str) -> AppResult<String> {
    cloud_domain::normalize_admin_login_name(value)
}
