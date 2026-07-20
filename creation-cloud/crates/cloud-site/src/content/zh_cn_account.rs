//! 提供简体中文登录与注册表单内容。

use crate::{FormField, PageContent, PageId};

use super::zh_cn::{action, page};

pub(super) fn login() -> PageContent {
    page(
        PageId::Login,
        "登录｜Creation Cloud",
        "登录 Creation Cloud 用户中心。",
        "Creation Cloud",
        "登录你的账号",
        "进入设备、同步、模型和零知识保险库概览。",
    )
    .with_actions(vec![action("创建账号", "/register", "text-link")])
    .with_form(
        "/web/auth/login",
        "登录",
        "账号密码只用于登录，保险库密码始终独立。",
        vec![
            FormField::new("email", "邮箱", "email", "email", "name@example.com"),
            FormField::new(
                "password",
                "密码",
                "password",
                "current-password",
                "输入账号密码",
            ),
        ],
    )
}

pub(super) fn register() -> PageContent {
    page(
        PageId::Register,
        "注册｜Creation Cloud",
        "创建 Creation Cloud 账号。",
        "Creation Cloud",
        "创建账号",
        "用一个独立账号管理设备与可选同步。",
    )
    .with_actions(vec![action("已有账号，去登录", "/login", "text-link")])
    .with_form(
        "/web/auth/register",
        "注册",
        "创建账号不代表上传 SSH 主机资料；敏感数据默认不上云。",
        vec![
            FormField::new("display_name", "显示名称", "text", "name", "你的称呼"),
            FormField::new("email", "邮箱", "email", "email", "name@example.com"),
            FormField::new(
                "password",
                "密码",
                "password",
                "new-password",
                "至少 12 个字符",
            ),
        ],
    )
}
