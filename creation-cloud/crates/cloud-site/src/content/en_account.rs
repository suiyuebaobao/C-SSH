//! 提供英文登录与注册表单内容。

use crate::{FormField, PageContent, PageId};

use super::en::{action, page};

pub(super) fn login() -> PageContent {
    page(
        PageId::Login,
        "Sign in | Creation Cloud",
        "Sign in to the Creation Cloud console.",
        "Creation Cloud",
        "Sign in to your account",
        "Open your device, sync, model, and zero-knowledge vault overview.",
    )
    .with_actions(vec![action("Create an account", "/register", "text-link")])
    .with_form(
        "/web/auth/login",
        "Sign in",
        "Your account password is only for sign-in; the vault password always remains separate.",
        vec![
            FormField::new("email", "Email", "email", "email", "name@example.com"),
            FormField::new(
                "password",
                "Password",
                "password",
                "current-password",
                "Enter your account password",
            ),
        ],
    )
}

pub(super) fn register() -> PageContent {
    page(PageId::Register, "Register | Creation Cloud", "Create a Creation Cloud account.", "Creation Cloud", "Create an account", "Use one separate account for devices and optional sync.")
        .with_actions(vec![action("Already registered? Sign in", "/login", "text-link")])
        .with_form(
            "/web/auth/register",
            "Create account",
            "Creating an account does not upload SSH host details; sensitive data stays local by default.",
            vec![
                FormField::new("display_name", "Display name", "text", "name", "How should we address you?"),
                FormField::new("email", "Email", "email", "email", "name@example.com"),
                FormField::new("password", "Password", "password", "new-password", "At least 12 characters"),
            ],
        )
}
