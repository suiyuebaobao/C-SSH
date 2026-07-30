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
        "Regular users follow the current email-verification setting; administrators use a one-time visual CAPTCHA.",
    )
    .with_actions(vec![action("Create an account", "/register", "text-link")])
    .with_form(
        "/web/auth/login",
        "Sign in",
        "When email verification is on, regular users receive a six-digit code; administrators always use username, password and CAPTCHA.",
        vec![
            FormField::new(
                "identifier",
                "Email or admin username",
                "text",
                "username",
                "name@example.com / ops-admin",
            ),
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
    page(PageId::Register, "Register | Creation Cloud", "Create a Creation Cloud account.", "Creation Cloud", "Create an account", "Submit your details and the server will apply the current email-verification setting.")
        .with_actions(vec![action("Already registered? Sign in", "/login", "text-link")])
        .with_form(
            "/web/auth/register",
            "Create account",
            "When email verification is on, enter the six-digit code within 10 minutes; when it is off, a session is created directly.",
            vec![
                FormField::new("display_name", "Display name", "text", "name", "How should we address you?"),
                FormField::new("email", "Email", "email", "email", "name@example.com"),
                FormField::new("password", "Password", "password", "new-password", "At least 12 characters"),
            ],
        )
}
