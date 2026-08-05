//! 渲染普通用户双语密码找回申请与确认页面。

use askama::Template;
use axum::{extract::State, response::Html};
use cloud_domain::{AppError, AppResult};
use cloud_site::{Locale, PageId, SiteView, content_service};

use crate::{PublicPageState, seo::SeoHead};

#[derive(Template)]
#[template(path = "forgot-password.html")]
struct ForgotPasswordTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
    captcha_required: bool,
}

#[derive(Template)]
#[template(path = "reset-password.html")]
struct ResetPasswordTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
}

pub(crate) async fn forgot() -> AppResult<Html<String>> {
    render_forgot(Locale::ZhCn, true)
}

pub(crate) async fn forgot_en() -> AppResult<Html<String>> {
    render_forgot(Locale::En, true)
}

pub(crate) async fn forgot_live(State(state): State<PublicPageState>) -> AppResult<Html<String>> {
    let captcha = state.login_captcha_settings().await?;
    render_forgot(Locale::ZhCn, captcha.user_captcha_enabled)
}

pub(crate) async fn forgot_en_live(
    State(state): State<PublicPageState>,
) -> AppResult<Html<String>> {
    let captcha = state.login_captcha_settings().await?;
    render_forgot(Locale::En, captcha.user_captcha_enabled)
}

pub(crate) async fn reset() -> AppResult<Html<String>> {
    render_reset(Locale::ZhCn)
}

pub(crate) async fn reset_en() -> AppResult<Html<String>> {
    render_reset(Locale::En)
}

fn render_forgot(locale: Locale, captcha_required: bool) -> AppResult<Html<String>> {
    let is_en = locale == Locale::En;
    let mut view = content_service().view(PageId::Login, locale);
    view.page.meta_title = if is_en {
        "Forgot password | Creation Cloud"
    } else {
        "找回密码｜Creation Cloud"
    }
    .to_owned();
    view.page.meta_description = if is_en {
        "Request a one-time email code to reset a Creation Cloud password."
    } else {
        "申请一次性邮箱验证码以重置 Creation Cloud 密码。"
    }
    .to_owned();
    view.shell.language_href = if is_en {
        "/forgot-password"
    } else {
        "/en/forgot-password"
    }
    .to_owned();
    ForgotPasswordTemplate {
        view,
        seo: SeoHead::private(),
        is_en,
        captcha_required,
    }
    .render()
    .map(Html)
    .map_err(|_| AppError::Internal("密码找回申请页面暂时无法渲染".to_owned()))
}

fn render_reset(locale: Locale) -> AppResult<Html<String>> {
    let is_en = locale == Locale::En;
    let mut view = content_service().view(PageId::Login, locale);
    view.page.meta_title = if is_en {
        "Reset password | Creation Cloud"
    } else {
        "重置密码｜Creation Cloud"
    }
    .to_owned();
    view.page.meta_description = if is_en {
        "Confirm the one-time email code and set a new Creation Cloud password."
    } else {
        "确认一次性邮箱验证码并设置新的 Creation Cloud 密码。"
    }
    .to_owned();
    view.shell.language_href = if is_en {
        "/reset-password"
    } else {
        "/en/reset-password"
    }
    .to_owned();
    ResetPasswordTemplate {
        view,
        seo: SeoHead::private(),
        is_en,
    }
    .render()
    .map(Html)
    .map_err(|_| AppError::Internal("密码重置页面暂时无法渲染".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_page_uses_the_user_captcha_switch() {
        let enabled = render_forgot(Locale::ZhCn, true)
            .expect("开启图形验证码时找回页面应可渲染")
            .0;
        assert!(enabled.contains("purpose=password_reset"));
        assert!(enabled.contains("name=\"captcha_code\""));
        assert!(enabled.contains("data-captcha-refresh"));

        let disabled = render_forgot(Locale::En, false)
            .expect("关闭图形验证码时找回页面应可渲染")
            .0;
        assert!(!disabled.contains("purpose=password_reset"));
        assert!(!disabled.contains("name=\"captcha_code\""));
    }

    #[test]
    fn reset_page_never_exposes_an_internal_challenge() {
        let body = render_reset(Locale::ZhCn).expect("密码重置页面应可渲染").0;
        assert!(body.contains("href=\"/en/reset-password\""));
        assert!(!body.contains("challenge_id"));
        assert!(!body.contains("专属链接"));
    }
}
