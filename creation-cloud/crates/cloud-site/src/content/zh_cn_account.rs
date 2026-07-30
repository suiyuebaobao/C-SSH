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
        "普通用户按站点当前设置完成邮箱验证；管理员进入后台时使用一次性图形验证码。",
    )
    .with_actions(vec![action("创建账号", "/register", "text-link")])
    .with_form(
        "/web/auth/login",
        "登录",
        "邮箱验证开启时，普通用户收到六位登录码；管理员始终使用管理员账号、密码和图形验证码。",
        vec![
            FormField::new(
                "identifier",
                "邮箱或管理员账号",
                "text",
                "username",
                "name@example.com / ops-admin",
            ),
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
        "提交资料后，系统会按当前邮箱验证设置激活账号。",
    )
    .with_actions(vec![action("已有账号，去登录", "/login", "text-link")])
    .with_form(
        "/web/auth/register",
        "注册",
        "邮箱验证开启时请输入十分钟内有效的六位验证码；关闭时会直接建立登录会话。",
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
